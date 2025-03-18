use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;
use crate::{print, println};

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = 
        Mutex::new(Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore));
}

// Buffer to store the last command
pub static COMMAND_BUFFER: Mutex<CommandBuffer> = Mutex::new(CommandBuffer::new());

// Initialize keyboard handling
pub fn init() {
    // Any initialization code can go here
}

// Handle a scancode from the keyboard controller
pub fn handle_scancode(scancode: u8) {
    let mut keyboard = KEYBOARD.lock();
    let mut command_buffer = COMMAND_BUFFER.lock();

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => {
                    match character {
                        '\n' => {
                            // Handle enter key
                            println!();
                            
                            // Process the command (will be implemented later)
                            let command = command_buffer.get_command();
                            if !command.is_empty() {
                                process_command(command);
                            }
                            
                            // Reset buffer and display prompt for the next command
                            command_buffer.clear();
                            print!(">");
                        },
                        '\u{8}' => { // Backspace
                            if command_buffer.backspace() {
                                print!("\u{8} \u{8}"); // Erase the character on screen
                            }
                        },
                        _ => {
                            if command_buffer.push(character) {
                                print!("{}", character);
                            }
                        }
                    }
                },
                DecodedKey::RawKey(key) => {
                    // Handle special keys like F1, F2, etc.
                    // For now, we just print the key
                    println!("{:?}", key);
                }
            }
        }
    }
}

// Process a command from the command buffer
fn process_command(command: &str) {
    match command {
        "help" => {
            println!("Available commands:");
            println!("  help    - Show this help");
            println!("  clear   - Clear the screen");
            println!("  version - Show ScreammOS version");
            println!("  theme   - Change the theme");
        },
        "clear" => {
            // Use the VGA buffer to clear the screen
            // This will be implemented later
            crate::vga_buffer::WRITER.lock().clear_screen();
            println!("Screen cleared");
        },
        "version" => {
            println!("ScreammOS 0.1.0 - Prototype");
        },
        "theme" => {
            println!("Available themes:");
            println!("  dos     - Classic DOS blue");
            println!("  amber   - Amber terminal");
            println!("  green   - Green CRT");
            println!("  modern  - Modern dark");
            println!("");
            println!("Use 'theme [name]' to change theme");
        },
        _ if command.starts_with("theme ") => {
            let theme_name = command.trim_start_matches("theme ").trim();
            match theme_name {
                "dos" => {
                    crate::vga_buffer::change_theme(crate::vga_buffer::ThemeStyle::DOSClassic);
                    println!("Theme changed to DOS Classic");
                },
                "amber" => {
                    crate::vga_buffer::change_theme(crate::vga_buffer::ThemeStyle::AmberTerminal);
                    println!("Theme changed to Amber Terminal");
                },
                "green" => {
                    crate::vga_buffer::change_theme(crate::vga_buffer::ThemeStyle::GreenCRT);
                    println!("Theme changed to Green CRT");
                },
                "modern" => {
                    crate::vga_buffer::change_theme(crate::vga_buffer::ThemeStyle::Modern);
                    println!("Theme changed to Modern");
                },
                _ => {
                    println!("Unknown theme: {}", theme_name);
                }
            }
        },
        "" => {
            // Empty command, do nothing
        },
        _ => {
            println!("Unknown command: {}", command);
            println!("Type 'help' for available commands");
        }
    }
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