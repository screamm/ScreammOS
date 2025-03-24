// src/ui/text_editor.rs
// Text editor for ScreammOS

use crate::vga_buffer::{BUFFER_HEIGHT, BUFFER_WIDTH, WRITER, Color};
use crate::ui::{Rect, BorderStyle, draw_box};
use crate::simple_fs::{FILESYSTEM, SimpleString};
use spin::Mutex;
use core::fmt::Write;

const EDITOR_WIDTH: usize = 60;
const EDITOR_HEIGHT: usize = 20;
const EDITOR_TEXT_HEIGHT: usize = EDITOR_HEIGHT - 6; // Space for title and status bar

const MAX_LINES: usize = 100; // Max number of lines we can edit
const MAX_LINE_LENGTH: usize = 80; // Max length per line

/// A simple text editor
pub struct TextEditor {
    filename: SimpleString,
    content: [SimpleString; MAX_LINES],
    line_count: usize,
    cursor_x: usize,
    cursor_y: usize,
    scroll_offset: usize,
    rect: Rect,
    pub visible: bool,
    modified: bool,
}

// Helper function for formatting
fn format_str(args: core::fmt::Arguments) -> SimpleString {
    let mut s = SimpleString::new();
    s.push_str("[Formatting not supported]");
    s
}

// Helper function for title formatting
fn format_title(prefix: &str, name: &str, modified: bool) -> SimpleString {
    let mut s = SimpleString::new();
    s.push_str(prefix);
    s.push_str(" ");
    s.push_str(name);
    if modified {
        s.push_str(" *");
    }
    s
}

// Helper function for status bar
fn format_status(row: usize, total_rows: usize, col: usize) -> SimpleString {
    let mut s = SimpleString::new();
    s.push_str("Row: ");
    // Convert row to string
    let mut row_str = SimpleString::new();
    let row_plus_one = row + 1;
    let mut digits = [0; 10];
    let mut i = 0;
    let mut num = row_plus_one;
    while num > 0 {
        digits[i] = (num % 10) as u8;
        num /= 10;
        i += 1;
    }
    if i == 0 {
        row_str.push('0');
    } else {
        for j in (0..i).rev() {
            row_str.push((digits[j] + b'0') as char);
        }
    }
    s.push_str(row_str.as_str());
    s.push_str("/");
    
    // Convert total_rows to string using same technique
    let mut total_str = SimpleString::new();
    let mut digits = [0; 10];
    let mut i = 0;
    let mut num = total_rows;
    while num > 0 {
        digits[i] = (num % 10) as u8;
        num /= 10;
        i += 1;
    }
    if i == 0 {
        total_str.push('0');
    } else {
        for j in (0..i).rev() {
            total_str.push((digits[j] + b'0') as char);
        }
    }
    s.push_str(total_str.as_str());
    
    s.push_str(" Col: ");
    
    // Convert col to string
    let mut col_str = SimpleString::new();
    let col_plus_one = col + 1;
    let mut digits = [0; 10];
    let mut i = 0;
    let mut num = col_plus_one;
    while num > 0 {
        digits[i] = (num % 10) as u8;
        num /= 10;
        i += 1;
    }
    if i == 0 {
        col_str.push('0');
    } else {
        for j in (0..i).rev() {
            col_str.push((digits[j] + b'0') as char);
        }
    }
    s.push_str(col_str.as_str());
    
    // Add shortcuts info
    s.push_str(" | Esc: Close | Ctrl+S: Save");
    s
}

impl TextEditor {
    /// Create a new text editor
    pub fn new() -> Self {
        // Center the window on the screen
        let x = (BUFFER_WIDTH - EDITOR_WIDTH) / 2;
        let y = (BUFFER_HEIGHT - EDITOR_HEIGHT) / 2;
        
        // Initialize all lines as empty strings
        let content = [SimpleString::new(); MAX_LINES];
        
        Self {
            filename: SimpleString::new(),
            content,
            line_count: 0,
            cursor_x: 0,
            cursor_y: 0,
            scroll_offset: 0,
            rect: Rect {
                x,
                y,
                width: EDITOR_WIDTH,
                height: EDITOR_HEIGHT,
            },
            visible: false,
            modified: false,
        }
    }
    
