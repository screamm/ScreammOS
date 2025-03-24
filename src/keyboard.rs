use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyCode};
use spin::Mutex;
use x86_64::instructions::port::Port;
use crate::{print, println};
use crate::simple_fs::SimpleString;
use crate::vga_buffer::{clear_screen, get_current_theme, set_theme, Theme};
use crate::ui::file_manager::FILE_MANAGER;
use crate::ui::text_editor::TEXT_EDITOR;
use crate::queue::ArrayQueue;
use core::sync::atomic::{AtomicBool, Ordering};
use crate::{log_info, log_warn, log_error};
use crate::error_handler::{report_error, report_warning, ErrorDomain, ErrorSeverity};
use crate::vga_buffer::Color;
use crate::ui::command_line::CommandLine;

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = 
        Mutex::new(Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore));
    static ref CURRENT_LINE: Mutex<crate::simple_fs::SimpleString> = Mutex::new(crate::simple_fs::SimpleString::new());
    static ref SCANCODE_QUEUE: Mutex<Option<ArrayQueue<u8>>> = Mutex::new(None);
    static ref KEYBOARD_COMMAND: Mutex<SimpleString> = Mutex::new(SimpleString::new());
    static ref KEYBOARD_STATE: Mutex<KeyboardState> = Mutex::new(KeyboardState::new());
}

// Buffer to store the last command
pub static COMMAND_BUFFER: Mutex<CommandBuffer> = Mutex::new(CommandBuffer::new());

// Global keyboard state
static KEYBOARD_INITIALIZED: AtomicBool = AtomicBool::new(false);
const SCANCODE_QUEUE_SIZE: usize = 100;

pub struct KeyboardState {
    pub command: SimpleString,
    pub is_shift_pressed: bool,
    pub is_ctrl_pressed: bool,
    pub is_alt_pressed: bool,
}

impl KeyboardState {
    pub fn new() -> Self {
        KeyboardState {
            command: SimpleString::new(),
            is_shift_pressed: false,
            is_ctrl_pressed: false,
            is_alt_pressed: false,
        }
    }
}

/// Initialize the keyboard systems and set up interrupt handler
pub fn init() {
    log_info!("Initializing keyboard");
    
    // Initialize the scancode queue
    let mut scancode_queue = SCANCODE_QUEUE.lock();
    *scancode_queue = Some(ArrayQueue::new(SCANCODE_QUEUE_SIZE));
    drop(scancode_queue);
    
    KEYBOARD_INITIALIZED.store(true, Ordering::SeqCst);
    
    log_info!("Keyboard initialized successfully");
}

/// Returns true if the keyboard has been initialized
pub fn is_initialized() -> bool {
    KEYBOARD_INITIALIZED.load(Ordering::SeqCst)
}

/// Add a scancode to the queue, safely
pub fn add_scancode(scancode: u8) {
    if let Some(queue) = &mut *SCANCODE_QUEUE.lock() {
        if queue.is_full() {
            // Queue is full, this could happen if processing can't keep up
            log_warn!("Keyboard scancode queue overflow");
            report_warning(ErrorDomain::IO, "Keyboard scancode queue overflow").ok();
            return;
        }
        
        queue.push(scancode);
    } else {
        // This should never happen if init is called properly
        log_error!("Keyboard scancode queue uninitialized");
        report_error(
            0x4001, 
            ErrorDomain::IO, 
            ErrorSeverity::Error, 
            "Keyboard scancode queue uninitialized"
        ).ok();
    }
}

/// Process keyboard input with error recovery
pub fn process_keypress() {
    let mut keyboard = KEYBOARD.lock();
    
    let scancode = match get_scancode() {
        Some(code) => code,
        None => return,
    };
    
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => {
                    // Handle the character
                    handle_character(character);
                }
                DecodedKey::RawKey(key) => {
                    // Handle special keys if needed
                    // print!("{:?}", key);
                }
            }
        }
    } else {
        // Error decoding scancode, log but continue
        log_warn!("Failed to decode scancode: {}", scancode);
    }
}

/// Get a scancode from the queue, or None if empty
fn get_scancode() -> Option<u8> {
    // Check if keyboard is initialized
    if !is_initialized() {
        log_error!("Keyboard not initialized in get_scancode");
        return None;
    }
    
    // Try to get a scancode from the queue
    let mut queue_guard = SCANCODE_QUEUE.try_lock();
    
    if let Some(queue_guard) = queue_guard.as_mut() {
        if let Some(queue) = queue_guard.as_mut() {
            return queue.pop();
        }
    }
    
    // If we couldn't lock the queue, it might be in use by another interrupt
    // Just skip this scancode to prevent deadlocks
    None
}

