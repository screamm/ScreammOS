use crate::ui::{Rect, BorderStyle, draw_box};
use crate::vga_buffer::Theme;

pub struct WindowManager {
    // För framtida utökning
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            // För framtida utökning
        }
    }
    
    pub fn show_message(&mut self, title: &str, message: &str, _theme: Theme) {
        // Skapa ett enkelt meddelandefönster
        let rect = Rect {
            x: 10,
            y: 5,
            width: 40,
            height: 10,
        };
        
        draw_box(rect, BorderStyle::Double, Some(title));
        
        // Rita meddelandet centrerat i fönstret
        let mut writer = crate::vga_buffer::WRITER.lock();
        let saved_row = writer.column_position;
        let saved_col = writer.row_position;
        
        let x = rect.x + 2;
        let y = rect.y + 2;
        
        writer.column_position = y;
        writer.row_position = x;
        writer.write_string(message);
        
        writer.column_position = saved_row;
        writer.row_position = saved_col;
    }
}

/// Initialisera fönsterhanteraren
pub fn init() {
    // Lägg till init-kod här senare
    // För tillfället behöver vi bara denna funktion för att kompilera
} 