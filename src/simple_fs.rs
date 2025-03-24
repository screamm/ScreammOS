// src/simple_fs.rs
// A simple RAM-based file system that works without the alloc crate

use core::fmt;
use core::str;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::println;
use crate::{log_error, log_warn, log_info};
use crate::error_handler::{report_filesystem_error, report_warning};
use alloc::string::String;
use alloc::vec::Vec;
use crate::vga_buffer::Color;
use crate::error_handler::{report_error, report_warning, ErrorDomain, ErrorSeverity};

// File system constants
pub const MAX_FILES: usize = 100;
const MAX_FILENAME_LENGTH: usize = 32;
const MAX_FILE_SIZE: usize = 1024;  // 1KB per file
const MAX_CONTENT_LENGTH: usize = MAX_FILE_SIZE - MAX_FILENAME_LENGTH;

// File type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    #[default]
    Regular,
    Directory,
    File,
    Symlink,
}

// File entry
#[derive(Debug, Clone, Copy)]
pub struct FileEntry {
    name: [u8; MAX_FILENAME_LENGTH],
    name_len: usize,
    file_type: FileType,
    content: [u8; MAX_FILE_SIZE],
    content_len: usize,
    is_used: bool,
    size: usize,
}

impl FileEntry {
    pub fn new() -> Self {
        FileEntry {
            name: [0; MAX_FILENAME_LENGTH],
            name_len: 0,
            file_type: FileType::Regular,
            content: [0; MAX_FILE_SIZE],
            content_len: 0,
            is_used: false,
            size: 0,
        }
    }

    pub fn set_name(&mut self, name: &str) -> bool {
        if name.len() > MAX_FILENAME_LENGTH {
            return false;
        }

        self.name_len = 0;
        for (i, &byte) in name.as_bytes().iter().enumerate() {
            if i >= MAX_FILENAME_LENGTH {
                break;
            }
            self.name[i] = byte;
            self.name_len += 1;
        }
        true
    }

    pub fn get_name(&self) -> &str {
        // Convert name buffer to a string
        match str::from_utf8(&self.name[0..self.name_len]) {
            Ok(s) => s,
            Err(_) => "???",
        }
    }

    pub fn set_content(&mut self, content: &str) -> bool {
        if content.len() > MAX_CONTENT_LENGTH {
            return false;
        }

        self.content_len = 0;
        for (i, &byte) in content.as_bytes().iter().enumerate() {
            if i >= MAX_CONTENT_LENGTH {
                break;
            }
            self.content[i] = byte;
            self.content_len += 1;
        }
        true
    }

    pub fn get_content(&self) -> &str {
        // Convert content buffer to a string
        match str::from_utf8(&self.content[0..self.content_len]) {
            Ok(s) => s,
            Err(_) => "???",
        }
    }

    pub fn set_type(&mut self, file_type: FileType) {
        self.file_type = file_type;
    }

    pub fn get_type(&self) -> FileType {
        self.file_type
    }

    pub fn get_size(&self) -> usize {
        self.content_len
    }
}

// File system structure
pub struct SimpleFileSystem {
    files: [FileEntry; MAX_FILES],
    current_dir: usize, // Index to the current directory, 0 = root
    file_count: usize,
}

impl SimpleFileSystem {
    pub fn new() -> Self {
        let mut fs = SimpleFileSystem {
            files: [FileEntry::new(); MAX_FILES],
            current_dir: 0,
            file_count: 0,
        };

        // Create root directory
        let mut root = FileEntry::new();
        root.set_name("/");
        root.set_type(FileType::Directory);
        root.is_used = true;
        fs.files[0] = root;

        // Create some standard files and directories
        let _ = fs.create_file("readme.txt", "Welcome to ScreammOS! A simple filesystem.");
        let _ = fs.create_directory("system");
        let _ = fs.create_directory("home");
        let _ = fs.create_directory("tmp");

        fs
    }

    /// Hitta en fil med det givna namnet
    pub fn find_file(&self, name: &str) -> Option<usize> {
        for i in 0..self.file_count {
            if self.files[i].is_used && self.files[i].get_name() == name {
                return Some(i);
            }
        }
        None
    }

    // Find a free file entry
    fn find_free_entry(&self) -> Option<usize> {
        for i in 0..MAX_FILES {
            if !self.files[i].is_used {
                return Some(i);
            }
        }
        None
    }

    // Create a file
    pub fn create_file(&mut self, name: &str, content: &str) -> Result<bool, &'static str> {
        if self.file_count >= MAX_FILES {
            let error_msg = "Filesystem is full";
            log_error!("{}", error_msg);
            report_filesystem_error(error_msg).ok();
            return Err(error_msg);
        }

        if name.len() >= MAX_FILENAME_LENGTH {
            let error_msg = "Filename too long";
            log_error!("{}", error_msg);
            report_filesystem_error(error_msg).ok();
            return Err(error_msg);
        }

        if content.len() >= MAX_FILE_SIZE {
            let error_msg = "File content too large";
            log_error!("{}", error_msg);
            report_filesystem_error(error_msg).ok();
            return Err(error_msg);
        }

        let mut file = FileEntry::new();
        
        // Copy filename
        for (i, &byte) in name.as_bytes().iter().enumerate() {
            file.name[i] = byte;
        }
        
        // Copy content
        for (i, &byte) in content.as_bytes().iter().enumerate() {
            file.content[i] = byte;
        }
        
