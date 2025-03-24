//! User interface module for ScreammOS

use spin::Mutex;
use crate::vga_buffer::{Color, WRITER};
use crate::println;
use crate::ui::command_line::CommandLine;

pub mod window_manager;
pub mod file_manager;
pub mod text_editor;
pub mod command_line;
pub mod splash_screen;
pub mod retro_commands;

pub static UI_STATE: Mutex<CommandLine> = Mutex::new(CommandLine::new());

/// Different UI themes for ScreammOS
pub enum UIThemeType {
    Classic,
    Dark,
    Light,
    Retro,
}

/// Customizable theme settings
pub struct UITheme {
    pub window_bg: Color,
    pub window_fg: Color,
    pub border_color: Color,
    pub highlight_color: Color,
    pub menu_bg: Color,
    pub menu_fg: Color,
    pub shadow_enabled: bool,
    pub crt_effect: bool,
}

impl UITheme {
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
    SingleHeavy, // ━ ┃ ┏ ┓ ┗ ┛
    None,
}

/// Draw a DOS-style box
pub fn draw_box(rect: Rect, style: BorderStyle, title: Option<&str>) {
    let mut writer = WRITER.lock();
    
    // Spara nuvarande position
    let saved_row = writer.row_position;
    let saved_col = writer.column_position;
    
    // Välj tecken för ramen baserat på stil
    let (top_left, top_right, bottom_left, bottom_right, horizontal, vertical) = match style {
        BorderStyle::Single => (b'\xDA', b'\xBF', b'\xC0', b'\xD9', b'\xC4', b'\xB3'),
        BorderStyle::Double => (b'\xC9', b'\xBB', b'\xC8', b'\xBC', b'\xCD', b'\xBA'),
        BorderStyle::SingleHeavy => (0xD5, 0xB8, 0xD4, 0xBE, 0xCD, 0xB3),
        BorderStyle::None => (b' ', b' ', b' ', b' ', b' ', b' '),
    };
    
    // Rita övre ramen
    writer.row_position = rect.y;
    writer.column_position = rect.x;
    writer.write_byte(top_left);
    
    // Rita titeln om den finns
    if let Some(title_text) = title {
        let title_len = title_text.len();
        let available_width = rect.width - 2;
        
        if title_len <= available_width {
            let padding_before = (available_width - title_len) / 2;
            let padding_after = available_width - title_len - padding_before;
            
            for _ in 0..padding_before {
                writer.write_byte(horizontal);
            }
            
            writer.write_string(title_text);
            
            for _ in 0..padding_after {
                writer.write_byte(horizontal);
            }
        } else {
            for _ in 0..(rect.width-2) {
                writer.write_byte(horizontal);
            }
        }
    } else {
        for _ in 0..(rect.width-2) {
            writer.write_byte(horizontal);
        }
    }
    
    writer.write_byte(top_right);
    
    // Rita sidoramarna
    for y in 1..(rect.height-1) {
        writer.row_position = rect.y + y;
        writer.column_position = rect.x;
        writer.write_byte(vertical);
        
        writer.column_position = rect.x + rect.width - 1;
        writer.write_byte(vertical);
    }
    
    // Rita nedre ramen
    writer.row_position = rect.y + rect.height - 1;
    writer.column_position = rect.x;
    writer.write_byte(bottom_left);
    
    for _ in 0..(rect.width-2) {
        writer.write_byte(horizontal);
    }
    
    writer.write_byte(bottom_right);
    
    // Återställ skrivarpositionen
    writer.row_position = saved_row;
    writer.column_position = saved_col;
}

/// Rensa insidan av en rektangel
pub fn clear_rect(rect: Rect) {
    let mut writer = WRITER.lock();
    
    // Spara nuvarande position
    let saved_row = writer.row_position;
    let saved_col = writer.column_position;
    
    // Rensa insidan av rektangeln
    for y in 1..(rect.height-1) {
        writer.row_position = rect.y + y;
        writer.column_position = rect.x + 1;
        
        for _ in 0..(rect.width-2) {
            writer.write_byte(b' ');
        }
    }
    
    // Återställ skrivarpositionen
    writer.row_position = saved_row;
    writer.column_position = saved_col;
}

/// Initialisera UI-systemet
pub fn init() {
    println!("UI: Initialisering av användargränssnittet");
    
    // Registrera nödvändiga komponenter
    window_manager::init();
    
    println!("UI: Användargränssnitt initierat");
    println!("UI: Använd F1 för filhanteraren");
} 