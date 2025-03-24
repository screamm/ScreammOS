// src/ui/file_manager.rs
// File manager for ScreammOS

use crate::vga_buffer::{Color, WRITER};
use crate::simple_fs::{FILESYSTEM, FileType, String as FsString, SimpleString};
use lazy_static::lazy_static;
use spin::Mutex;
use crate::{println, print, format};
use crate::{log_info, log_error};
use crate::error_handler::{report_warning, ErrorDomain};
use crate::ui::text_editor::TEXT_EDITOR;
use alloc::vec::Vec;
use crate::ui::{Rect, BorderStyle, draw_box};
use core::fmt::Write;

// Constants for the file manager UI
const WINDOW_WIDTH: usize = 60;
const WINDOW_HEIGHT: usize = 20;
const LIST_HEIGHT: usize = 16;

pub struct FileManager {
    pub visible: bool,
    current_dir: SimpleString,
    files: Vec<FileEntry>,
    selected_index: usize,
    scroll_offset: usize,
    is_active: bool,
    rect: Rect,
}

#[derive(Clone)]
struct FileEntry {
    name: FsString,
    file_type: FileType,
    size: usize,
}

impl FileManager {
    pub fn new() -> Self {
        let mut current_dir = SimpleString::new();
        current_dir.push_str("/");
        
        FileManager {
            visible: false,
            current_dir,
            files: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            is_active: false,
            rect: Rect::new(0, 0, 80, 24),
        }
    }
    
    // Show the file manager
    pub fn show(&mut self) {
        self.visible = true;
        self.refresh_file_list();
        self.render();
        log_info!("File manager opened");
    }
    
    // Hide the file manager
    pub fn hide(&mut self) {
        self.visible = false;
        // Restore screen
        let mut writer = WRITER.lock();
        writer.set_color(Color::LightGray, Color::Black);
        writer.clear_screen();
        log_info!("File manager closed");
    }
    
    // Update the file list
    fn refresh_file_list(&mut self) {
        self.files.clear();
        
        let fs = FILESYSTEM.lock();
        
        // Add .. directory for going up
        self.files.push(FileEntry {
            name: FsString::from(".."),
            file_type: FileType::Directory,
            size: 0,
        });
        
        // Get all files and directories
        for (file_type, name, size) in fs.list_directory() {
            // Skip special files
            if name == "." || name == ".." {
                continue;
            }
            
            let mut file_name = FsString::new();
            file_name.push_str(name);
            
            self.files.push(FileEntry {
                name: file_name,
                file_type,
                size,
            });
        }
        
        // Reset cursor if list has changed
        if self.selected_index >= self.files.len() && !self.files.is_empty() {
            self.selected_index = self.files.len() - 1;
        }
    }
    
    // Draw the file manager UI
    fn render(&self) {
        let mut writer = WRITER.lock();
        writer.clear_screen();
        
        // Draw title and border
        writer.set_color(Color::Black, Color::LightGray);
        
        // Top border
        writer.set_position(10, 1);
        for _ in 0..WINDOW_WIDTH {
            print!("═");
        }
        
        // Title
        writer.set_position(10 + (WINDOW_WIDTH - 13) / 2, 1);
        print!(" FILE MANAGER ");
        
        // Left and right borders
        for y in 2..2+WINDOW_HEIGHT {
            writer.set_position(10, y);
            print!("║");
            writer.set_position(10 + WINDOW_WIDTH - 1, y);
            print!("║");
        }
        
        // Bottom border
        writer.set_position(10, 2 + WINDOW_HEIGHT);
        for _ in 0..WINDOW_WIDTH {
            print!("═");
        }
        
        // Show current directory
        writer.set_color(Color::White, Color::Blue);
        writer.set_position(12, 3);
        print!(" Current directory: {} ", self.current_dir.as_str());
        
        // Show file list
        writer.set_color(Color::LightGray, Color::Black);
        writer.set_position(12, 5);
        print!(" Name                  Type      Size    ");
        
        writer.set_position(12, 6);
        print!("─────────────────────────────────────────");
        
        // Show files and directories with scrolling
        let visible_items = LIST_HEIGHT.min(self.files.len());
        for i in 0..visible_items {
            let file_index = i + self.scroll_offset;
            if file_index >= self.files.len() {
                break;
            }
            
            let file = &self.files[file_index];
            
            // Highlight selected file
            if file_index == self.selected_index {
                writer.set_color(Color::Black, Color::LightGray);
            } else {
                writer.set_color(Color::LightGray, Color::Black);
            }
            
            writer.set_position(12, 7 + i);
            
            // Filename (max 20 characters)
            let mut display_name = FsString::new();
            display_name.push_str(file.name.as_str());
            if display_name.len() > 20 {
                display_name.clear();
                display_name.push_str(&file.name.as_str()[0..17]);
                display_name.push_str("...");
            }
            
            // File type
            let type_str = match file.file_type {
                FileType::Directory => "<DIR>     ",
                FileType::Regular => "<FILE>    ",
            };
            
            // Size
            let size_str = if file.file_type == FileType::Directory {
                let mut dir_str = FsString::new();
                dir_str.push_str("     -");
                dir_str
            } else {
                format!("{:6}", file.size)
            };
            
            // Write the line
            print!("{:<20} {:9} {:7}", display_name.as_str(), type_str, size_str);
        }
        
        // Show help text
        writer.set_color(Color::Black, Color::LightGray);
        writer.set_position(12, 7 + LIST_HEIGHT + 1);
        print!(" ↑/↓:Navigate  ENTER:Open  ESC:Close ");
    }
    