    /// Open a file for editing
    pub fn open_file(&mut self, filename: &str) -> bool {
        let fs = FILESYSTEM.lock();
        
        match fs.read_file(filename) {
            Ok(content) => {
                // Clear existing content
                for i in 0..MAX_LINES {
                    self.content[i] = SimpleString::new();
                }
                
                // Set the filename
                self.filename = SimpleString::new();
                self.filename.push_str(filename);
                
                // Split content into lines
                let mut line_index = 0;
                let mut current_line = SimpleString::new();
                
                for c in content.chars() {
                    if c == '\n' {
                        // Save the line and move to the next
                        self.content[line_index] = current_line;
                        line_index += 1;
                        
                        if line_index >= MAX_LINES {
                            break; // Too many lines
                        }
                        
                        current_line = SimpleString::new();
                    } else {
                        // Add character to current line
                        current_line.push(c);
                    }
                }
                
                // Handle the last line if it doesn't end with a line break
                if current_line.len() > 0 && line_index < MAX_LINES {
                    self.content[line_index] = current_line;
                    line_index += 1;
                }
                
                self.line_count = line_index;
                self.cursor_x = 0;
                self.cursor_y = 0;
                self.scroll_offset = 0;
                self.modified = false;
                
                true
            },
            Err(_) => {
                // Could not read the file, but we'll create a new empty file
                self.filename = SimpleString::new();
                self.filename.push_str(filename);
                
                // Clear the content
                for i in 0..MAX_LINES {
                    self.content[i] = SimpleString::new();
                }
                
                self.line_count = 1; // An empty line
                self.cursor_x = 0;
                self.cursor_y = 0;
                self.scroll_offset = 0;
                self.modified = true; // Mark as modified since it's new
                
                true
            }
        }
    }
    
    /// Save the file
    pub fn save_file(&mut self) -> bool {
        if self.filename.len() == 0 {
            return false; // No filename
        }
        
        // Build up the content as a string
        let mut content = SimpleString::new();
        
        for i in 0..self.line_count {
            content.push_str(self.content[i].as_str());
            if i < self.line_count - 1 {
                content.push_str("\n");
            }
        }
        
        // Save the file
        let mut fs = FILESYSTEM.lock();
        match fs.create_file(self.filename.as_str(), content.as_str()) {
            Ok(_) => {
                self.modified = false;
                true
            },
            Err(_) => false
        }
    }
    
    /// Show the editor
    pub fn show(&mut self) {
        self.visible = true;
        self.render();
    }
    
    /// Hide the editor
    pub fn hide(&mut self) {
        self.visible = false;
    }
    
    /// Insert a character at the cursor position
    pub fn insert_char(&mut self, c: char) {
        if !self.visible {
            return;
        }
        
        if c == '\n' {
            // Handle line break, split the current line
            if self.line_count < MAX_LINES {
                // Make space for the new line
                for i in (self.cursor_y + 1..self.line_count).rev() {
                    self.content[i + 1] = self.content[i].clone();
                }
                
                // Read the content of the current line
                let current_text = self.content[self.cursor_y].as_str();
                
                // Create before-cursor content
                let mut before_line = SimpleString::new();
                for i in 0..self.cursor_x {
                    if i < current_text.len() {
                        before_line.push(current_text.chars().nth(i).unwrap_or(' '));
                    }
                }
                
                // Create after-cursor content
                let mut after_line = SimpleString::new();
                for i in self.cursor_x..current_text.len() {
                    after_line.push(current_text.chars().nth(i).unwrap_or(' '));
                }
                
                // Update the lines
                self.content[self.cursor_y] = before_line;
                self.content[self.cursor_y + 1] = after_line;
                
                // Update line count and cursor position
                self.line_count += 1;
                self.cursor_y += 1;
                self.cursor_x = 0;
                self.modified = true;
            }
        } else {
            // Regular character, insert in current line
            if self.cursor_y < self.line_count {
                let current_line_index = self.cursor_y;
                let current_text = self.content[current_line_index].as_str();
                
                if current_text.len() < MAX_LINE_LENGTH {
                    // Create a new line with the inserted character
                    let mut new_line_content = SimpleString::new();
                    
                    // Copy text before cursor
                    for i in 0..self.cursor_x {
                        if i < current_text.len() {
                            new_line_content.push(current_text.chars().nth(i).unwrap_or(' '));
                        }
                    }
                    
                    // Insert the new character
                    new_line_content.push(c);
                    
                    // Copy text after cursor
                    for i in self.cursor_x..current_text.len() {
                        new_line_content.push(current_text.chars().nth(i).unwrap_or(' '));
                    }
                    
                    // Set the new line
                    self.content[current_line_index] = new_line_content;
                    
                    // Move cursor
                    self.cursor_x += 1;
                    self.modified = true;
                }
            }
        }
        
        self.ensure_cursor_visible();
        self.render();
    }
    
