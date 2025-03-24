// src/ui/splash_screen.rs
// Retro splash screen for ScreammOS

use crate::vga_buffer::{Color, WRITER};
use crate::{print, println};
use spin::Mutex;
use lazy_static::lazy_static;
use alloc::vec::Vec;
use crate::string_ext::StringExt;

const SPLASH_ART: &str = r#"
    ███████╗ ██████╗██████╗ ███████╗ █████╗ ███╗   ███╗███╗   ███╗ ██████╗ ███████╗
    ██╔════╝██╔════╝██╔══██╗██╔════╝██╔══██╗████╗ ████║████╗ ████║██╔═══██╗██╔════╝
    ███████╗██║     ██████╔╝█████╗  ███████║██╔████╔██║██╔████╔██║██║   ██║███████╗
    ╚════██║██║     ██╔══██╗██╔══╝  ██╔══██║██║╚██╔╝██║██║╚██╔╝██║██║   ██║╚════██║
    ███████║╚██████╗██║  ██║███████╗██║  ██║██║ ╚═╝ ██║██║ ╚═╝ ██║╚██████╔╝███████║
    ╚══════╝ ╚═════╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝     ╚═╝ ╚═════╝ ╚══════╝
"#;

pub struct SplashScreen {
    visible: bool,
    frame: u32,
}

impl SplashScreen {
    pub fn new() -> Self {
        Self {
            visible: false,
            frame: 0,
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.frame = 0;
        self.render();
    }

    pub fn hide(&mut self) {
        self.visible = false;
        let mut writer = WRITER.lock();
        writer.clear_screen();
    }

    pub fn update(&mut self) {
        if self.visible {
            self.frame += 1;
            self.render();
        }
    }

    fn render(&self) {
        let mut writer = WRITER.lock();
        writer.clear_screen();
        
        // Set retro color scheme
        writer.set_color(Color::LightCyan, Color::Black);
        
        // Calculate center position
        let lines: Vec<&str> = SPLASH_ART.lines().filter(|line| !line.is_empty()).collect();
        let start_y = (25 - lines.len()) / 2;
        
        // Draw ASCII art
        for (i, line) in lines.iter().enumerate() {
            let x = (80 - line.len()) / 2;
            writer.set_position(x, start_y + i);
            print!("{}", line);
        }
        
        // Draw version info
        writer.set_color(Color::LightGray, Color::Black);
        writer.set_position(35, 20);
        print!("Version 1.0.0");
        
        // Draw loading animation
        let dots = (self.frame / 10) % 4;
        writer.set_position(35, 21);
        print!("Loading{}", ".".repeat(dots as usize));
    }
}

lazy_static! {
    pub static ref SPLASH_SCREEN: Mutex<SplashScreen> = Mutex::new(SplashScreen::new());
} 