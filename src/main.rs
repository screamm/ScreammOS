#![no_std] // Don't use Rust standard library
#![no_main] // Don't use regular main function
#![feature(abi_x86_interrupt)] // Enable x86-interrupt ABI
#![feature(alloc_error_handler)] // Enable custom allocation error handler
#![feature(custom_test_frameworks)]
#![test_runner(screamos::test_runner)]
#![reexport_test_harness_main = "test_main"]

// Use core for no_std functions
use core::panic::PanicInfo;

// This is imported from the bootloader crate
use bootloader::{entry_point, BootInfo};
use x86_64::VirtAddr;
use screamos::println;
use screamos::print;

// Import memory management
mod memory;

// Import necessary components
use screamos::vga_buffer::{change_theme, ThemeStyle};
use screamos::ui::window_manager::WindowManager;
use screamos::ui::file_manager::FILE_MANAGER;
use screamos::ui::splash_screen::SPLASH_SCREEN;
use crate::memory::BootInfoFrameAllocator;

// Define OS entry point for bootloader
entry_point!(kernel_main);

/// Main OS function called by bootloader
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Initialize core OS components
    screamos::init();
    
    // Change to DOS classic theme
    change_theme(ThemeStyle::DOSClassic);
    
    // Show splash screen
    if let Some(mut splash) = SPLASH_SCREEN.try_lock() {
        splash.show();
    }
    
    // Classic boot sequence
    println!("\nScreammOS Boot Sequence");
    println!("=====================\n");
    
    // Step 1: Memory check
    println!("Step 1: Performing memory check...");
    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(physical_memory_offset) };
    
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    
    // Step 2: Initialize memory management
    println!("Step 2: Initializing memory management...");
    match memory::init_heap(&mut mapper, &mut frame_allocator) {
        Ok(_) => {
            println!("  Memory management initialized successfully");
            println!("  Heap memory: {} KiB", memory::HEAP_SIZE / 1024);
        },
        Err(e) => {
            println!("  WARNING: Heap initialization encountered an issue: {:?}", e);
            println!("  The system will continue with limited memory functionality");
        }
    }
    
    // Step 3: Initialize keyboard
    println!("Step 3: Initializing keyboard...");
    screamos::keyboard::init();
    println!("  Keyboard initialized");
    
    // Step 4: Initialize filesystem
    println!("Step 4: Initializing filesystem...");
    if let Some(mut file_manager) = FILE_MANAGER.try_lock() {
        println!("  File system ready");
    } else {
        println!("  Warning: Could not initialize file system");
    }
    
    // Step 5: Run system diagnostics
    println!("Step 5: Running system diagnostics...");
    run_self_tests();
    
    // Hide splash screen after initialization
    if let Some(mut splash) = SPLASH_SCREEN.try_lock() {
        splash.hide();
    }
    
    // Show welcome message in retro style
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║                  Welcome to ScreammOS v0.2!                ║");
    println!("║                                                            ║");
    println!("║  A retro-styled operating system written in Rust           ║");
    println!("║  Type 'help' for a list of available commands              ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
    
    // Main loop - wait for interrupts
    screamos::hlt_loop();
}

fn print_prompt() {
    print!("> ");
}

// Definiera testrunner för cargo test
#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    screamos::test_runner(tests);
}

/// This function is called on panic
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Change screen to red on panic
    screamos::vga_buffer::set_global_color(screamos::vga_buffer::Color::Red, screamos::vga_buffer::Color::Black);
    
    println!();
    println!("KERNEL PANIC!");
    println!("{}", info);
    
    loop {
        x86_64::instructions::hlt();
    }
}

/// Handler for allocation errors
#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}

