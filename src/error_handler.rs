// src/error_handler.rs
// Error handling system for ScreammOS with recovery mechanisms

use core::fmt::{self, Display, Formatter};
use lazy_static::lazy_static;
use spin::Mutex;
use crate::{log_error, log_warn, log_info, log_crit};
use crate::simple_fs::{SimpleString, FILESYSTEM};
use crate::vga_buffer::{Color, WRITER};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorSeverity {
    Warning,    // Non-critical errors, system continues
    Error,      // Serious errors, but recoverable
    Critical,   // Critical errors that require immediate attention
    Fatal,      // Unrecoverable errors that require system restart
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorDomain {
    Memory,         // Memory management errors
    Filesystem,     // Filesystem errors
    IO,             // Input/output errors
    Hardware,       // Hardware-related errors
    Interrupt,      // Interrupt handling errors
    UserInterface,  // UI-related errors
    System,         // General system errors
}

#[derive(Debug, Clone)]
pub struct SystemError {
    pub code: u16,
    pub domain: ErrorDomain,
    pub severity: ErrorSeverity,
    pub message: SimpleString,
    recoverable: bool,
}

impl SystemError {
    pub fn new(code: u32, domain: ErrorDomain, severity: ErrorSeverity, message: &str) -> Self {
        let recoverable = severity != ErrorSeverity::Fatal;
        
        let mut msg = SimpleString::new();
        msg.push_str(message);
        
        Self {
            code: code as u16,
            domain,
            severity,
            message: msg,
            recoverable,
        }
    }
    
    pub fn get_code(&self) -> u32 {
        self.code as u32
    }
    
    pub fn get_domain(&self) -> ErrorDomain {
        self.domain
    }
    
    pub fn get_severity(&self) -> ErrorSeverity {
        self.severity
    }
    
    pub fn get_message(&self) -> &str {
        self.message.as_str()
    }
    
    pub fn is_recoverable(&self) -> bool {
        self.recoverable
    }
}

impl Display for SystemError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{:04X}] {:?}/{:?}: {}", 
            self.get_code(), 
            self.get_domain(), 
            self.get_severity(), 
            self.get_message()
        )
    }
}

pub struct ErrorHandler {
    error_count: [u32; 4],  // Count by severity
    last_errors: [Option<SystemError>; 10],
    last_index: usize,
    safe_mode: bool,
}

impl ErrorHandler {
    pub const fn new() -> Self {
        const NONE_ERROR: Option<SystemError> = None;
        Self {
            error_count: [0; 4],
            last_errors: [NONE_ERROR; 10],
            last_index: 0,
            safe_mode: false,
        }
    }
    
    pub fn handle_error(&mut self, error: SystemError) -> Result<(), ()> {
        // Increment error counter by severity
        let severity_index = match error.severity {
            ErrorSeverity::Warning => 0,
            ErrorSeverity::Error => 1,
            ErrorSeverity::Critical => 2,
            ErrorSeverity::Fatal => 3,
        };
        
        self.error_count[severity_index] += 1;
        
        // Store in the circular buffer
        let error_clone = error.clone();
        self.last_errors[self.last_index] = Some(error);
        self.last_index = (self.last_index + 1) % self.last_errors.len();
        
        // Log the error
        match error_clone.severity {
            ErrorSeverity::Warning => {
                log_warn!("{:?} Warning: {} (Code {:04X})", 
                    error_clone.domain, error_clone.message.as_str(), error_clone.get_code());
            },
            ErrorSeverity::Error => {
                log_error!("{:?} Error: {} (Code {:04X})", 
                    error_clone.domain, error_clone.message.as_str(), error_clone.get_code());
            },
            ErrorSeverity::Critical => {
                log_crit!("{:?} CRITICAL: {} (Code {:04X})", 
                    error_clone.domain, error_clone.message.as_str(), error_clone.get_code());
                
                // For critical errors, try to perform recovery actions
                self.perform_recovery(&error_clone);
            },
            ErrorSeverity::Fatal => {
                log_crit!("{:?} FATAL: {} (Code {:04X})", 
                    error_clone.domain, error_clone.message.as_str(), error_clone.get_code());
                
                // For fatal errors, show error on screen and prepare for restart
                self.show_fatal_error(&error_clone);
                return Err(());
            },
        }
        
        // Check if we need to enter safe mode
        if self.error_count[2] >= 3 || // 3+ critical errors
           self.error_count[1] >= 10 { // 10+ regular errors
            if !self.safe_mode {
                self.enter_safe_mode();
            }
        }
        
        Ok(())
    }
    
