use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

// Standardfärger för VGA-text
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

// ColorCode representerar färgattribut för VGA-text (förgrund/bakgrund)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

// Ett tecken på skärmen med färgattribut
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

// Storleken på VGA-textbufferten
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// VGA-textbuffert med flyktiga läsningar/skrivningar
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// Writer-struktur för att hantera textutmatning
pub struct Writer {
    pub column_position: usize,
    pub row_position: usize,
    pub color_code: ColorCode,
    buffer: &'static mut Buffer,
    pub crt_effect_enabled: bool,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                });

                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        // DOS-stil scrollning
        if self.row_position >= BUFFER_HEIGHT - 1 {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(character);
                }
            }
            self.clear_row(BUFFER_HEIGHT - 1);
        } else {
            self.row_position += 1;
        }
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Skrivbara ASCII-tecken eller radbrytning
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Icke i ASCII-området använder ett ersättningstecken (DOS-stil)
                _ => self.write_byte(0xFE),
            }
        }
    }
    
    // Ställ in textfärg i DOS-stil
    pub fn set_color(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }
    
    // Aktivera/inaktivera CRT-effekt för retrokänsla
    pub fn set_crt_effect(&mut self, enabled: bool) {
        self.crt_effect_enabled = enabled;
        // Implementering av faktisk CRT-effekt kommer senare
    }
    
    // Rensa skärmen (som CLS i DOS)
    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.column_position = 0;
        self.row_position = 0;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// Global Writer-instans med Mutex för säker global åtkomst
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row_position: 0,
        color_code: ColorCode::new(Color::LightGray, Color::Blue), // Klassisk DOS-blå
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        crt_effect_enabled: false,
    });
}

// Makron för att förenkla utskrift
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    
    // Inaktivera avbrott under skrivning för att undvika race conditions
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

// Ändra tema för DOS-känsla
pub fn change_theme(theme_style: ThemeStyle) {
    match theme_style {
        ThemeStyle::DOSClassic => {
            WRITER.lock().set_color(Color::LightGray, Color::Blue);
            WRITER.lock().set_crt_effect(false);
        },
        ThemeStyle::AmberTerminal => {
            WRITER.lock().set_color(Color::Brown, Color::Black);
            WRITER.lock().set_crt_effect(true);
        },
        ThemeStyle::GreenCRT => {
            WRITER.lock().set_color(Color::Green, Color::Black);
            WRITER.lock().set_crt_effect(true);
        },
        ThemeStyle::Modern => {
            WRITER.lock().set_color(Color::White, Color::DarkGray);
            WRITER.lock().set_crt_effect(false);
        },
    }
}

// Enkla temastilar för att komma igång
pub enum ThemeStyle {
    DOSClassic,     // Klassisk DOS-stil (blå bakgrund, ljusgrå text)
    AmberTerminal,  // Amber terminal (gul/brun text på svart)
    GreenCRT,       // Grön CRT terminal (grön text på svart)
    Modern,         // Modern tolkning (vit text på mörkgrå)
} 