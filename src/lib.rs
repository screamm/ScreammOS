#![no_std]
#![no_main]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)]

use core::panic::PanicInfo;
use core::fmt::Arguments;
use crate::vga_buffer::_print;
use crate::simple_fs::SimpleString;

pub mod vga_buffer;
pub mod interrupts;
pub mod keyboard;
pub mod ui;
pub mod simple_fs;
pub mod gdt;
pub mod logger;
pub mod queue;
pub mod error_handler;
pub mod string_ext;

pub mod ui {
    pub mod window_manager;
    pub mod command_line;
    pub mod text_editor;
    pub mod file_manager;
    pub mod splash_screen;
    pub mod retro_commands;
}

#[macro_export]
macro_rules! format {
    ($($arg:tt)*) => ({
        let mut s = SimpleString::new();
        s.write_fmt(format_args!($($arg)*)).unwrap();
        s
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! vec {
    ($($x:expr),*) => (
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
            temp_vec
        }
    );
}

/// Initialize core OS components
pub fn init() {
    // Initialize GDT (Global Descriptor Table)
    gdt::init();
    
    // Initialize logger first for early logging
    logger::init();
    log_info!("System initialization started");
    
    // Initialize IDT (Interrupt Descriptor Table)
    interrupts::init_idt();
    log_info!("Interrupt descriptor table initialized");
    
    // Initialize and enable PIC (Programmable Interrupt Controller)
    unsafe { 
        interrupts::PICS.lock().initialize();
        log_info!("Programmable interrupt controller initialized");
    }
    
    // Enable interrupts
    x86_64::instructions::interrupts::enable();
    log_info!("Interrupts enabled");
    
    // Initialize filesystem
    simple_fs::init();
    log_info!("Filesystem initialized");
    
    // Initialize UI
    ui::init();
    log_info!("User interface initialized");
    
    // Initialize error handling
    error_handler::init();
    log_info!("Error handler initialized");
    
    log_info!("System initialization completed successfully");
}

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for `cargo test`
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &core::panic::PanicInfo) -> ! {
    println!("[failed]\n");
    println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

/// Loop som väntar på avbrott med hlt-instruktionen
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        println!("{}...\t", core::any::type_name::<T>());
        self();
        println!("[ok]");
    }
}

/// Startar om systemet
pub fn reboot() -> ! {
    println!("Startar om...");
    
    // Använd 8042 PS/2-kontrollern för att utlösa en systemomstart
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0x64);
        port.write(0xFE as u8);
    }
    
    // Om omstarten misslyckades, vänta bara i en loop
    hlt_loop();
}

pub fn run_self_tests() {
    println!("Running self tests...");
    // Add self tests here
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("ScreammOS v0.1.0");
    init();
    run_self_tests();
    println!("Boot sequence completed!");
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn trivial_assertion() {
        assert_eq!(1, 1);
    }
} 