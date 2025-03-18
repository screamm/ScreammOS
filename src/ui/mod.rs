//! User interface module for ScreammOS

pub mod window_manager;

use crate::vga_buffer::{Color, WRITER};

/// Different UI themes for ScreammOS
pub enum UITheme {
    DOSClassic,     // Classic DOS style - blue background, white text
    AmberTerminal,  // Amber terminal - yellow/brown text on black background
    GreenCRT,       // Green CRT terminal - green text on black background
    ModernRetro,    // Modern retro style with 16-color palette
    Custom(Theme),  // Custom theme
}

/// Customizable theme settings
pub struct Theme {
    pub window_bg: Color,
    pub window_fg: Color,
    pub border_color: Color,
    pub highlight_color: Color,
    pub menu_bg: Color,
    pub menu_fg: Color,
    pub shadow_enabled: bool,
    pub crt_effect: bool,
}

impl Theme {
    /// Create the classic DOS theme
    pub fn dos_classic() -> Self {
        Self {
            window_bg: Color::Blue,
            window_fg: Color::White,
            border_color: Color::LightGray,
            highlight_color: Color::Yellow,
            menu_bg: Color::LightGray,
            menu_fg: Color::Black,
            shadow_enabled: true,
            crt_effect: false,
        }
    }
    
    /// Create the Amber terminal theme
    pub fn amber_terminal() -> Self {
        Self {
            window_bg: Color::Black,
            window_fg: Color::Brown,
            border_color: Color::Brown,
            highlight_color: Color::Yellow,
            menu_bg: Color::Black,
            menu_fg: Color::Brown,
            shadow_enabled: false,
            crt_effect: true,
        }
    }
    
    /// Create the green CRT theme
    pub fn green_crt() -> Self {
        Self {
            window_bg: Color::Black,
            window_fg: Color::Green,
            border_color: Color::Green,
            highlight_color: Color::LightGreen,
            menu_bg: Color::Black,
            menu_fg: Color::Green,
            shadow_enabled: false,
            crt_effect: true,
        }
    }
}

/// A basic rectangle for layout
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

/// Base class for all UI components
pub trait Component {
    fn render(&self);
    fn handle_input(&mut self, key: u8) -> bool;
    fn get_bounds(&self) -> Rect;
}

/// Border types for DOS-style UI
pub enum BorderStyle {
    Single,     // ─ │ ┌ ┐ └ ┘
    Double,     // ═ ║ ╔ ╗ ╚ ╝
    SingleHeavy // ━ ┃ ┏ ┓ ┗ ┛
}

/// Draw a DOS-style box
pub fn draw_box(rect: Rect, style: BorderStyle, title: Option<&str>) {
    let mut writer = WRITER.lock();
    
    // Characters for different border types
    let (top_left, top_right, bottom_left, bottom_right, horizontal, vertical) = match style {
        BorderStyle::Single => (0xDA, 0xBF, 0xC0, 0xD9, 0xC4, 0xB3),
        BorderStyle::Double => (0xC9, 0xBB, 0xC8, 0xBC, 0xCD, 0xBA),
        BorderStyle::SingleHeavy => (0xD5, 0xB8, 0xD4, 0xBE, 0xCD, 0xB3),
    };
    
    // Draw upper frame
    writer.set_color(Color::LightGray, Color::Blue);
    for i in rect.x..rect.x+rect.width {
        writer.write_byte(if i == rect.x { top_left } else if i == rect.x+rect.width-1 { top_right } else { horizontal });
    }
    
    // Draw title if present
    if let Some(title) = title {
        // Position for title
        let title_pos = rect.x + 2;
        writer.column_position = title_pos;
        writer.row_position = rect.y;
        
        // Draw the title
        writer.write_byte(b' ');
        for &byte in title.as_bytes() {
            writer.write_byte(byte);
        }
        writer.write_byte(b' ');
    }
    
    // Draw side frames and filling
    for y in rect.y+1..rect.y+rect.height-1 {
        writer.column_position = rect.x;
        writer.row_position = y;
        writer.write_byte(vertical);
        
        // Fill with spaces
        for _ in rect.x+1..rect.x+rect.width-1 {
            writer.write_byte(b' ');
        }
        
        writer.write_byte(vertical);
    }
    
    // Draw lower frame
    writer.column_position = rect.x;
    writer.row_position = rect.y + rect.height - 1;
    for i in rect.x..rect.x+rect.width {
        writer.write_byte(if i == rect.x { bottom_left } else if i == rect.x+rect.width-1 { bottom_right } else { horizontal });
    }
} 