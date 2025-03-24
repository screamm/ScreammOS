// src/fs.rs
// A simple RAM-based filesystem for ScreammOS

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::boxed::Box;
use core::fmt;
use crate::println;
use spin::Mutex;
use lazy_static::lazy_static;
use core::fmt::Write;

// File entry types
#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    File,
    Directory,
}

// File metadata
#[derive(Debug, Clone)]
pub struct Metadata {
    pub file_type: FileType,
    pub size: usize,
    pub created: u64, // Simplified timestamp (just a counter)
    pub modified: u64,
}

// File content
#[derive(Debug, Clone)]
pub struct FileContent {
    pub data: Vec<u8>,
}

impl FileContent {
    pub fn new() -> Self {
        FileContent {
            data: Vec::new(),
        }
    }

    pub fn from_string(content: &str) -> Self {
        let mut data = Vec::new();
        for byte in content.bytes() {
            data.push(byte);
        }
        FileContent { data }
    }

    pub fn as_string(&self) -> String {
        let mut result = String::new();
        for &byte in &self.data {
            if let Some(c) = core::char::from_u32(byte as u32) {
                result.push(c);
            }
        }
        result
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

// File system entry (file or directory)
#[derive(Debug, Clone)]
pub struct FSEntry {
    pub name: String,
    pub metadata: Metadata,
    pub content: Option<FileContent>,
    pub children: Option<BTreeMap<String, FSEntry>>,
}

impl FSEntry {
    pub fn new_file(name: &str, content: Option<FileContent>) -> Self {
        let content = content.unwrap_or_else(FileContent::new);
        FSEntry {
            name: String::from(name),
            metadata: Metadata {
                file_type: FileType::File,
                size: content.len(),
                created: 1, // Simple counter for now
                modified: 1,
            },
            content: Some(content),
            children: None,
        }
    }

    pub fn new_directory(name: &str) -> Self {
        FSEntry {
            name: String::from(name),
            metadata: Metadata {
                file_type: FileType::Directory,
                size: 0,
                created: 1,
                modified: 1,
            },
            content: None,
            children: Some(BTreeMap::new()),
        }
    }

    pub fn is_directory(&self) -> bool {
        self.metadata.file_type == FileType::Directory
    }

    pub fn is_file(&self) -> bool {
        self.metadata.file_type == FileType::File
    }
}

// Path handling utilities
#[derive(Clone)]
pub struct Path {
    components: Vec<String>,
}

impl Path {
    pub fn new(path: &str) -> Self {
        // Handle both Unix-style and DOS-style paths
        let mut path_modified = String::new();
        
        for c in path.chars() {
            if c == '\\' {
                path_modified.push('/');
            } else {
                path_modified.push(c);
            }
        }
        
        let mut components = Vec::new();
        for part in path_modified.split('/') {
            if !part.is_empty() {
                components.push(String::from(part));
            }
        }

        Path { components }
    }

    pub fn is_absolute(&self) -> bool {
        // For simplicity, we don't support absolute paths yet
        false
    }

    pub fn components(&self) -> &[String] {
        &self.components
    }

    pub fn join(&self, other: &Path) -> Path {
        let mut result = self.components.clone();
        for component in &other.components {
            if component == ".." {
                if !result.is_empty() {
                    result.pop();
                }
            } else if component != "." {
                result.push(component.clone());
            }
        }
        Path { components: result }
    }

    pub fn parent(&self) -> Option<Path> {
        if self.components.is_empty() {
            None
        } else {
            let mut parent_components = self.components.clone();
            parent_components.pop();
            Some(Path { components: parent_components })
        }
    }

    pub fn file_name(&self) -> Option<&str> {
        self.components.last().map(|s| s.as_str())
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.components.is_empty() {
            write!(f, "/")
        } else {
            write!(f, "/{}", join_strings(&self.components, "/"))
        }
    }
}

// Helper function to join strings with a separator
fn join_strings(strings: &[String], separator: &str) -> String {
    let mut result = String::new();
    for (i, s) in strings.iter().enumerate() {
        if i > 0 {
            result.push_str(separator);
        }
        result.push_str(s);
    }
    result
}

// Filesystem structure
pub struct FileSystem {
    root: FSEntry,
    current_path: Path,
}

impl FileSystem {
    pub fn new() -> Self {
        let mut root = FSEntry::new_directory("");
        
        // Create standard directories
        let mut sys_dir = FSEntry::new_directory("system");
        let mut home_dir = FSEntry::new_directory("home");
        let mut temp_dir = FSEntry::new_directory("temp");
        
        // Add a README file to the root
        let readme_content = "Welcome to ScreammOS Filesystem!\n\nThis is a simple RAM-based filesystem.\nAll files and directories will be lost when the system is turned off.";
        let readme = FSEntry::new_file("README.TXT", Some(FileContent::from_string(readme_content)));
        
        // Add a help file to the system directory
        let help_content = "Available commands:\n\ndir - List directory contents\ntype - Display file contents\ncd - Change directory\nmkdir - Create directory\ndel - Delete file\ncopy - Copy file\nren - Rename file";
        let help_file = FSEntry::new_file("HELP.TXT", Some(FileContent::from_string(help_content)));
        
        if let Some(ref mut children) = sys_dir.children {
            children.insert(help_file.name.clone(), help_file);
        }
        
        if let Some(ref mut children) = root.children {
            children.insert(readme.name.clone(), readme);
            children.insert(sys_dir.name.clone(), sys_dir);
            children.insert(home_dir.name.clone(), home_dir);
            children.insert(temp_dir.name.clone(), temp_dir);
        }
        
        FileSystem {
            root,
            current_path: Path::new(""),
        }
    }
    
    // Get the current working directory path
    pub fn get_current_path(&self) -> String {
        let mut path_str = String::new();
        let _ = write!(&mut path_str, "{}", self.current_path);
        path_str
    }
    
    // Change directory
    pub fn change_directory(&mut self, path_str: &str) -> Result<(), &'static str> {
        let path = Path::new(path_str);
        
        if path_str == "/" || path_str == "\\" {
            self.current_path = Path::new("");
            return Ok(());
        }
        
        if path_str == ".." {
            if let Some(parent) = self.current_path.parent() {
                self.current_path = parent;
                return Ok(());
            } else {
                return Ok(());  // Already at root
            }
        }
        
        // Navigate to the target directory
        let target_path = if path.is_absolute() {
            path
        } else {
            self.current_path.join(&path)
        };
        
        // Check if the target exists and is a directory
        let entry = self.get_entry(&target_path)?;
        
        if !entry.is_directory() {
            return Err("Not a directory");
        }
        
        self.current_path = target_path;
        Ok(())
    }
    
    // List directory contents
    pub fn list_directory(&self, path_str: Option<&str>) -> Result<Vec<FSEntry>, &'static str> {
        let path = if let Some(p) = path_str {
            if p.is_empty() {
                self.current_path.clone()
            } else {
                Path::new(p)
            }
        } else {
            self.current_path.clone()
        };
        
        let dir_entry = self.get_entry(&path)?;
        
        if !dir_entry.is_directory() {
            return Err("Not a directory");
        }
        
        let mut entries = Vec::new();
        
        if let Some(ref children) = dir_entry.children {
            for entry in children.values() {
                entries.push(entry.clone());
            }
        }
        
        Ok(entries)
    }
    