/// Handle a character input
fn handle_character(character: char) {
    use crate::ui::UI;
    
    // Get a lock on the UI
    match crate::ui::UI_STATE.try_lock() {
        Some(mut ui) => {
            // Handle the character in the UI
            ui.handle_input(character);
        },
        None => {
            // We couldn't get a lock on the UI
            // This could happen during context switches or if there's contention
            log_warn!("Could not get UI lock in keyboard handler");
        }
    }
}

/// Read a scancode directly from the keyboard controller
pub fn read_scancode() -> u8 {
    let mut port = Port::new(0x60);
    unsafe { port.read() }
}

// Print the command prompt
fn print_prompt() {
    print!("> ");
}

// Handle a scancode from the keyboard controller
pub fn handle_scancode(scancode: u8) {
    let mut keyboard = KEYBOARD.lock();
    
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            process_special_key(key);
            process_normal_key(key);
        }
    }
}

/// Hantera 'speciella' tangentkombinationer och kortkommandon
fn process_special_key(key: DecodedKey) {
    match key {
        DecodedKey::Unicode(c) => match c {
            '\u{0003}' => println!("\nCtrl+C: Avbrott!"),
            '\u{0008}' => {
                if let Some(mut text_editor) = TEXT_EDITOR.try_lock() {
                    if text_editor.visible {
                        text_editor.handle_backspace();
                        return;
                    }
                }
                handle_backspace();
            },
            // F1 - hjälp
            '\u{0011}' => {
                if let Some(mut file_manager) = FILE_MANAGER.try_lock() {
                    if file_manager.visible {
                        file_manager.hide();
                    } else {
                        file_manager.show();
                    }
                    return;
                }
            },
            // Escape - stäng textredigerare
            '\u{001B}' => {
                if let Some(mut text_editor) = TEXT_EDITOR.try_lock() {
                    if text_editor.visible {
                        text_editor.hide();
                        return;
                    }
                }
            },
            // Ctrl+S - spara fil i redigeraren
            '\u{0013}' => {
                if let Some(mut text_editor) = TEXT_EDITOR.try_lock() {
                    if text_editor.visible {
                        text_editor.save_file();
                        return;
                    }
                }
            },
            _ => {}
        },
        DecodedKey::RawKey(key) => match key {
            KeyCode::ArrowUp => {
                if let Some(mut file_manager) = FILE_MANAGER.try_lock() {
                    if file_manager.visible {
                        file_manager.navigate_up();
                        return;
                    }
                }
                if let Some(mut text_editor) = TEXT_EDITOR.try_lock() {
                    if text_editor.visible {
                        text_editor.move_up();
                        return;
                    }
                }
            },
            KeyCode::ArrowDown => {
                if let Some(mut file_manager) = FILE_MANAGER.try_lock() {
                    if file_manager.visible {
                        file_manager.navigate_down();
                        return;
                    }
                }
                if let Some(mut text_editor) = TEXT_EDITOR.try_lock() {
                    if text_editor.visible {
                        text_editor.move_down();
                        return;
                    }
                }
            },
            KeyCode::ArrowLeft => {
                if let Some(mut text_editor) = TEXT_EDITOR.try_lock() {
                    if text_editor.visible {
                        text_editor.move_left();
                        return;
                    }
                }
            },
            KeyCode::ArrowRight => {
                if let Some(mut text_editor) = TEXT_EDITOR.try_lock() {
                    if text_editor.visible {
                        text_editor.move_right();
                        return;
                    }
                }
            },
            KeyCode::Delete => {
                if let Some(mut text_editor) = TEXT_EDITOR.try_lock() {
                    if text_editor.visible {
                        text_editor.handle_delete();
                        return;
                    }
                }
            },
            KeyCode::Return => {
                if let Some(mut file_manager) = FILE_MANAGER.try_lock() {
                    if file_manager.visible {
                        file_manager.open_selected();
                        return;
                    }
                }
                handle_enter();
            },
            _ => {}
        },
    }
}

/// Hantera vanlig teckenimatning
fn process_normal_key(key: DecodedKey) {
    match key {
        DecodedKey::Unicode(c) => {
            // Om textredigeraren är aktiv, skicka tecknet dit
            if let Some(mut text_editor) = TEXT_EDITOR.try_lock() {
                if text_editor.visible {
                    text_editor.insert_char(c);
                    return;
                }
            }
            
            // Annars skriv tecknet i terminalen
            handle_printable_character(c);
        },
        DecodedKey::RawKey(_) => {},
    }
}