        file.size = content.len();
        file.file_type = FileType::Regular;
        
        self.files[self.file_count] = file;
        self.file_count += 1;
        
        log_info!("File created: {}", name);
        Ok(true)
    }

    // Create a directory
    pub fn create_directory(&mut self, name: &str) -> bool {
        if self.find_file(name).is_some() {
            return false;
        }

        if let Some(index) = self.find_free_entry() {
            self.files[index].set_name(name);
            self.files[index].set_type(FileType::Directory);
            self.files[index].is_used = true;
            self.file_count += 1;
            true
        } else {
            false
        }
    }

    // Read a file
    pub fn read_file(&self, name: &str) -> Option<&str> {
        if let Some(index) = self.find_file(name) {
            if self.files[index].get_type() == FileType::File {
                Some(self.files[index].get_content())
            } else {
                None
            }
        } else {
            None
        }
    }

    // List files in current directory
    pub fn list_directory(&self) -> FileList {
        FileList {
            filesystem: self,
            index: 0,
        }
    }

    // Change directory
    pub fn change_directory(&mut self, path: &str) -> Result<(), &'static str> {
        if path == "/" {
            self.current_dir = 0;
            return Ok(());
        }
        
        let index = self.find_file(path)
            .ok_or("Directory not found")?;
            
        if self.files[index].get_type() != FileType::Directory {
            return Err("Not a directory");
        }
        
        self.current_dir = index;
        Ok(())
    }
    
    // Get current directory
    pub fn get_current_directory(&self) -> &str {
        self.files[self.current_dir].get_name()
    }

    pub fn write_file(&mut self, name: &str, content: &str) -> bool {
        if let Some(index) = self.find_file(name) {
            if self.files[index].get_type() == FileType::File {
                self.files[index].set_content(content);
                true
            } else {
                false
            }
        } else {
            if let Some(index) = self.find_free_entry() {
                self.files[index].set_name(name);
                self.files[index].set_type(FileType::File);
                self.files[index].set_content(content);
                self.files[index].is_used = true;
                self.file_count += 1;
                true
            } else {
                false
            }
        }
    }
    
    pub fn delete_file(&mut self, name: &str) -> bool {
        if let Some(index) = self.find_file(name) {
            self.files[index].is_used = false;
            self.file_count -= 1;
            true
        } else {
            false
        }
    }

    pub fn get_file_count(&self) -> usize {
        self.file_count
    }

    pub fn get_filename(&self, index: usize) -> &str {
        if index >= self.files.len() {
            return "";
        }
        unsafe { core::str::from_utf8_unchecked(&self.files[index].name[..self.files[index].name_len]) }
    }

    pub fn get_file_type(&self, index: usize) -> FileType {
        if index >= self.file_count {
            return FileType::Regular;
        }
        self.files[index].file_type
    }

    pub fn get_file_size(&self, index: usize) -> usize {
        if index >= self.file_count {
            return 0;
        }
        self.files[index].size
    }
}

// File listing iterator
pub struct FileList<'a> {
    filesystem: &'a SimpleFileSystem,
    index: usize,
}

impl<'a> Iterator for FileList<'a> {
    type Item = (FileType, &'a str, usize);
    
    fn next(&mut self) -> Option<Self::Item> {
        while self.index < MAX_FILES {
            let current = self.index;
            self.index += 1;
            
            if self.filesystem.files[current].is_used {
                return Some((
                    self.filesystem.files[current].get_type(),
                    self.filesystem.files[current].get_name(),
                    self.filesystem.files[current].get_size(),
                ));
            }
        }
        
        None
    }
}

// A simple string implementation to handle text without depending on alloc
#[derive(Debug)]
pub struct SimpleString {
    buffer: [u8; 256],
    len: usize,
}

impl SimpleString {
    pub fn new() -> Self {
        SimpleString {
            buffer: [0; 256],
            len: 0,
        }
    }

    pub fn push(&mut self, c: char) {
        if self.len < 255 {
            self.buffer[self.len] = c as u8;
            self.len += 1;
        }
    }

    pub fn push_str(&mut self, s: &str) {
        for c in s.chars() {
            self.push(c);
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
        for byte in self.buffer.iter_mut() {
            *byte = 0;
        }
    }

    pub fn as_str(&self) -> &str {
        // Konvertera bytes[0..len] till en str slice
        // Detta är säkert eftersom vi bara lägger till giltiga UTF-8 tecken i push() och push_str()
        unsafe {
            core::str::from_utf8_unchecked(&self.buffer[0..self.len])
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn pop(&mut self) -> Option<char> {
        if self.len > 0 {
            self.len -= 1;
            Some(self.buffer[self.len] as char)
        } else {
            None
        }
    }
}

impl core::fmt::Write for SimpleString {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.push_str(s);
        Ok(())
    }
}

impl Default for SimpleString {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SimpleString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = unsafe { core::str::from_utf8_unchecked(&self.buffer[..self.len]) };
        write!(f, "{}", s)
    }
}

// Global instance of the file system
lazy_static! {
    pub static ref FILESYSTEM: Mutex<SimpleFileSystem> = Mutex::new(SimpleFileSystem::new());
}

// Initialization of the file system
pub fn init() {
    println!("SimpleFS: file system initialized");
}

// Re-export necessary type names with simpler names
pub use SimpleString as String;

pub type FsString = String; 