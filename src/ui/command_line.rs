use crate::{log_info, log_error, format};
use crate::error_handler::{report_error, report_warning, ErrorDomain, ErrorSeverity};
use crate::simple_fs::{FILESYSTEM, SimpleString, SimpleFileSystem, FileType};
use crate::vga_buffer;
use crate::ui::text_editor::TEXT_EDITOR;
use crate::vga_buffer::{WRITER, ColorCode, Color};
use crate::keyboard::{KeyEvent, KeyCode, KeyState};
use crate::ui::text_editor::TextEditor;
use crate::ui::file_manager::FileManager;
use crate::ui::retro_commands::handle_retro_command;
use alloc::vec::Vec;
use crate::string_ext::{StringExt, StringSliceExt};
use core::fmt::Write;
use crate::error_handler::{ERROR_HANDLER, ErrorSeverity};

// Constants for command handling
const MAX_COMMAND_HISTORY: usize = 10;
const MAX_HISTORY: usize = 50;
const MAX_TAB_COMPLETIONS: usize = 10;

// Command structure for more organized command handling
struct Command {
    name: &'static str,
    description: &'static str,
    usage: &'static str,
    handler: fn(&mut CommandLine, &[&str]) -> Result<(), &'static str>,
}

/// Command line interface for ScreammOS
pub struct CommandLine {
    input: SimpleString,
    history: [SimpleString; MAX_HISTORY],
    history_index: usize,
    history_count: usize,
    last_tab_command: SimpleString,
    tab_completions: [SimpleString; MAX_TAB_COMPLETIONS],
    tab_completion_index: usize,
    tab_completion_count: usize,
    cursor_position: usize,
    text_editor: Option<TextEditor>,
    file_manager: Option<FileManager>,
}

impl CommandLine {
    pub fn new() -> Self {
        CommandLine {
            input: SimpleString::new(),
            history: [SimpleString::new(); MAX_HISTORY],
            history_index: 0,
            history_count: 0,
            last_tab_command: SimpleString::new(),
            tab_completions: [SimpleString::new(); MAX_TAB_COMPLETIONS],
            tab_completion_index: 0,
            tab_completion_count: 0,
            cursor_position: 0,
            text_editor: None,
            file_manager: None,
        }
    }
    
    pub fn handle_input(&mut self, ch: char) {
        match ch {
            // ... existing code ...
            
            // Up arrow for command history
            '\u{1b}' => {
                // Check if this is an escape sequence for arrow keys
                if self.input.len() >= 2 && 
                   self.input.as_str().chars().nth(self.input.len() - 2) == Some('[') {
                    match self.input.as_str().chars().last() {
                        Some('A') => { // Up arrow
                            self.navigate_history_up();
                        },
                        Some('B') => { // Down arrow
                            self.navigate_history_down();
                        },
                        _ => {}
                    }
                }
            },
            
            // Tab for command completion
            '\t' => {
                self.command_completion();
            },
            
            _ => {
                // ... existing code ...
            }
        }
    }
    
    pub fn navigate_history_up(&mut self) {
        if self.history_index < self.history_count {
            self.history_index += 1;
            self.input = self.history[self.history_count - self.history_index];
            self.cursor_position = self.input.len();
        }
    }
    