/// Hantera backspace
fn handle_backspace() {
    let mut current_line = CURRENT_LINE.lock();
    
    if current_line.len() > 0 {
        current_line.pop();
        print!("\u{0008} \u{0008}"); // Ta bort tecknet från skärmen
    }
}

/// Hantera enter
fn handle_enter() {
    println!();
    
    // Skapa en kopia av kommandot istället för att hålla en referens
    let command_copy = {
        let current_line = CURRENT_LINE.lock();
        if current_line.is_empty() {
            crate::simple_fs::SimpleString::new()
        } else {
            let mut copy = crate::simple_fs::SimpleString::new();
            copy.push_str(current_line.as_str());
            copy
        }
    };
    
    if !command_copy.is_empty() {
        process_command(&command_copy);
    } else {
        print_prompt();
    }
    
    CURRENT_LINE.lock().clear();
}

/// Hantera skrivbara tecken
fn handle_printable_character(c: char) {
    if c.is_control() {
        return;
    }
    
    // Lägg till tecknet i kommandoraden
    CURRENT_LINE.lock().push(c);
    print!("{}", c);
}

/// Hantera kommandon i kommandoraden
fn process_command(command: &SimpleString) {
    // Enkel parsing av kommandoraden
    let mut parts = [""; 10]; // Max 10 argument
    let mut current_part = 0;
    let mut start = 0;
    
    // Hitta alla icke-tomma delar av kommandot
    for (i, c) in command.as_str().char_indices() {
        if c.is_whitespace() {
            if i > start {
                if current_part < parts.len() {
                    parts[current_part] = &command.as_str()[start..i];
                    current_part += 1;
                }
            }
            start = i + 1;
        }
    }
    
    // Lägg till sista delen om den finns
    if start < command.len() && current_part < parts.len() {
        parts[current_part] = &command.as_str()[start..];
        current_part += 1;
    }
    
    if current_part == 0 {
        print_prompt();
        return;
    }

    let mut handled = true;
    
    match parts[0] {
        "help" => {
            println!("Available commands:");
            println!("  help     - Display this help");
            println!("  clear    - Clear the screen");
            println!("  exit     - Exit ScreammOS");
            println!("  sysinfo  - Display system information");
            println!("  about    - Show information about ScreammOS");
            println!("  edit     - Open the text editor with a file (e.g., edit file.txt)");
            println!("  files    - Open the file manager");
            println!("  theme    - Change color theme (theme dark|light|retro)");
            println!("  write    - Write text to a file (e.g., write file.txt Hello world)");
            println!("  cat      - Display the contents of a file (e.g., cat file.txt)");
            println!("  ls       - List files in the current directory");
            println!("\nUpcoming features:");
            println!("  pwd, cd, mkdir, touch, echo");
        },
        "clear" => {
            clear_screen();
        },
        "exit" => {
            println!("Shutting down ScreammOS...");
            x86_64::instructions::hlt();
        },
        "sysinfo" => {
            println!("ScreammOS System Information");
            println!("---------------------------");
            println!("Version: 0.2.0");
            println!("Features: Keyboard, Text Mode, Filesystem");
            println!("Color Theme: {}", get_current_theme());
        },
        "about" => {
            println!("ScreammOS");
            println!("--------");
            println!("An experimental DOS-inspired operating system");
            println!("developed in Rust for x86_64 architecture.");
            println!("\nFeatures:");
            println!("- Keyboard support");
            println!("- Text editor");
            println!("- File manager");
            println!("- Customizable color themes");
        },
        "edit" => {
            if parts[1] != "" {
                let filename = parts[1];
                if let Some(mut text_editor) = TEXT_EDITOR.try_lock() {
                    if text_editor.open_file(filename) {
                        text_editor.show();
                    } else {
                        println!("Could not open file: {}", filename);
                    }
                }
            } else {
                println!("Usage: edit <filename>");
            }
        },
        "files" => {
            if let Some(mut file_manager) = FILE_MANAGER.try_lock() {
                file_manager.show();
            }
        },
        "theme" => {
            if parts[1] != "" {
                match parts[1] {
                    "dark" => set_theme(Theme::Modern),
                    "light" => set_theme(Theme::Classic),
                    "retro" => set_theme(Theme::Green),
                    _ => println!("Invalid theme. Use: dark, light, or retro"),
                }
            } else {
                println!("Specify a theme: dark, light, or retro");
            }
        },
        "write" => {
            if parts[1] != "" {
                let filename = parts[1];
                // Combine all remaining parts as text content
                let mut content = SimpleString::new();
                
                for i in 2..parts.len() {
                    if parts[i] == "" {
                        break;
                    }
                    
                    if i > 2 {
                        content.push(' ');
                    }
                    content.push_str(parts[i]);
                }
                
                let mut fs = crate::simple_fs::FILESYSTEM.lock();
                match fs.create_file(filename, content.as_str()) {
                    Ok(_) => println!("Wrote to file: {}", filename),
                    Err(_) => println!("Could not write to file: {}", filename),
                }
            } else {
                println!("Usage: write <filename> <content>");
            }
        },
        "cat" => {
            if parts[1] != "" {
                let filename = parts[1];
                let fs = crate::simple_fs::FILESYSTEM.lock();
                match fs.read_file(filename) {
                    Ok(content) => {
                        println!("----- {} -----", filename);
                        println!("{}", content);
                        println!("----- End of {} -----", filename);
                    },
                    Err(_) => println!("Could not read file: {}", filename),
                }
            } else {
                println!("Usage: cat <filename>");
            }
        },
        "ls" => {
            let fs = crate::simple_fs::FILESYSTEM.lock();
            println!("Contents of current directory:");
            let mut found = false;
            
            for (file_type, name, size) in fs.list_directory() {
                let type_str = match file_type {
                    crate::simple_fs::FileType::Regular => "File",
                    crate::simple_fs::FileType::Directory => "Dir",
                };
                println!("{:<5} {:<20} {:>8} bytes", type_str, name, size);
                found = true;
            }
            
            if !found {
                println!("(Directory is empty)");
            }
        },
        _ => {
            handled = false;
        }
    }
    
    // If the command wasn't handled, show an error message
    if !handled {
        println!("Unknown command: {}", command.as_str());
        println!("Type 'help' for help");
    }
    
    // Visa prompten igen efter kommandot
    print_prompt();
}