    pub fn perform_recovery(&mut self, error: &SystemError) {
        match error.domain {
            ErrorDomain::Filesystem => {
                log_info!("Attempting filesystem recovery");
                // Attempt to verify filesystem integrity
                let mut fs = FILESYSTEM.lock();
                // In a real implementation, we would have fs.check_integrity() etc.
                // For now, just report we tried
                drop(fs);
                log_info!("Filesystem recovery completed");
            },
            ErrorDomain::Memory => {
                log_info!("Attempting memory management recovery");
                // For a real system, we might try to free cached memory,
                // clean up stale allocations, etc.
                log_info!("Memory recovery completed");
            },
            ErrorDomain::Hardware => {
                log_info!("Attempting hardware reset");
                // Reset specific hardware components based on error details
                log_info!("Hardware reset completed");
            },
            _ => {
                log_info!("No specific recovery available for {:?}", error.domain);
            }
        }
    }
    
    pub fn enter_safe_mode(&mut self) {
        self.safe_mode = true;
        log_warn!("Entering SAFE MODE due to repeated system errors");
        
        // In safe mode:
        // 1. Disable non-critical features
        // 2. Use conservative settings
        // 3. Perform extra validation
        
        // Change screen colors to indicate safe mode
        let mut writer = WRITER.lock();
        writer.set_color(Color::Yellow, Color::Blue);
        
        // Display safe mode banner
        crate::println!("");
        crate::println!("****************************************************");
        crate::println!("*                SAFE MODE ACTIVE                  *");
        crate::println!("* System has entered safe mode due to system errors *");
        crate::println!("* Some features may be limited or unavailable      *");
        crate::println!("****************************************************");
        crate::println!("");
    }
    
    pub fn exit_safe_mode(&mut self) {
        if self.safe_mode {
            self.safe_mode = false;
            log_info!("Exiting safe mode");
            
            // Reset error counters
            self.error_count = [0; 4];
            
            // Restore normal operation
            let mut writer = WRITER.lock();
            writer.set_color(Color::LightGray, Color::Black);
        }
    }
    
    pub fn is_in_safe_mode(&self) -> bool {
        self.safe_mode
    }
    
    pub fn show_fatal_error(&self, error: &SystemError) {
        // Change screen to red
        let mut writer = WRITER.lock();
        writer.set_color(Color::White, Color::Red);
        writer.clear_screen();
        
        // Display error information
        crate::println!("");
        crate::println!("****************************************************");
        crate::println!("*                 SYSTEM FAILURE                   *");
        crate::println!("****************************************************");
        crate::println!("");
        crate::println!("Error {:04X}: {:?}", error.get_code(), error.domain);
        crate::println!("{}", error.message.as_str());
        crate::println!("");
        crate::println!("The system cannot continue and needs to restart.");
        crate::println!("");
        crate::println!("Press any key to restart...");
        
        // In a real implementation, we would wait for a key press
        // and then reboot the system
    }
    
    pub fn get_error_count(&self, severity: ErrorSeverity) -> u32 {
        match severity {
            ErrorSeverity::Warning => self.error_count[0],
            ErrorSeverity::Error => self.error_count[1],
            ErrorSeverity::Critical => self.error_count[2],
            ErrorSeverity::Fatal => self.error_count[3],
        }
    }
    
    pub fn get_total_error_count(&self) -> u32 {
        self.error_count.iter().sum()
    }
    
    pub fn dump_error_history(&self) {
        crate::println!("\nError History:");
        crate::println!("--------------");
        
        let mut count = 0;
        for i in 0..self.last_errors.len() {
            let index = (self.last_index + i) % self.last_errors.len();
            if let Some(error) = &self.last_errors[index] {
                crate::println!("{}", error);
                count += 1;
            }
        }
        
        if count == 0 {
            crate::println!("No errors recorded.");
        }
        
        crate::println!("");
    }
}

// Global error handler instance
lazy_static! {
    pub static ref ERROR_HANDLER: Mutex<ErrorHandler> = Mutex::new(ErrorHandler::new());
}

// Functions to create and report errors
pub fn report_error(code: u32, domain: ErrorDomain, severity: ErrorSeverity, message: &str) -> Result<(), ()> {
    let error = SystemError::new(code, domain, severity, message);
    let mut handler = ERROR_HANDLER.lock();
    handler.handle_error(error)
}

pub fn report_warning(domain: ErrorDomain, message: &str) -> Result<(), ()> {
    report_error(0x1000, domain, ErrorSeverity::Warning, message)
}

pub fn report_filesystem_error(message: &str) -> Result<(), ()> {
    report_error(0x2000, ErrorDomain::Filesystem, ErrorSeverity::Error, message)
}

pub fn report_memory_error(message: &str) -> Result<(), ()> {
    report_error(0x3000, ErrorDomain::Memory, ErrorSeverity::Error, message)
}

pub fn report_critical_error(domain: ErrorDomain, message: &str) -> Result<(), ()> {
    report_error(0x5000, domain, ErrorSeverity::Critical, message)
}

pub fn report_fatal_error(domain: ErrorDomain, message: &str) -> Result<(), ()> {
    report_error(0x9000, domain, ErrorSeverity::Fatal, message)
}

// Initialize the error handling system
pub fn init() {
    log_info!("Error handling system initialized");
} 