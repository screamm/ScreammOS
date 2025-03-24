use core::fmt::{self, Write};
use spin::Mutex;
use x86_64::instructions::interrupts;
use crate::simple_fs::SimpleString;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

pub static WRITER: Mutex<Writer> = Mutex::new(Writer {
    buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    x: 0,
    y: 0,
    color_code: ColorCode::new(Color::White, Color::Black),
});

#[macro_export]
macro_rules! _print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

pub fn _format(args: fmt::Arguments) -> SimpleString {
    let mut s = SimpleString::new();
    s.write_fmt(args).unwrap();
    s
}

#[derive(Debug, Clone)]
pub struct SimpleString {
    data: [u8; 256],
    len: usize,
}

impl SimpleString {
    pub fn new() -> Self {
        SimpleString {
            data: [0; 256],
            len: 0,
        }
    }

    pub fn push_str(&mut self, s: &str) {
        for &byte in s.as_bytes() {
            if self.len < 256 {
                self.data[self.len] = byte;
                self.len += 1;
            }
        }
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.data[..self.len]).unwrap_or("")
    }
}

impl Write for SimpleString {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    buffer: &'static mut Buffer,
    x: usize,
    y: usize,
    color_code: ColorCode,
}

impl Writer {
    pub fn set_position(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
        self.update_cursor();
    }

    pub fn write_char_at(&mut self, x: usize, y: usize, c: char, fg: Color, bg: Color) {
        if x >= BUFFER_WIDTH || y >= BUFFER_HEIGHT {
            return;
        }

        let color_code = ColorCode::new(fg, bg);
        self.buffer.chars[y][x] = ScreenChar {
            ascii_character: c as u8,
            color_code,
        };
    }

    pub fn set_cursor_position(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
        self.update_cursor();
    }

    pub fn update_cursor(&mut self) {
        // Implementation of update_cursor method
    }
} 