    // Navigate up in the file list
    pub fn navigate_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            
            // Adjust scroll position if needed
            if self.selected_index < self.scroll_offset {
                self.scroll_offset = self.selected_index;
            }
            
            self.render();
        }
    }
    
    // Navigate down in the file list
    pub fn navigate_down(&mut self) {
        if !self.files.is_empty() && self.selected_index < self.files.len() - 1 {
            self.selected_index += 1;
            
            // Adjust scroll position if needed
            if self.selected_index >= self.scroll_offset + LIST_HEIGHT {
                self.scroll_offset = self.selected_index - LIST_HEIGHT + 1;
            }
            
            self.render();
        }
    }
    
    // Open selected file or directory
    pub fn open_selected(&mut self) {
        if self.files.is_empty() || self.selected_index >= self.files.len() {
            return;
        }
        
        let selected = &self.files[self.selected_index];
        
        match selected.file_type {
            FileType::Directory => {
                // Handle directory navigation
                if selected.name.as_str() == ".." {
                    // Navigate up one level
                    let mut fs = FILESYSTEM.lock();
                    match fs.change_directory("..") {
                        Ok(_) => {
                            log_info!("Navigated to parent directory");
                            // Update current directory
                            let dir_name = fs.get_current_directory();
                            self.current_dir.clear();
                            self.current_dir.push_str(dir_name);
                            
                            // Update file list
                            drop(fs);
                            self.refresh_file_list();
                            self.selected_index = 0;
                            self.scroll_offset = 0;
                            self.render();
                        },
                        Err(e) => {
                            log_error!("Failed to navigate to parent directory: {}", e);
                            report_warning(ErrorDomain::Filesystem, "Failed to navigate to parent directory").ok();
                        }
                    }
                } else {
                    // Navigate to selected directory
                    let dir_name = selected.name.as_str();
                    let mut fs = FILESYSTEM.lock();
                    match fs.change_directory(dir_name) {
                        Ok(_) => {
                            log_info!("Navigated to directory: {}", dir_name);
                            // Update current directory
                            let dir_name = fs.get_current_directory();
                            self.current_dir.clear();
                            self.current_dir.push_str(dir_name);
                            
                            // Update file list
                            drop(fs);
                            self.refresh_file_list();
                            self.selected_index = 0;
                            self.scroll_offset = 0;
                            self.render();
                        },
                        Err(e) => {
                            log_error!("Failed to navigate to directory {}: {}", dir_name, e);
                            report_warning(ErrorDomain::Filesystem, &format!("Failed to navigate to directory {}", dir_name)).ok();
                        }
                    }
                }
            },
            FileType::Regular => {
                // Open file in text editor
                let file_name = selected.name.as_str();
                if let Some(mut text_editor) = TEXT_EDITOR.try_lock() {
                    self.hide(); // Hide file manager first
                    
                    if text_editor.open_file(file_name) {
                        text_editor.show();
                        log_info!("Opened file in text editor: {}", file_name);
                    } else {
                        // Show error message
                        log_error!("Failed to open file in text editor: {}", file_name);
                        report_warning(ErrorDomain::UserInterface, &format!("Failed to open file: {}", file_name)).ok();
                    }
                }
            },
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    pub fn draw(&self, writer: &mut Writer) {
        draw_box(self.rect, BorderStyle::Double, Some("File Manager"));
        
        // Draw current directory
        print!(" Current directory: {} ", self.current_dir.as_str());
        
        // Draw file list
        let mut fs = FILESYSTEM.lock();
        let file_count = fs.get_file_count();
        
        for i in 0..file_count {
            let filename = fs.get_filename(i);
            let file_type = fs.get_file_type(i);
            let size = fs.get_file_size(i);
            
            let type_str = match file_type {
                FileType::File => "File",
                FileType::Directory => "Dir",
                FileType::Symlink => "Link",
            };
            
            let size_str = if size < 1024 {
                format!("{} B", size)
            } else if size < 1024 * 1024 {
                format!("{} KB", size / 1024)
            } else {
                format!("{} MB", size / (1024 * 1024))
            };
            
            let display_name = if i == self.selected_index {
                format!("> {}", filename)
            } else {
                format!("  {}", filename)
            };
            
            print!("{:<20} {:9} {:7}", display_name.as_str(), type_str, size_str);
        }
    }

    pub fn handle_key(&mut self, key: char) {
        match key {
            'j' => {
                let mut fs = FILESYSTEM.lock();
                if self.selected_index < fs.get_file_count() - 1 {
                    self.selected_index += 1;
                }
            }
            'k' => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            'l' => {
                let mut fs = FILESYSTEM.lock();
                if self.selected_index < fs.get_file_count() {
                    let filename = fs.get_filename(self.selected_index);
                    if fs.get_file_type(self.selected_index) == FileType::Directory {
                        self.current_dir.push_str(filename);
                        self.current_dir.push_str("/");
                        self.selected_index = 0;
                    }
                }
            }
            'h' => {
                if self.current_dir.len() > 1 {
                    let mut new_dir = SimpleString::new();
                    let dir_str = self.current_dir.as_str();
                    let parent = dir_str.rsplit_once('/').unwrap_or(("", "")).0;
                    new_dir.push_str(parent);
                    new_dir.push_str("/");
                    self.current_dir = new_dir;
                    self.selected_index = 0;
                }
            }
            _ => {}
        }
    }
}

// Global instances
lazy_static! {
    pub static ref FILE_MANAGER: Mutex<FileManager> = Mutex::new(FileManager::new());
} 