    pub fn navigate_history_down(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            if self.history_index == 0 {
                self.input = SimpleString::new();
            } else {
                self.input = self.history[self.history_count - self.history_index];
            }
            self.cursor_position = self.input.len();
        }
    }
    
    pub fn command_completion(&mut self) {
        if self.input.len() == 0 {
            return;
        }

        if self.last_tab_command.len() == 0 {
            self.last_tab_command = self.input;
            self.clear_tab_completions();
            
            // Hitta alla kommandon som börjar med input
            let input_str = self.input.as_str();
            let fs = FILESYSTEM.lock();
            for i in 0..fs.get_file_count() {
                let filename = fs.get_filename(i);
                if filename.starts_with(input_str) {
                    self.add_tab_completion(filename);
                }
            }
        }

        if self.tab_completion_count > 0 {
            self.input = self.tab_completions[self.tab_completion_index];
            self.cursor_position = self.input.len();
            self.tab_completion_index = (self.tab_completion_index + 1) % self.tab_completion_count;
        }
    }
    
    pub fn process_command(&mut self) {
        self.println("");
        
        let command = self.input.as_str().trim();
        
        // Add command to history
        self.add_to_history(command);
        
        // Parse the command and arguments
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        if parts.is_empty() {
            self.input.clear();
            return;
        }
        
        let cmd = parts[0];
        let args = &parts[1..];
        
        // Log the command
        log_info!("Command executed: {}", command);
        
        // Find and execute the command
        let mut found = false;
        
        for command in COMMANDS.iter() {
            if command.name == cmd {
                found = true;
                
                match (command.handler)(self, args) {
                    Ok(_) => {},
                    Err(msg) => {
                        self.println(&format!("Error: {}", msg));
                        report_warning(ErrorDomain::UserInterface, &format!("Command error: {}", msg)).ok();
                    }
                }
                
                break;
            }
        }
        
        if !found {
            self.println(&format!("Unknown command: {}", cmd));
            self.println("Type 'help' for a list of commands.");
            report_warning(ErrorDomain::UserInterface, &format!("Unknown command: {}", cmd)).ok();
        }
        
        self.input.clear();
    }
    
    // Command handlers
    fn cmd_help(&mut self, args: &[&str]) -> Result<(), &'static str> {
        if args.is_empty() {
            self.println("Available commands:");
            self.println("------------------");
            
            for cmd in COMMANDS.iter() {
                self.println(&format!("{:10} - {}", cmd.name, cmd.description));
            }
            
            self.println("");
            self.println("Type 'help <command>' for more information about a specific command.");
        } else {
            let cmd_name = args[0];
            let mut found = false;
            
            for cmd in COMMANDS.iter() {
                if cmd.name == cmd_name {
                    found = true;
                    self.println(&format!("Command: {}", cmd.name));
                    self.println(&format!("Description: {}", cmd.description));
                    self.println(&format!("Usage: {}", cmd.usage));
                    break;
                }
            }
            
            if !found {
                self.println(&format!("No help available for '{}'", cmd_name));
                return Err("Unknown command");
            }
        }
        
        Ok(())
    }
    
    fn cmd_clear(&mut self, _args: &[&str]) -> Result<(), &'static str> {
        self.clear();
        Ok(())
    }
    
    fn cmd_ls(&mut self, args: &[&str]) -> Result<(), &'static str> {
        let mut fs = FILESYSTEM.lock();
        
        let show_all = args.contains(&"-a");
        let show_long = args.contains(&"-l");
        
        let mut count = 0;
        
        // Print header for long format
        if show_long {
            self.println("Type  Size  Modified  Name");
            self.println("----  ----  --------  ----");
        }
        
        // List files in current directory
        for i in 0..fs.get_file_count() {
            let filename = fs.get_filename(i);
            let file_type = fs.get_file_type(i);
            
            // Skip hidden files unless -a flag is provided
            if !show_all && filename.starts_with(".") {
                continue;
            }
            
            if show_long {
                let file_size = fs.get_file_size(i);
                let type_indicator = if file_type == FileType::Directory { "DIR" } else { "FILE" };
                self.println(&format!("{:4}  {:4}           {}", type_indicator, file_size, filename));
            } else {
                let type_indicator = if file_type == FileType::Directory { "/" } else { "" };
                self.print(&format!("{}{} ", filename, type_indicator));
                
                count += 1;
                if count % 5 == 0 {
                    self.println("");
                }
            }
        }
        
        if !show_long && count % 5 != 0 {
            self.println("");
        }
        
        // Print total count
        self.println(&format!("Total: {} files", fs.get_file_count()));
        
        Ok(())
    }
    
    fn cmd_cat(&mut self, args: &[&str]) -> Result<(), &'static str> {
        if args.is_empty() {
            return Err("No filename specified");
        }
        
        let filename = args[0];
        
        let fs = FILESYSTEM.lock();
        
        if let Some(content) = fs.read_file(filename) {
            self.println(content);
            Ok(())
        } else {
            Err("File not found")
        }
    }
    
    fn cmd_edit(&mut self, args: &[&str]) -> Result<(), &'static str> {
        if args.is_empty() {
            return Err("No filename specified");
        }
        
        let filename = args[0];
        
        let mut editor = TEXT_EDITOR.try_lock().ok_or("Text editor is busy")?;
        
        editor.open_file(filename);
        editor.set_active(true);
        
        Ok(())
    }
    
    fn cmd_mkdir(&mut self, args: &[&str]) -> Result<(), &'static str> {
        if args.is_empty() {
            return Err("No directory name specified");
        }
        
        let dirname = args[0];
        
        let mut fs = FILESYSTEM.lock();
        
        match fs.create_directory(dirname) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
    
    fn cmd_touch(&mut self, args: &[&str]) -> Result<(), &'static str> {
        if args.is_empty() {
            return Err("No filename specified");
        }
        
        let filename = args[0];
        
        let mut fs = FILESYSTEM.lock();
        
        // Check if the file already exists
        if fs.find_file(filename).is_some() {
            // File exists, just update timestamp (not implemented in SimpleFs yet)
            return Ok(());
        }
        
        // Create empty file
        match fs.create_file(filename, "") {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
    
    fn cmd_rm(&mut self, args: &[&str]) -> Result<(), &'static str> {
        if args.is_empty() {
            return Err("No filename specified");
        }
        
        let filename = args[0];
        
        let mut fs = FILESYSTEM.lock();
        
        match fs.delete_file(filename) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
    
    fn cmd_echo(&mut self, args: &[&str]) -> Result<(), &'static str> {
        let text = args.join(" ");
        self.println(&text);
        Ok(())
    }
    
    fn cmd_write(&mut self, args: &[&str]) -> Result<(), &'static str> {
        if args.len() < 2 {
            return Err("Usage: write <filename> <text>");
        }
        
        let filename = args[0];
        let content = args[1..].join(" ");
        
        let mut fs = FILESYSTEM.lock();
        
        // If file doesn't exist, create it
        if fs.find_file(filename).is_none() {
            match fs.create_file(filename, &content) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        } else {
            // File exists, update its content
            if fs.write_file(filename, &content) {
                Ok(())
            } else {
                Err("Failed to write file")
            }
        }
    }
    
    fn cmd_theme(&mut self, args: &[&str]) -> Result<(), &'static str> {
        if args.is_empty() {
            // Show current theme
            let current_theme = vga_buffer::get_current_theme();
            self.println(&format!("Current theme: {}", vga_buffer::get_theme_name(current_theme)));
            
            // List available themes
            self.println("Available themes:");
            for i in 0..4 {
                self.println(&format!("  {} - {}", i, vga_buffer::get_theme_name(i)));
            }
            
            return Ok(());
        }
        
        // Try to parse theme number
        if let Ok(theme_num) = args[0].parse::<u8>() {
            if theme_num > 3 {
                return Err("Invalid theme number");
            }
            
            let theme_name = vga_buffer::get_theme_name(theme_num);
            self.println(&format!("Setting theme to: {}", theme_name));
            
            vga_buffer::set_theme(theme_num);
            
            Ok(())
        } else {
            Err("Invalid theme number")
        }
    }
    
    fn cmd_test(&mut self, _args: &[&str]) -> Result<(), &'static str> {
        self.println("Running system self-tests...");
        
        crate::run_self_tests();
        
        Ok(())
    }
    
    fn cmd_errors(&mut self, _args: &[&str]) -> Result<(), &'static str> {
        let handler = ERROR_HANDLER.lock();
        
        self.println("Error statistics:");
        self.println("----------------");
        self.println(&format!("Warnings:  {}", handler.get_error_count(ErrorSeverity::Warning)));
        self.println(&format!("Errors:    {}", handler.get_error_count(ErrorSeverity::Error)));
        self.println(&format!("Critical:  {}", handler.get_error_count(ErrorSeverity::Critical)));
        self.println(&format!("Fatal:     {}", handler.get_error_count(ErrorSeverity::Fatal)));
        self.println(&format!("Total:     {}", handler.get_total_error_count()));
        
        // Dump error history
        handler.dump_error_history();
        
        Ok(())
    }
    
    fn cmd_restart(&mut self, _args: &[&str]) -> Result<(), &'static str> {
        self.println("Restarting system...");
        self.println("This is a simulated restart (not implemented)");
        
        // In a real OS, we'd do something like this:
        // unsafe { x86_64::instructions::port::outb(0x64, 0xFE); }
        
        self.clear();
        self.println("System restarted");
        
        // Re-initialize components
        crate::init();
        
        Ok(())
    }

    fn println(&mut self, text: &str) {
        use crate::println;
        println!("{}", text);
    }

    fn print(&mut self, text: &str) {
        use crate::print;
        print!("{}", text);
    }

    fn clear(&mut self) {
        self.input.clear();
        self.tab_completion_count = 0;
        self.tab_completion_index = 0;
        self.last_tab_command.clear();
    }

    pub fn clear_tab_completions(&mut self) {
        self.tab_completion_count = 0;
        self.tab_completion_index = 0;
    }

    pub fn add_tab_completion(&mut self, completion: &str) {
        if self.tab_completion_count < MAX_TAB_COMPLETIONS {
            self.tab_completions[self.tab_completion_count] = SimpleString::new();
            self.tab_completions[self.tab_completion_count].push_str(completion);
            self.tab_completion_count += 1;
        }
    }

    fn add_to_history(&mut self, command: &str) {
        // Don't add empty commands or duplicates of the last command
        if command.trim().is_empty() || 
           (self.history_count > 0 && 
            command == self.history[self.history_count - 1].as_str()) {
            return;
        }
        
        let mut cmd = SimpleString::new();
        cmd.push_str(command);
        
        self.history[self.history_count % MAX_HISTORY] = cmd;
        self.history_count = self.history_count.saturating_add(1);
    }
}