    // Create a new directory
    pub fn create_directory(&mut self, path_str: &str) -> Result<(), &'static str> {
        let path = Path::new(path_str);
        
        // Get parent directory path and new directory name
        let parent_path = if path.components().len() > 1 {
            let mut parent = self.current_path.clone();
            for i in 0..path.components().len() - 1 {
                parent = parent.join(&Path::new(path.components()[i].as_str()));
            }
            parent
        } else {
            self.current_path.clone()
        };
        
        let dir_name = path.file_name().ok_or("Invalid directory name")?;
        
        // Check if a file/directory with this name already exists
        let parent_entry = self.get_entry_mut(&parent_path)?;
        
        if !parent_entry.is_directory() {
            return Err("Parent is not a directory");
        }
        
        if let Some(ref mut children) = parent_entry.children {
            if children.contains_key(dir_name) {
                return Err("Entry already exists");
            }
            
            let new_dir = FSEntry::new_directory(dir_name);
            children.insert(String::from(dir_name), new_dir);
            
            Ok(())
        } else {
            Err("Parent directory error")
        }
    }
    
    // Create a new file with content
    pub fn create_file(&mut self, path_str: &str, content: &str) -> Result<(), &'static str> {
        let path = Path::new(path_str);
        
        // Get parent directory path and new file name
        let parent_path = if path.components().len() > 1 {
            let mut parent = self.current_path.clone();
            for i in 0..path.components().len() - 1 {
                parent = parent.join(&Path::new(path.components()[i].as_str()));
            }
            parent
        } else {
            self.current_path.clone()
        };
        
        let file_name = path.file_name().ok_or("Invalid file name")?;
        
        // Check if a file/directory with this name already exists
        let parent_entry = self.get_entry_mut(&parent_path)?;
        
        if !parent_entry.is_directory() {
            return Err("Parent is not a directory");
        }
        
        if let Some(ref mut children) = parent_entry.children {
            if children.contains_key(file_name) {
                return Err("Entry already exists");
            }
            
            let new_file = FSEntry::new_file(file_name, Some(FileContent::from_string(content)));
            children.insert(String::from(file_name), new_file);
            
            Ok(())
        } else {
            Err("Parent directory error")
        }
    }
    
    // Read file content
    pub fn read_file(&self, path_str: &str) -> Result<String, &'static str> {
        let path = Path::new(path_str);
        let target_path = if path.is_absolute() {
            path
        } else {
            self.current_path.join(&path)
        };
        
        let entry = self.get_entry(&target_path)?;
        
        if !entry.is_file() {
            return Err("Not a file");
        }
        
        if let Some(ref content) = entry.content {
            Ok(content.as_string())
        } else {
            Err("File has no content")
        }
    }
    
    // Delete a file or directory
    pub fn delete_entry(&mut self, path_str: &str) -> Result<(), &'static str> {
        let path = Path::new(path_str);
        let target_path = if path.is_absolute() {
            path
        } else {
            self.current_path.join(&path)
        };
        
        let file_name = target_path.file_name().ok_or("Invalid path")?;
        let parent_path = target_path.parent().ok_or("Cannot delete root")?;
        
        let parent_entry = self.get_entry_mut(&parent_path)?;
        
        if !parent_entry.is_directory() {
            return Err("Parent is not a directory");
        }
        
        if let Some(ref mut children) = parent_entry.children {
            if children.contains_key(file_name) {
                children.remove(file_name);
                Ok(())
            } else {
                Err("File or directory not found")
            }
        } else {
            Err("Parent directory error")
        }
    }
    
    // Get entry reference by path
    fn get_entry(&self, path: &Path) -> Result<&FSEntry, &'static str> {
        if path.components().is_empty() {
            return Ok(&self.root);
        }
        
        let mut current_entry = &self.root;
        
        for component in path.components() {
            if !current_entry.is_directory() {
                return Err("Path component is not a directory");
            }
            
            if let Some(ref children) = current_entry.children {
                if let Some(entry) = children.get(component) {
                    current_entry = entry;
                } else {
                    return Err("Path not found");
                }
            } else {
                return Err("Directory has no children");
            }
        }
        
        Ok(current_entry)
    }
    
    // Get mutable entry reference by path
    fn get_entry_mut(&mut self, path: &Path) -> Result<&mut FSEntry, &'static str> {
        if path.components().is_empty() {
            return Ok(&mut self.root);
        }
        
        let mut current_entry = &mut self.root;
        
        for component in path.components() {
            if !current_entry.is_directory() {
                return Err("Path component is not a directory");
            }
            
            if let Some(ref mut children) = current_entry.children {
                if let Some(entry) = children.get_mut(component) {
                    current_entry = entry;
                } else {
                    return Err("Path not found");
                }
            } else {
                return Err("Directory has no children");
            }
        }
        
        Ok(current_entry)
    }
}

