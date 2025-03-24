use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
        OffsetPageTable, PhysFrame, PageTable,
    },
    PhysAddr, VirtAddr,
};
use linked_list_allocator::LockedHeap;
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use crate::println;

// Define the kernel heap size
pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 500 * 1024; // 500 KiB (increased for filesystem)

// Create a global heap allocator
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Initialize a new OffsetPageTable.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Returns a mutable reference to the active level 4 page table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
    -> &'static mut PageTable
{
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
    // Håll reda på de senaste ramarna som har tilldelats för att undvika dubbla tilldelningar
    allocated_frames: [u64; 64], // Vi håller bara de senaste 64 ramarna för enkelhetens skull
    allocated_count: usize,
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
            allocated_frames: [0; 64],
            allocated_count: 0,
        }
    }
    
    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // Get usable regions from memory map
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        // Map each region to its address range
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // Transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // Create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }

    // Kolla om en ram redan är allokerad
    fn is_frame_allocated(&self, frame: PhysFrame) -> bool {
        let frame_addr = frame.start_address().as_u64();
        for i in 0..self.allocated_count {
            if self.allocated_frames[i] == frame_addr {
                return true;
            }
        }
        false
    }

    // Lägg till en ram till listan över allokerade ramar
    fn mark_frame_allocated(&mut self, frame: PhysFrame) {
        let frame_addr = frame.start_address().as_u64();
        if self.allocated_count < self.allocated_frames.len() {
            self.allocated_frames[self.allocated_count] = frame_addr;
            self.allocated_count += 1;
        } else {
            // Om listan är full, starta om från början (cirkulär buffer)
            for i in 0..(self.allocated_frames.len() - 1) {
                self.allocated_frames[i] = self.allocated_frames[i + 1];
            }
            self.allocated_frames[self.allocated_frames.len() - 1] = frame_addr;
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let mut frame_iter = self.usable_frames().skip(self.next);
        
        // Hitta nästa lediga ram som inte är allokerad
        let frame = loop {
            let frame = frame_iter.next()?;
            
            // Öka next-räknaren så vi inte hamnar i en oändlig loop
            self.next += 1;
            
            // Kolla om ramen redan är allokerad
            if !self.is_frame_allocated(frame) {
                // Markera ramen som allokerad och returnera den
                self.mark_frame_allocated(frame);
                break frame;
            }
        };
        
        Some(frame)
    }
}

// Initialize the heap memory
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    // Map the heap pages
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    // Create the page tables and allocate frames for the heap
    for page in page_range {
        // Försök att mappa sidan upp till 3 gånger om det behövs
        let mut attempts = 0;
        let max_attempts = 3;
        
        loop {
            match unsafe { try_map_page(page, mapper, frame_allocator) } {
                Ok(_) => break,  // Lyckades mappa sidan
                Err(MapToError::FrameAllocationFailed) if attempts < max_attempts => {
                    // Försök igen med en annan ram
                    attempts += 1;
                    continue;
                }
                Err(MapToError::PageAlreadyMapped(_)) => {
                    // Sidan är redan mappad, vi kan hoppa över den
                    println!("INFO: Page {:?} was already mapped", page);
                    break;
                }
                Err(e) => return Err(e),  // Annat fel, returnera det
            }
        }
    }

    // Initialize the allocator with the heap area
    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }

    Ok(())
}

// Hjälpfunktion för att försöka mappa en sida
unsafe fn try_map_page(
    page: Page,
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let frame = frame_allocator
        .allocate_frame()
        .ok_or(MapToError::FrameAllocationFailed)?;
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    mapper.map_to(page, frame, flags, frame_allocator)?.flush();
    Ok(())
}

// Functions for memory information

// Get total memory size
pub fn get_total_memory() -> usize {
    HEAP_SIZE
}

// Get used memory
pub fn get_used_memory() -> usize {
    // Since LockedHeap doesn't have a stats method in this version,
    // we'll just return 0 for now
    0
}

// Get free memory
pub fn get_free_memory() -> usize {
    // Since LockedHeap doesn't have a stats method in this version,
    // we'll just return the total heap size for now
    HEAP_SIZE
}

// Struct to collect memory stats
pub struct MemoryStats {
    pub total: usize,
    pub used: usize,
    pub free: usize,
}

// Get memory statistics
pub fn get_memory_stats() -> MemoryStats {
    MemoryStats {
        total: get_total_memory(),
        used: get_used_memory(),
        free: get_free_memory(),
    }
} 