//! Window management system for ScreammOS
//! Inspired by early DOS window management systems

use crate::vga_buffer::WRITER;
use crate::ui::{BorderStyle, Rect, Theme};

/// Represents a window in the interface
pub struct Window {
    title: &'static str,
    bounds: Rect,
    is_active: bool,
    is_visible: bool,
    style: BorderStyle,
    theme: Theme,
}

impl Window {
    /// Create a new window
    pub fn new(title: &'static str, x: usize, y: usize, width: usize, height: usize, theme: Theme) -> Self {
        Self {
            title,
            bounds: Rect { x, y, width, height },
            is_active: false,
            is_visible: true,
            style: BorderStyle::Double, // Standard DOS style
            theme,
        }
    }
    
    /// Show the window
    pub fn show(&mut self) {
        self.is_visible = true;
        self.render();
    }
    
    /// Hide the window
    pub fn hide(&mut self) {
        self.is_visible = false;
        // Ideally we would clear the area and redraw underlying windows here
    }
    
    /// Change window position
    pub fn move_to(&mut self, x: usize, y: usize) {
        self.bounds.x = x;
        self.bounds.y = y;
        if self.is_visible {
            self.render();
        }
    }
    
    /// Change window size
    pub fn resize(&mut self, width: usize, height: usize) {
        self.bounds.width = width;
        self.bounds.height = height;
        if self.is_visible {
            self.render();
        }
    }
    
    /// Draw the window
    pub fn render(&self) {
        if !self.is_visible {
            return;
        }
        
        // Draw window frame
        let mut writer = WRITER.lock();
        
        // Set colors based on whether the window is active
        let border_color = if self.is_active {
            self.theme.highlight_color 
        } else {
            self.theme.border_color
        };
        
        // Future implementation: Draw shadows if enabled
        // if self.theme.shadow_enabled { ... }
        
        // Draw the frame
        let (top_left, top_right, bottom_left, bottom_right, horizontal, vertical) = match self.style {
            BorderStyle::Single => (0xDA, 0xBF, 0xC0, 0xD9, 0xC4, 0xB3),
            BorderStyle::Double => (0xC9, 0xBB, 0xC8, 0xBC, 0xCD, 0xBA),
            BorderStyle::SingleHeavy => (0xD5, 0xB8, 0xD4, 0xBE, 0xCD, 0xB3),
        };
        
        // Draw upper frame
        writer.set_color(border_color, self.theme.window_bg);
        
        // Top
        for i in self.bounds.x..self.bounds.x+self.bounds.width {
            writer.column_position = i;
            writer.row_position = self.bounds.y;
            writer.write_byte(if i == self.bounds.x { top_left } else if i == self.bounds.x+self.bounds.width-1 { top_right } else { horizontal });
        }
        
        // Title
        writer.column_position = self.bounds.x + 2;
        writer.row_position = self.bounds.y;
        writer.write_byte(b' ');
        writer.set_color(self.theme.highlight_color, self.theme.window_bg);
        for &byte in self.title.as_bytes() {
            writer.write_byte(byte);
        }
        writer.set_color(border_color, self.theme.window_bg);
        writer.write_byte(b' ');
        
        // Sides
        for y in self.bounds.y+1..self.bounds.y+self.bounds.height-1 {
            // Left edge
            writer.column_position = self.bounds.x;
            writer.row_position = y;
            writer.write_byte(vertical);
            
            // Fill inside
            writer.set_color(self.theme.window_fg, self.theme.window_bg);
            for x in self.bounds.x+1..self.bounds.x+self.bounds.width-1 {
                writer.column_position = x;
                writer.write_byte(b' ');
            }
            
            // Right edge
            writer.set_color(border_color, self.theme.window_bg);
            writer.column_position = self.bounds.x + self.bounds.width - 1;
            writer.write_byte(vertical);
        }
        
        // Bottom
        writer.set_color(border_color, self.theme.window_bg);
        for i in self.bounds.x..self.bounds.x+self.bounds.width {
            writer.column_position = i;
            writer.row_position = self.bounds.y + self.bounds.height - 1;
            writer.write_byte(if i == self.bounds.x { bottom_left } else if i == self.bounds.x+self.bounds.width-1 { bottom_right } else { horizontal });
        }
    }
    
    /// Write text in the window at a given position
    pub fn write_at(&self, x: usize, y: usize, text: &str) {
        if !self.is_visible || 
           x >= self.bounds.width - 2 || 
           y >= self.bounds.height - 2 {
            return;
        }
        
        let mut writer = WRITER.lock();
        writer.set_color(self.theme.window_fg, self.theme.window_bg);
        
        let abs_x = self.bounds.x + 1 + x;
        let abs_y = self.bounds.y + 1 + y;
        
        writer.column_position = abs_x;
        writer.row_position = abs_y;
        
        for &byte in text.as_bytes() {
            writer.write_byte(byte);
        }
    }
}

/// Manages and organizes all windows
pub struct WindowManager {
    windows: [Option<Window>; 10], // Max 10 windows for simple implementation
    active_window: Option<usize>,
    next_window_slot: usize,
}

impl WindowManager {
    /// Create a new window manager
    pub fn new() -> Self {
        Self {
            windows: [None, None, None, None, None, None, None, None, None, None],
            active_window: None,
            next_window_slot: 0,
        }
    }
    
    /// Add a new window
    pub fn add_window(&mut self, window: Window) -> Option<usize> {
        if self.next_window_slot >= self.windows.len() {
            return None; // No space for more windows
        }
        
        let window_id = self.next_window_slot;
        self.windows[window_id] = Some(window);
        self.next_window_slot += 1;
        
        // Activate the window if it's the first one
        if self.active_window.is_none() {
            self.active_window = Some(window_id);
            if let Some(window) = &mut self.windows[window_id] {
                window.is_active = true;
            }
        }
        
        // Render the window
        if let Some(window) = &self.windows[window_id] {
            window.render();
        }
        
        Some(window_id)
    }
    
    /// Activate a window
    pub fn activate_window(&mut self, window_id: usize) -> bool {
        if window_id >= self.windows.len() || self.windows[window_id].is_none() {
            return false;
        }
        
        // Deactivate the current active window
        if let Some(active_id) = self.active_window {
            if let Some(window) = &mut self.windows[active_id] {
                window.is_active = false;
                window.render();
            }
        }
        
        // Activate the new window
        if let Some(window) = &mut self.windows[window_id] {
            window.is_active = true;
            window.render();
            self.active_window = Some(window_id);
            return true;
        }
        
        false
    }
    
    /// Render all visible windows
    pub fn render_all(&self) {
        // Render windows from bottom to top for correct z-ordering
        for window in self.windows.iter().flatten() {
            window.render();
        }
    }
    
    /// Show a message in a new window
    pub fn show_message(&mut self, title: &'static str, message: &'static str, theme: Theme) -> Option<usize> {
        let width = message.len() + 6;
        let height = 5;
        
        // Center in the middle of the screen
        let x = (80 - width) / 2;
        let y = (25 - height) / 2;
        
        let mut window = Window::new(title, x, y, width, height, theme);
        window.style = BorderStyle::Double;
        
        let window_id = self.add_window(window)?;
        
        if let Some(window) = &self.windows[window_id] {
            window.write_at(2, 1, message);
        }
        
        Some(window_id)
    }
    
    /// Show a simple dialog with buttons
    pub fn show_dialog(&mut self, title: &'static str, message: &'static str, 
                      _buttons: &[&'static str], theme: Theme) -> Option<usize> {
        // Implement later: Dialog with buttons
        self.show_message(title, message, theme)
    }
} 