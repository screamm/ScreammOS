#![no_std] // Använder inte Rust standardbibliotek
#![no_main] // Använder inte vanlig main-funktion

use core::panic::PanicInfo;

// Den här importeras från bootloader-crate
use bootloader::{BootInfo, entry_point};

// Importera VGA-buffertfunktionalitet
mod vga_buffer;

// Importera UI-moduler
mod ui;

// Importera nödvändiga komponenter
use vga_buffer::{change_theme, ThemeStyle};
use ui::{Theme, window_manager::WindowManager};

// Definiera OS-entry point för bootloader
entry_point!(kernel_main);

/// Huvud OS-funktion som anropas av bootloader
fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    // Rensa skärmen med DOS-blå tema
    change_theme(ThemeStyle::DOSClassic);
    
    // Visa en stilig ASCII-konst ScreammOS-logotyp
    println!("");
    println!("                                         _____ _____ ");
    println!("                                        / ____/ ____|");
    println!("  ___  ___ _ __ ___  __ _ _ __ ___    | (___| (___  ");
    println!(" / __|/ __| '__/ _ \\/ _` | '_ ` _ \\    \\___ \\\\___ \\ ");
    println!(" \\__ \\ (__| | |  __/ (_| | | | | | |   ____) |___) |");
    println!(" |___/\\___|_|  \\___|\\__,_|_| |_| |_|  |_____/_____/ ");
    println!("");
    println!(" Välkommen till ScreammOS - Den Retro-moderna Upplevelsen");
    println!(" Version 0.1.0 - Prototyp");
    println!("");
    println!(" ScreammOS startar upp...");
    
    // Skapa fönsterhanteraren och visa ett välkomstfönster
    let mut window_manager = WindowManager::new();
    
    // Skapa DOS-klassiskt tema
    let dos_theme = ui::Theme::dos_classic();
    
    // Visa välkomstfönstret
    window_manager.show_message(
        "Välkommen", 
        "ScreammOS 0.1.0 - Tryck F1 för hjälp", 
        dos_theme
    );
    
    // Huvudloop
    loop {
        // Här kommer senare kod för tangentbordshantering och kommandon
    }
}

/// Denna funktion anropas vid panik
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Byt till röd skärm vid panik
    vga_buffer::WRITER.lock().set_color(vga_buffer::Color::White, vga_buffer::Color::Red);
    println!("KERNEL PANIC: {}", info);
    loop {}
}
