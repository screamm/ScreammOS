// src/logger.rs
// Simple logging system for ScreammOS

use core::fmt::{self, Write};
use lazy_static::lazy_static;
use spin::Mutex;
use crate::simple_fs::{FILESYSTEM, SimpleString};
use crate::print;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

pub struct Logger {
    level: LogLevel,
    buffer: [Option<SimpleString>; 50],  // Circular buffer for last 50 log entries
    buffer_index: usize,
    buffer_full: bool,
    log_to_console: bool,
    log_to_file: bool,
}

impl Logger {
    pub const fn new() -> Self {
        Self {
            level: LogLevel::Info,
            buffer: [None; 50],
            buffer_index: 0,
            buffer_full: false,
            log_to_console: true,
            log_to_file: true,
        }
    }

    pub fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    pub fn set_log_to_console(&mut self, enabled: bool) {
        self.log_to_console = enabled;
    }

    pub fn set_log_to_file(&mut self, enabled: bool) {
        self.log_to_file = enabled;
    }

    pub fn log(&mut self, level: LogLevel, message: &str) {
        if level as u8 >= self.level as u8 {
            // Format timestamp (simple counter for now)
            let timestamp = unsafe { SYSTEM_TIMER };
            
            // Format log message
            let mut entry = SimpleString::new();
            match level {
                LogLevel::Debug => entry.push_str("[DEBUG]"),
                LogLevel::Info => entry.push_str("[INFO]"),
                LogLevel::Warning => entry.push_str("[WARN]"),
                LogLevel::Error => entry.push_str("[ERROR]"),
                LogLevel::Critical => entry.push_str("[CRIT]"),
            }
            
            entry.push_str(" [");
            // Add timestamp as a simple number
            let mut tmp = timestamp;
            if tmp == 0 {
                entry.push('0');
            } else {
                let mut digits = [0u8; 20];
                let mut i = 0;
                while tmp > 0 {
                    digits[i] = (tmp % 10) as u8 + b'0';
                    tmp /= 10;
                    i += 1;
                }
                while i > 0 {
                    i -= 1;
                    entry.push(digits[i] as char);
                }
            }
            entry.push_str("] ");
            entry.push_str(message);
            
            // Store in circular buffer
            self.buffer[self.buffer_index] = Some(entry);
            self.buffer_index = (self.buffer_index + 1) % self.buffer.len();
            if self.buffer_index == 0 {
                self.buffer_full = true;
            }
            
            // Output to console if enabled
            if self.log_to_console {
                match level {
                    LogLevel::Debug => print!("\x1B[90m"),   // Dark gray
                    LogLevel::Info => print!("\x1B[37m"),    // White
                    LogLevel::Warning => print!("\x1B[93m"), // Yellow
                    LogLevel::Error => print!("\x1B[91m"),   // Light red
                    LogLevel::Critical => print!("\x1B[31m"), // Red
                }
                print!("{}\x1B[0m\n", entry.as_str());
            }
            
            // Save to log file if enabled
            if self.log_to_file {
                let _ = self.append_to_log_file(entry.as_str());
            }
        }
    }
    
    fn append_to_log_file(&self, message: &str) -> Result<(), &'static str> {
        // For now, we'll simply create a new file each time
        // In a real implementation, we would append to an existing file
        let mut fs = FILESYSTEM.lock();
        
        // Try to read existing log
        let current_content = match fs.read_file("system.log") {
            Ok(content) => {
                let mut s = SimpleString::new();
                s.push_str(content);
                s
            },
            Err(_) => SimpleString::new(),
        };
        
        // Append new message
        let mut new_content = SimpleString::new();
        if current_content.len() > 0 {
            new_content.push_str(current_content.as_str());
            new_content.push_str("\n");
        }
        new_content.push_str(message);
        
        // Truncate if too long (keep last 4KB)
        let max_log_size = 4096;
        if new_content.len() > max_log_size {
            // Find a newline near the cutoff point
            let mut cutoff = new_content.len() - max_log_size;
            while cutoff < new_content.len() {
                if new_content.as_str().as_bytes()[cutoff] == b'\n' {
                    cutoff += 1;
                    break;
                }
                cutoff += 1;
            }
            
            // Create a new string with just the tail
            let mut truncated = SimpleString::new();
            truncated.push_str(&new_content.as_str()[cutoff..]);
            new_content = truncated;
        }
        
        // Write back to file
        match fs.create_file("system.log", new_content.as_str()) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
    
    pub fn dump_log(&self) {
        let count = if self.buffer_full { self.buffer.len() } else { self.buffer_index };
        let start = if self.buffer_full { self.buffer_index } else { 0 };
        
        print!("\n--- System Log (Last {} entries) ---\n", count);
        for i in 0..count {
            let index = (start + i) % self.buffer.len();
            if let Some(entry) = self.buffer[index] {
                print!("{}\n", entry.as_str());
            }
        }
        print!("--- End of Log ---\n\n");
    }
}

// Global logger instance
lazy_static! {
    pub static ref LOGGER: Mutex<Logger> = Mutex::new(Logger::new());
}

// System timer for timestamps (incremented by timer interrupt)
static mut SYSTEM_TIMER: u64 = 0;

// Function to increment the system timer (called from timer interrupt handler)
pub fn increment_timer() {
    unsafe { SYSTEM_TIMER += 1; }
}

// Convenience macros for logging
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut s = $crate::simple_fs::SimpleString::new();
            let _ = write!(s, $($arg)*);
            $crate::logger::LOGGER.lock().log($crate::logger::LogLevel::Debug, s.as_str());
        }
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut s = $crate::simple_fs::SimpleString::new();
            let _ = write!(s, $($arg)*);
            $crate::logger::LOGGER.lock().log($crate::logger::LogLevel::Info, s.as_str());
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut s = $crate::simple_fs::SimpleString::new();
            let _ = write!(s, $($arg)*);
            $crate::logger::LOGGER.lock().log($crate::logger::LogLevel::Warning, s.as_str());
        }
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut s = $crate::simple_fs::SimpleString::new();
            let _ = write!(s, $($arg)*);
            $crate::logger::LOGGER.lock().log($crate::logger::LogLevel::Error, s.as_str());
        }
    };
}

#[macro_export]
macro_rules! log_crit {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut s = $crate::simple_fs::SimpleString::new();
            let _ = write!(s, $($arg)*);
            $crate::logger::LOGGER.lock().log($crate::logger::LogLevel::Critical, s.as_str());
        }
    };
}

// Initialize the logger
pub fn init() {
    log_info!("Logger initialized");
}

pub fn log_to_file(level: LogLevel, message: &str) {
    let mut fs = FILESYSTEM.lock();
    let timestamp = get_timestamp();
    let level_str = match level {
        LogLevel::Info => "INFO",
        LogLevel::Warning => "WARN",
        LogLevel::Error => "ERROR",
        LogLevel::Critical => "CRIT",
    };
    
    let log_entry = format!("[{}] {}: {}\n", timestamp, level_str, message);
    
    let current_content = match fs.read_file("system.log") {
        Some(content) => content,
        None => "",
    };
    
    let new_content = format!("{}{}", current_content, log_entry);
    fs.write_file("system.log", &new_content);
}

fn get_timestamp() -> &'static str {
    // TODO: Implementera riktig tidsstämpel när vi har RTC
    "00:00:00"
} 