    /// Handle backspace
    pub fn handle_backspace(&mut self) {
        if !self.visible || (self.cursor_x == 0 && self.cursor_y == 0) {
            return;
        }
        
        if self.cursor_x > 0 {
            // Delete a character on the current line
            let current_line_index = self.cursor_y;
            let current_text = self.content[current_line_index].as_str();
            
            // Create a new string without the character at cursor-1
            let mut new_content = SimpleString::new();
            
            // Copy all characters except the one to be deleted
            for i in 0..current_text.len() {
                if i != self.cursor_x - 1 {
                    new_content.push(current_text.chars().nth(i).unwrap_or(' '));
                }
            }
            
            // Replace the content
            self.content[current_line_index] = new_content;
            
            // Move cursor back
            self.cursor_x -= 1;
            self.modified = true;
        } else if self.cursor_y > 0 {
            // We are at the beginning of a line, merge with the previous line
            let current_line_index = self.cursor_y;
            let prev_line_index = self.cursor_y - 1;
            
            // Copy the current line's content to a temporary string
            let mut current_text_copy = SimpleString::new();
            {
                let current_text = self.content[current_line_index].as_str();
                for c in current_text.chars() {
                    current_text_copy.push(c);
                }
            }
            
            // Move cursor
            self.cursor_y -= 1;
            
            // Find the length of the previous line
            let prev_cursor_pos = self.content[prev_line_index].len();
            self.cursor_x = prev_cursor_pos;
            
            // Add the content from the current line to the end of the previous line
            for c in current_text_copy.as_str().chars() {
                self.content[prev_line_index].push(c);
            }
            
            // Remove the current line
            for i in current_line_index..self.line_count - 1 {
                self.content[i] = self.content[i + 1].clone();
            }
            self.content[self.line_count - 1] = SimpleString::new();
            
            // Update line count
            self.line_count -= 1;
            self.modified = true;
        }
        
        self.ensure_cursor_visible();
        self.render();
    }
    
    /// Handle deletion (delete key)
    pub fn handle_delete(&mut self) {
        if !self.visible {
            return;
        }
        
        if self.cursor_y < self.line_count {
            let current_line_index = self.cursor_y;
            let current_text = self.content[current_line_index].as_str();
            
            if self.cursor_x < current_text.len() {
                // Remove the character at the cursor position
                let mut new_content = SimpleString::new();
                
                // Copy all characters except the one to be deleted
                for i in 0..current_text.len() {
                    if i != self.cursor_x {
                        new_content.push(current_text.chars().nth(i).unwrap_or(' '));
                    }
                }
                
                // Replace the content
                self.content[current_line_index] = new_content;
                self.modified = true;
            } else if self.cursor_y < self.line_count - 1 {
                // Merge current line with next line when we're at the end
                let next_line_index = self.cursor_y + 1;
                
                // Copy next line's content to a temporary string
                let mut next_text_copy = SimpleString::new();
                {
                    let next_text = self.content[next_line_index].as_str();
                    for c in next_text.chars() {
                        next_text_copy.push(c);
                    }
                }
                
                // Add the content from the next line to the end of this one
                for c in next_text_copy.as_str().chars() {
                    self.content[current_line_index].push(c);
                }
                
                // Remove the next line
                for i in next_line_index..self.line_count - 1 {
                    self.content[i] = self.content[i + 1].clone();
                }
                self.content[self.line_count - 1] = SimpleString::new();
                
                // Update line count
                self.line_count -= 1;
                self.modified = true;
            }
        }
        
        self.ensure_cursor_visible();
        self.render();
    }
    