// Hjälpfunktion för att hämta nästa tecken från tangentbordet
fn next_character() -> Option<char> {
    // Kontrollera om det finns en scancode tillgänglig
    let mut status_port = Port::new(0x64);
    let status: u8 = unsafe { status_port.read() };
    
    // Om bit 0 av statusregistret är satt är utdatabufferten full (det finns data)
    if status & 1 != 0 {
        // Hämta scancoden
        let mut data_port = Port::new(0x60);
        let scancode: u8 = unsafe { data_port.read() };
        
        // Behandla scancoden
        let mut keyboard = KEYBOARD.lock();
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(c) => {
                        return Some(c);
                    },
                    _ => {}
                }
            }
        }
    }
    
    None
}

// Hjälpfunktion för att dela upp kommandoraden i delar
fn parse_command(command: &str) -> [&str; 16] {
    let mut result = [""; 16];
    let mut in_part = false;
    let mut start = 0;
    let mut index = 0;
    
    for (i, c) in command.char_indices() {
        if c.is_whitespace() {
            if in_part {
                if index < result.len() {
                    result[index] = &command[start..i];
                    index += 1;
                }
                in_part = false;
            }
        } else {
            if !in_part {
                start = i;
                in_part = true;
            }
        }
    }
    
    // Lägg till den sista delen om det finns en
    if in_part && index < result.len() {
        result[index] = &command[start..];
    }
    
    result
}

// Command buffer to store characters as they are typed
pub struct CommandBuffer {
    buffer: [u8; 256], // Maximum command length
    position: usize,   // Current position in buffer
}

impl CommandBuffer {
    // Create a new empty command buffer
    pub const fn new() -> Self {
        Self {
            buffer: [0; 256],
            position: 0,
        }
    }
    
    // Add a character to the buffer
    pub fn push(&mut self, c: char) -> bool {
        // Only handle ASCII characters
        if c.is_ascii() && self.position < self.buffer.len() - 1 {
            self.buffer[self.position] = c as u8;
            self.position += 1;
            true
        } else {
            false // Non-ASCII or buffer is full
        }
    }
    
    // Remove the last character from the buffer
    pub fn backspace(&mut self) -> bool {
        if self.position > 0 {
            self.position -= 1;
            self.buffer[self.position] = 0;
            true
        } else {
            false // Buffer is empty
        }
    }
    
    // Get the current command as a string
    pub fn get_command(&self) -> &str {
        let slice = &self.buffer[0..self.position];
        // Safe because we only allow ASCII characters in push()
        unsafe { core::str::from_utf8_unchecked(slice) }
    }
    
    // Clear the buffer
    pub fn clear(&mut self) {
        for i in 0..self.position {
            self.buffer[i] = 0;
        }
        self.position = 0;
    }
} 