// Define all available commands
static COMMANDS: &[Command] = &[
    Command {
        name: "help",
        description: "Display help information",
        usage: "help [command]",
        handler: CommandLine::cmd_help,
    },
    Command {
        name: "clear",
        description: "Clear the screen",
        usage: "clear",
        handler: CommandLine::cmd_clear,
    },
    Command {
        name: "ls",
        description: "List files and directories",
        usage: "ls [-a] [-l]",
        handler: CommandLine::cmd_ls,
    },
    Command {
        name: "cat",
        description: "Display file contents",
        usage: "cat <filename>",
        handler: CommandLine::cmd_cat,
    },
    Command {
        name: "edit",
        description: "Edit a file",
        usage: "edit <filename>",
        handler: CommandLine::cmd_edit,
    },
    Command {
        name: "mkdir",
        description: "Create a directory",
        usage: "mkdir <dirname>",
        handler: CommandLine::cmd_mkdir,
    },
    Command {
        name: "touch",
        description: "Create an empty file",
        usage: "touch <filename>",
        handler: CommandLine::cmd_touch,
    },
    Command {
        name: "rm",
        description: "Remove a file",
        usage: "rm <filename>",
        handler: CommandLine::cmd_rm,
    },
    Command {
        name: "echo",
        description: "Display a message",
        usage: "echo <text>",
        handler: CommandLine::cmd_echo,
    },
    Command {
        name: "write",
        description: "Write text to a file",
        usage: "write <filename> <text>",
        handler: CommandLine::cmd_write,
    },
    Command {
        name: "theme",
        description: "Change the display theme",
        usage: "theme [number]",
        handler: CommandLine::cmd_theme,
    },
    Command {
        name: "test",
        description: "Run system self-tests",
        usage: "test",
        handler: CommandLine::cmd_test,
    },
    Command {
        name: "errors",
        description: "Display system error information",
        usage: "errors",
        handler: CommandLine::cmd_errors,
    },
    Command {
        name: "restart",
        description: "Restart the system",
        usage: "restart",
        handler: CommandLine::cmd_restart,
    },
]; 