// Global filesystem instance
lazy_static! {
    pub static ref FILESYSTEM: Mutex<FileSystem> = Mutex::new(FileSystem::new());
}

// Format the directory listing to display in the terminal
pub fn format_dir_listing(entries: &[FSEntry]) -> String {
    let mut result = String::new();
    
    result.push_str("Directory listing:\n\n");
    result.push_str("Name                 Size     Type\n");
    result.push_str("------------------------------------\n");
    
    for entry in entries {
        let type_str = if entry.is_directory() { "DIR" } else { "FILE" };
        let size_str = if entry.is_directory() { String::from("<DIR>") } else { 
            let mut bytes_str = String::new();
            let mut num = entry.metadata.size;
            if num == 0 {
                bytes_str.push('0');
            } else {
                while num > 0 {
                    let digit = (num % 10) as u8;
                    bytes_str.push((b'0' + digit) as char);
                    num /= 10;
                }
            }
            // Reverse the string
            let mut reversed = String::new();
            for c in bytes_str.chars().rev() {
                reversed.push(c);
            }
            reversed.push_str(" bytes");
            reversed
        };
        
        let mut line = String::new();
        // Format name (left-aligned, 20 chars)
        line.push_str(&entry.name);
        let mut spaces = 20 - entry.name.len();
        if spaces < 0 { spaces = 0; }
        for _ in 0..spaces {
            line.push(' ');
        }
        
        // Add size (left-aligned, 8 chars)
        line.push_str(&size_str);
        spaces = 8 - size_str.len();
        if spaces < 0 { spaces = 0; }
        for _ in 0..spaces {
            line.push(' ');
        }
        
        // Add type
        line.push_str(type_str);
        line.push('\n');
        
        result.push_str(&line);
    }
    
    result
}

// Initialize the filesystem (call this from main.rs)
pub fn init() {
    println!("Initializing filesystem...");
    // The filesystem is automatically initialized when first accessed
    let _fs = FILESYSTEM.lock();
    println!("RAM filesystem initialized");
}

// Helper function to list a directory and return the formatted string
pub fn list_directory_str(path_str: Option<&str>) -> Result<String, &'static str> {
    let entries = {
        let fs = FILESYSTEM.lock();
        fs.list_directory(path_str)?
    };
    
    Ok(format_dir_listing(&entries))
}

// Helper function to read a file and return its content as a String
pub fn read_file_str(path_str: &str) -> Result<String, &'static str> {
    let fs = FILESYSTEM.lock();
    fs.read_file(path_str)
}

// Helper function to copy a file to another location
pub fn copy_file(src: &str, dst: &str) -> Result<(), &'static str> {
    // First read the source file
    let content = {
        let fs = FILESYSTEM.lock();
        fs.read_file(src)?
    };
    
    // Then create the destination file with the same content
    let result = {
        let mut fs = FILESYSTEM.lock();
        fs.create_file(dst, &content)
    };
    
    result
} 