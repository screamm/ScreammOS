//! Fönsterhanteringssystem för ScreammOS
//! Inspirerat av tidiga DOS-fönsterhanteringssystem

use crate::vga_buffer::WRITER;
use crate::ui::{BorderStyle, Rect, Theme};

/// Representerar ett fönster i gränssnittet
pub struct Window {
    title: &'static str,
    bounds: Rect,
    is_active: bool,
    is_visible: bool,
    style: BorderStyle,
    theme: Theme,
}

impl Window {
    /// Skapa ett nytt fönster
    pub fn new(title: &'static str, x: usize, y: usize, width: usize, height: usize, theme: Theme) -> Self {
        Self {
            title,
            bounds: Rect { x, y, width, height },
            is_active: false,
            is_visible: true,
            style: BorderStyle::Double, // Standard DOS-stil
            theme,
        }
    }
    
    /// Visa fönstret
    pub fn show(&mut self) {
        self.is_visible = true;
        self.render();
    }
    
    /// Dölj fönstret
    pub fn hide(&mut self) {
        self.is_visible = false;
        // Här skulle vi idealt rensa området och rita om underliggande fönster
    }
    
    /// Ändra fönstrets position
    pub fn move_to(&mut self, x: usize, y: usize) {
        self.bounds.x = x;
        self.bounds.y = y;
        if self.is_visible {
            self.render();
        }
    }
    
    /// Ändra fönstrets storlek
    pub fn resize(&mut self, width: usize, height: usize) {
        self.bounds.width = width;
        self.bounds.height = height;
        if self.is_visible {
            self.render();
        }
    }
    
    /// Rita fönstret
    pub fn render(&self) {
        if !self.is_visible {
            return;
        }
        
        // Rita fönsterramen
        let mut writer = WRITER.lock();
        
        // Sätt färger baserat på om fönstret är aktivt
        let border_color = if self.is_active {
            self.theme.highlight_color 
        } else {
            self.theme.border_color
        };
        
        // Framtida implementation: Rita skuggor om de är aktiverade
        // if self.theme.shadow_enabled { ... }
        
        // Rita ramen
        let (top_left, top_right, bottom_left, bottom_right, horizontal, vertical) = match self.style {
            BorderStyle::Single => (0xDA, 0xBF, 0xC0, 0xD9, 0xC4, 0xB3),
            BorderStyle::Double => (0xC9, 0xBB, 0xC8, 0xBC, 0xCD, 0xBA),
            BorderStyle::SingleHeavy => (0xD5, 0xB8, 0xD4, 0xBE, 0xCD, 0xB3),
        };
        
        // Rita övre ramen
        writer.set_color(border_color, self.theme.window_bg);
        
        // Toppen
        for i in self.bounds.x..self.bounds.x+self.bounds.width {
            writer.column_position = i;
            writer.row_position = self.bounds.y;
            writer.write_byte(if i == self.bounds.x { top_left } else if i == self.bounds.x+self.bounds.width-1 { top_right } else { horizontal });
        }
        
        // Titel
        writer.column_position = self.bounds.x + 2;
        writer.row_position = self.bounds.y;
        writer.write_byte(b' ');
        writer.set_color(self.theme.highlight_color, self.theme.window_bg);
        for &byte in self.title.as_bytes() {
            writer.write_byte(byte);
        }
        writer.set_color(border_color, self.theme.window_bg);
        writer.write_byte(b' ');
        
        // Sidorna
        for y in self.bounds.y+1..self.bounds.y+self.bounds.height-1 {
            // Vänster kant
            writer.column_position = self.bounds.x;
            writer.row_position = y;
            writer.write_byte(vertical);
            
            // Fyll insidan
            writer.set_color(self.theme.window_fg, self.theme.window_bg);
            for x in self.bounds.x+1..self.bounds.x+self.bounds.width-1 {
                writer.column_position = x;
                writer.write_byte(b' ');
            }
            
            // Höger kant
            writer.set_color(border_color, self.theme.window_bg);
            writer.column_position = self.bounds.x + self.bounds.width - 1;
            writer.write_byte(vertical);
        }
        
        // Botten
        writer.set_color(border_color, self.theme.window_bg);
        for i in self.bounds.x..self.bounds.x+self.bounds.width {
            writer.column_position = i;
            writer.row_position = self.bounds.y + self.bounds.height - 1;
            writer.write_byte(if i == self.bounds.x { bottom_left } else if i == self.bounds.x+self.bounds.width-1 { bottom_right } else { horizontal });
        }
    }
    
    /// Skriv text i fönstret på en given position
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

/// Hanterar och organiserar alla fönster
pub struct WindowManager {
    windows: [Option<Window>; 10], // Max 10 fönster för enkel implementation
    active_window: Option<usize>,
    next_window_slot: usize,
}

impl WindowManager {
    /// Skapa en ny fönsterhanterare
    pub fn new() -> Self {
        Self {
            windows: [None, None, None, None, None, None, None, None, None, None],
            active_window: None,
            next_window_slot: 0,
        }
    }
    
    /// Lägg till ett nytt fönster
    pub fn add_window(&mut self, window: Window) -> Option<usize> {
        if self.next_window_slot >= self.windows.len() {
            return None; // Ingen plats för fler fönster
        }
        
        let window_id = self.next_window_slot;
        self.windows[window_id] = Some(window);
        self.next_window_slot += 1;
        
        // Aktivera fönstret om det är det första
        if self.active_window.is_none() {
            self.active_window = Some(window_id);
            if let Some(window) = &mut self.windows[window_id] {
                window.is_active = true;
            }
        }
        
        // Rendera fönstret
        if let Some(window) = &self.windows[window_id] {
            window.render();
        }
        
        Some(window_id)
    }
    
    /// Aktivera ett fönster
    pub fn activate_window(&mut self, window_id: usize) -> bool {
        if window_id >= self.windows.len() || self.windows[window_id].is_none() {
            return false;
        }
        
        // Inaktivera det nuvarande aktiva fönstret
        if let Some(active_id) = self.active_window {
            if let Some(window) = &mut self.windows[active_id] {
                window.is_active = false;
                window.render();
            }
        }
        
        // Aktivera det nya fönstret
        if let Some(window) = &mut self.windows[window_id] {
            window.is_active = true;
            window.render();
            self.active_window = Some(window_id);
            return true;
        }
        
        false
    }
    
    /// Rendera alla synliga fönster
    pub fn render_all(&self) {
        // Rendera fönster nerifrån och upp för korrekt z-ordning
        for window in self.windows.iter().flatten() {
            window.render();
        }
    }
    
    /// Visa ett meddelande i ett nytt fönster
    pub fn show_message(&mut self, title: &'static str, message: &'static str, theme: Theme) -> Option<usize> {
        let width = message.len() + 6;
        let height = 5;
        
        // Centrera i mitten av skärmen
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
    
    /// Visa en enkel dialogruta med knappar
    pub fn show_dialog(&mut self, title: &'static str, message: &'static str, 
                      _buttons: &[&'static str], theme: Theme) -> Option<usize> {
        // Implementera senare: Dialogruta med knappar
        self.show_message(title, message, theme)
    }
} 