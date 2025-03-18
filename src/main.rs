#![no_std] // Don't use Rust standard library
#![no_main] // Don't use regular main function
#![feature(abi_x86_interrupt)] // Enable x86-interrupt ABI
#![feature(alloc_error_handler)] // Enable custom allocation error handler

// Use core for no_std functions
use core::panic::PanicInfo;

// This is imported from the bootloader crate
use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;

// Import VGA buffer functionality
mod vga_buffer;

// Import UI modules
mod ui;

// Import interrupt handling
mod interrupts;

// Import keyboard handling
mod keyboard;

// Import memory management
mod memory;

// Import necessary components
use vga_buffer::{change_theme, ThemeStyle};
use ui::{window_manager::WindowManager};

// Define OS entry point for bootloader
entry_point!(kernel_main);

/// Main OS function called by bootloader
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Clear screen with DOS blue theme
    change_theme(ThemeStyle::DOSClassic);
    
    // Display a stylish ASCII art ScreammOS logo
    println!("");
    println!("                                         _____ _____ ");
    println!("                                        / ____/ ____|");
    println!("  ___  ___ _ __ ___  __ _ _ __ ___    | (___| (___  ");
    println!(" / __|/ __| '__/ _ \\/ _` | '_ ` _ \\    \\___ \\\\___ \\ ");
    println!(" \\__ \\ (__| | |  __/ (_| | | | | | |   ____) |___) |");
    println!(" |___/\\___|_|  \\___|\\__,_|_| |_| |_|  |_____/_____/ ");
    println!("");
    println!(" Welcome to ScreammOS - The Retro-modern Experience");
    println!(" Version 0.1.0 - Prototype");
    println!("");
    println!(" ScreammOS starting up...");
    
    // Initialize interrupt handling
    interrupts::init();
    
    // Initialize memory management
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    
    // Initialize the heap
    memory::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Heap initialization failed");
    
    println!(" Memory management initialized");
    println!(" Heap memory: {} KiB", memory::HEAP_SIZE / 1024);
    
    // Initialize keyboard handling
    keyboard::init();
    
    // Create window manager
    let mut window_manager = WindowManager::new();
    
    // Create DOS classic theme
    let dos_theme = ui::Theme::dos_classic();
    
    // Show welcome window
    window_manager.show_message(
        "Welcome", 
        "ScreammOS 0.1.0 - Type 'help' for commands", 
        dos_theme
    );
    
    // Main loop - wait for interrupts
    loop {
        // Use hlt instruction to save power while waiting for interrupts
        x86_64::instructions::hlt();
    }
}

/// This function is called on panic
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Switch to red screen on panic
    vga_buffer::WRITER.lock().set_color(vga_buffer::Color::White, vga_buffer::Color::Red);
    println!("KERNEL PANIC: {}", info);
    loop {}
}

/// Handler for allocation errors
#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}