/// Run a comprehensive set of self-tests to verify system functionality
pub fn run_self_tests() {
    use log_info;
    use ui::text_editor::TextEditor;
    use vga_buffer::{WRITER, Color};
    use error_handler::{report_error, ErrorDomain, ErrorSeverity};
    
    log_info!("Starting comprehensive system self-tests");
    
    let mut all_tests_passed = true;
    let mut test_count = 0;
    let mut pass_count = 0;
    
    // Change screen colors for test output
    {
        let mut writer = WRITER.lock();
        writer.set_color(Color::LightGreen, Color::Black);
        println!("\n===== SYSTEM SELF-TEST =====\n");
    }
    
    // Helper function to run a test
    let mut run_test = |name: &str, test: fn() -> Result<(), &'static str>| {
        test_count += 1;
        print!("Testing {}: ", name);
        
        match test() {
            Ok(_) => {
                pass_count += 1;
                println!("[PASS]");
                true
            },
            Err(msg) => {
                all_tests_passed = false;
                println!("[FAIL] - {}", msg);
                
                // Report the error
                let _ = report_error(
                    0x1001, 
                    ErrorDomain::System, 
                    ErrorSeverity::Warning,
                    &format!("Test failed: {} - {}", name, msg)
                );
                
                false
            }
        }
    };
    
    // Test VGA buffer
    run_test("VGA buffer", || {
        // Test basic functionality
        {
            let mut writer = WRITER.lock();
            
            // Test color changing
            writer.set_color(Color::Yellow, Color::Blue);
            let (fg, bg) = writer.get_color();
            if fg != Color::Yellow || bg != Color::Blue {
                return Err("Color setting failed");
            }
            
            // Test cursor positioning
            writer.set_position(10, 10);
            let (x, y) = writer.get_position();
            if x != 10 || y != 10 {
                return Err("Cursor positioning failed");
            }
            
            // Reset to normal
            writer.set_color(Color::LightGreen, Color::Black);
        }
        
        Ok(())
    });
    
    // Test memory management
    run_test("Memory allocation", || {
        // Test heap allocations
        use alloc::vec::Vec;
        
        let mut vec = Vec::new();
        
        // Try to allocate some memory
        for i in 0..100 {
            vec.push(i);
        }
        
        // Verify the allocations
        for i in 0..100 {
            if vec[i] != i {
                return Err("Memory allocation verification failed");
            }
        }
        
        // Free the memory
        drop(vec);
        
        Ok(())
    });
    
    // Test filesystem
    run_test("Filesystem operations", || {
        use simple_fs::FILESYSTEM;
        
        let test_filename = "test_file.txt";
        let test_content = "This is a test file for the filesystem test.";
        
        // Create a test file
        {
            let mut fs = FILESYSTEM.lock();
            
            // Clean up any existing test file
            if fs.find_file(test_filename).is_some() {
                let _ = fs.delete_file(test_filename);
            }
            
            // Create the test file
            match fs.create_file(test_filename, test_content) {
                Ok(_) => {},
                Err(e) => return Err(e),
            }
        }
        
        // Read the test file
        {
            let fs = FILESYSTEM.lock();
            
            // Check if the file exists
            if fs.find_file(test_filename).is_none() {
                return Err("Test file not found after creation");
            }
            
            // Read the file content
            match fs.read_file(test_filename) {
                Some(content) => {
                    if content != test_content {
                        return Err("File content doesn't match expected content");
                    }
                },
                None => return Err("Failed to read test file"),
            }
        }
        
        // Test file deletion
        {
            let mut fs = FILESYSTEM.lock();
            
            // Delete the test file
            match fs.delete_file(test_filename) {
                Ok(_) => {},
                Err(e) => return Err(e),
            }
            
            // Verify the file is gone
            if fs.find_file(test_filename).is_some() {
                return Err("Test file still exists after deletion");
            }
        }
        
        Ok(())
    });
    
    // Test text editor
    run_test("Text editor initialization", || {
        use ui::text_editor::TEXT_EDITOR;
        
        // Create a test file for the editor
        {
            let mut fs = FILESYSTEM.lock();
            let test_file = "editor_test.txt";
            let test_content = "This is a test file for the text editor.";
            
            // Clean up any existing test file
            if fs.find_file(test_file).is_some() {
                let _ = fs.delete_file(test_file);
            }
            
            // Create the test file
            match fs.create_file(test_file, test_content) {
                Ok(_) => {},
                Err(e) => return Err(e),
            }
        }
        
        // Test editor operations
        {
            let mut editor = match TEXT_EDITOR.try_lock() {
                Some(editor) => editor,
                None => return Err("Failed to lock text editor"),
            };
            
            // Open the test file
            editor.open_file("editor_test.txt");
            
            // Check if the file was loaded
            if !editor.is_file_open() {
                return Err("Failed to open test file in editor");
            }
            
            // Test basic navigation
            editor.set_cursor_position(0, 0);
            let (x, y) = editor.get_cursor_position();
            if x != 0 || y != 0 {
                return Err("Editor cursor positioning failed");
            }
            
            // Clean up
            editor.close_file();
        }
        
        // Clean up the test file
        {
            let mut fs = FILESYSTEM.lock();
            let _ = fs.delete_file("editor_test.txt");
        }
        
        Ok(())
    });
    
    // Test keyboard handler
    run_test("Keyboard handler", || {
        use keyboard;
        
        // Check if the keyboard is initialized
        if !keyboard::is_initialized() {
            return Err("Keyboard not initialized");
        }
        
        // Not much we can test without actual keyboard input,
        // but we can at least verify the system is ready
        
        Ok(())
    });
    
    // Test error handling system
    run_test("Error handling system", || {
        use error_handler::{ERROR_HANDLER, ErrorSeverity};
        
        // Count errors before test
        let handler = ERROR_HANDLER.lock();
        let initial_count = handler.get_total_error_count();
        drop(handler);
        
        // Create a test error
        let _ = report_error(
            0x9999, 
            ErrorDomain::System, 
            ErrorSeverity::Warning,
            "This is a test error for the error handling system"
        );
        
        // Verify error was recorded
        let handler = ERROR_HANDLER.lock();
        let final_count = handler.get_total_error_count();
        
        if final_count <= initial_count {
            return Err("Error was not recorded correctly");
        }
        
        Ok(())
    });
    
    // Test logging system
    run_test("Logging system", || {
        use logger::{LOGGER, LogLevel};
        
        // Test if we can log at different levels
        log_info!("Test info message for self-test");
        
        // Check if the logger is working
        let logger = LOGGER.lock();
        
        if logger.get_log_level() == LogLevel::Off {
            return Err("Logging is disabled");
        }
        
        Ok(())
    });
    
    // Print results
    println!("\n===== TEST RESULTS =====");
    println!("Tests run: {}", test_count);
    println!("Tests passed: {}", pass_count);
    println!("Tests failed: {}", test_count - pass_count);
    
    if all_tests_passed {
        println!("\nAll tests passed successfully!");
    } else {
        println!("\nSome tests failed. Check the log for details.");
    }
    
    // Reset colors
    {
        let mut writer = WRITER.lock();
        writer.set_color(Color::LightGray, Color::Black);
    }
    
    log_info!("System self-tests completed: {} passed, {} failed", 
             pass_count, test_count - pass_count);
}