    /// Navigate up
    pub fn move_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            
            // Adjust X position if it's outside the line
            let line_len = self.content[self.cursor_y].len();
            if self.cursor_x > line_len {
                self.cursor_x = line_len;
            }
            
            self.ensure_cursor_visible();
            self.render();
        }
    }
    
    /// Navigate down
    pub fn move_down(&mut self) {
        if self.cursor_y < self.line_count - 1 {
            self.cursor_y += 1;
            
            // Adjust X position if it's outside the line
            let line_len = self.content[self.cursor_y].len();
            if self.cursor_x > line_len {
                self.cursor_x = line_len;
            }
            
            self.ensure_cursor_visible();
            self.render();
        }
    }
    
    /// Navigate left
    pub fn move_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
            self.render();
        } else if self.cursor_y > 0 {
            // Go to the end of previous line
            self.cursor_y -= 1;
            self.cursor_x = self.content[self.cursor_y].len();
            self.ensure_cursor_visible();
            self.render();
        }
    }
    
    /// Navigate right
    pub fn move_right(&mut self) {
        let line_len = self.content[self.cursor_y].len();
        
        if self.cursor_x < line_len {
            self.cursor_x += 1;
            self.render();
        } else if self.cursor_y < self.line_count - 1 {
            // Go to the beginning of next line
            self.cursor_y += 1;
            self.cursor_x = 0;
            self.ensure_cursor_visible();
            self.render();
        }
    }
    
    /// Make sure the cursor is visible
    fn ensure_cursor_visible(&mut self) {
        if self.cursor_y < self.scroll_offset {
            self.scroll_offset = self.cursor_y;
        } else if self.cursor_y >= self.scroll_offset + EDITOR_TEXT_HEIGHT {
            self.scroll_offset = self.cursor_y - EDITOR_TEXT_HEIGHT + 1;
        }
    }
    
    /// Draw the editor
    pub fn render(&mut self) {
        if !self.visible {
            return;
        }
        let title = format_title("Text editor -", self.filename.as_str(), self.modified);
        draw_box(self.rect, BorderStyle::Double, Some(title.as_str()));
        
        // Draw the content
        for i in 0..EDITOR_TEXT_HEIGHT {
            let line_index = i + self.scroll_offset;
            
            if line_index < self.line_count {
                // Draw the line
                let line = &self.content[line_index];
                let x = self.rect.x + 2;
                let y = self.rect.y + 2 + i;
                
                // Save current cursor position
                let mut writer = WRITER.lock();
                let saved_row = writer.column_position;
                let saved_col = writer.row_position;
                
                // Place cursor and write
                writer.column_position = y;
                writer.row_position = x;
                
                // Display the line
                write!(writer, "{}", line.as_str()).unwrap();
                
                // Restore cursor
                writer.column_position = saved_row;
                writer.row_position = saved_col;
            }
        }
        
        // Draw help text at the bottom
        let y = self.rect.y + self.rect.height - 2;
        let mut writer = WRITER.lock();
        for x in self.rect.x+1..self.rect.x+self.rect.width-1 {
            writer.write_char_at(x, y, ' ', Color::Black, Color::LightGray);
        }
        
        let x = self.rect.x + 2;
        writer.set_cursor_position(x, y);
        writer.set_color(Color::Black, Color::LightGray);
        write!(writer, "Ctrl+X: Cut | Ctrl+C: Copy | Ctrl+V: Paste | Ctrl+S: Save | Esc: Close").unwrap();
        
        // Set the visual cursor
        if self.cursor_y >= self.scroll_offset && self.cursor_y < self.scroll_offset + EDITOR_TEXT_HEIGHT {
            let cursor_screen_y = self.rect.y + 2 + (self.cursor_y - self.scroll_offset);
            let cursor_screen_x = self.rect.x + 2 + self.cursor_x;
            
            writer.column_position = cursor_screen_y;
            writer.row_position = cursor_screen_x;
        }
    }
}

// Create a global instance of the text editor
lazy_static::lazy_static! {
    pub static ref TEXT_EDITOR: Mutex<TextEditor> = Mutex::new(TextEditor::new());
} 