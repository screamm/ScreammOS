// src/ui/retro_commands.rs
// Retro commands and themes for ScreammOS

use crate::vga_buffer::{Color, WRITER, change_theme, ThemeStyle};
use crate::simple_fs::{FILESYSTEM, SimpleString, SimpleFileSystem, FileType};
use crate::{print, println};
use alloc::vec::Vec;
use crate::ui::text_editor::TEXT_EDITOR;
use crate::string_ext::{StringExt, StringSliceExt};

// Retro color themes
pub enum RetroTheme {
    DOSClassic,    // Light gray on black
    CGA,           // Cyan, magenta, white, black
    EGA,           // 16 colors
    VGA,           // 256 colors
    Monochrome,    // White on black
}

impl RetroTheme {
    pub fn apply(&self) {
        let mut writer = WRITER.lock();
        match self {
            RetroTheme::DOSClassic => {
                writer.set_color(Color::LightGray, Color::Black);
            },
            RetroTheme::CGA => {
                writer.set_color(Color::Cyan, Color::Black);
            },
            RetroTheme::EGA => {
                writer.set_color(Color::LightGreen, Color::Black);
            },
            RetroTheme::VGA => {
                writer.set_color(Color::White, Color::Blue);
            },
            RetroTheme::Monochrome => {
                writer.set_color(Color::White, Color::Black);
            },
        }
    }
}

// Retro command structure
pub struct RetroCommand {
    name: &'static str,
    description: &'static str,
    usage: &'static str,
    handler: fn(&[&str]) -> Result<(), &'static str>,
}

// Retro commands implementation
pub fn get_retro_commands() -> Vec<RetroCommand> {
    vec![
        RetroCommand {
            name: "color",
            description: "Change the color scheme",
            usage: "color [theme]",
            handler: cmd_color,
        },
        RetroCommand {
            name: "cls",
            description: "Clear the screen",
            usage: "cls",
            handler: cmd_cls,
        },
        RetroCommand {
            name: "dir",
            description: "List directory contents",
            usage: "dir [path]",
            handler: cmd_dir,
        },
        RetroCommand {
            name: "cd",
            description: "Change directory",
            usage: "cd [path]",
            handler: cmd_cd,
        },
        RetroCommand {
            name: "type",
            description: "Display file contents",
            usage: "type [filename]",
            handler: cmd_type,
        },
        RetroCommand {
            name: "echo",
            description: "Display messages",
            usage: "echo [message]",
            handler: cmd_echo,
        },
        RetroCommand {
            name: "date",
            description: "Display or set the date",
            usage: "date",
            handler: cmd_date,
        },
        RetroCommand {
            name: "time",
            description: "Display or set the time",
            usage: "time",
            handler: cmd_time,
        },
        RetroCommand {
            name: "ver",
            description: "Display version information",
            usage: "ver",
            handler: cmd_ver,
        },
        RetroCommand {
            name: "help",
            description: "Display help information",
            usage: "help [command]",
            handler: cmd_help,
        },
    ]
}

// Command handlers
fn cmd_color(args: &[&str]) -> Result<(), &'static str> {
    if args.is_empty() {
        println!("Available themes:");
        println!("  DOSClassic - Classic DOS look");
        println!("  CGA        - CGA color scheme");
        println!("  EGA        - EGA 16 colors");
        println!("  VGA        - VGA 256 colors");
        println!("  Monochrome - Monochrome display");
        return Ok(());
    }

    match args[0].to_uppercase().as_str() {
        "DOSCLASSIC" => {
            change_theme(ThemeStyle::DOSClassic);
            println!("Theme changed to DOS Classic");
        },
        "CGA" => {
            RetroTheme::CGA.apply();
            println!("Theme changed to CGA");
        },
        "EGA" => {
            RetroTheme::EGA.apply();
            println!("Theme changed to EGA");
        },
        "VGA" => {
            RetroTheme::VGA.apply();
            println!("Theme changed to VGA");
        },
        "MONOCHROME" => {
            RetroTheme::Monochrome.apply();
            println!("Theme changed to Monochrome");
        },
        _ => return Err("Invalid theme. Use 'color' to see available themes."),
    }
    Ok(())
}

fn cmd_cls(_args: &[&str]) -> Result<(), &'static str> {
    let mut writer = WRITER.lock();
    writer.clear_screen();
    Ok(())
}

fn cmd_dir(args: &[&str]) -> Result<(), &'static str> {
    let path = if args.is_empty() { "." } else { args[0] };
    let fs = FILESYSTEM.lock();
    
    println!(" Directory of {}", path);
    println!("\n");
    
    for (file_type, name, size) in fs.list_directory() {
        let type_str = match file_type {
            FileType::Directory => "<DIR>",
            FileType::Regular => "     ",
            FileType::File => "FILE",
            FileType::Symlink => "LINK",
        };
        println!("{:5} {:20} {:10}", type_str, name, size);
    }
    
    Ok(())
}

fn cmd_cd(args: &[&str]) -> Result<(), &'static str> {
    if args.is_empty() {
        let fs = FILESYSTEM.lock();
        println!("Current directory: {}", fs.get_current_directory());
        return Ok(());
    }
    
    let mut fs = FILESYSTEM.lock();
    match fs.change_directory(args[0]) {
        Ok(_) => println!("Directory changed to: {}", fs.get_current_directory()),
        Err(e) => return Err(e),
    }
    Ok(())
}

fn cmd_type(args: &[&str]) -> Result<(), &'static str> {
    if args.is_empty() {
        return Err("Usage: type [filename]");
    }
    
    let fs = FILESYSTEM.lock();
    match fs.read_file(args[0]) {
        Some(content) => println!("{}", content),
        None => return Err("File not found"),
    }
    Ok(())
}

fn cmd_echo(args: &[&str]) -> Result<(), &'static str> {
    println!("{}", args.join(" "));
    Ok(())
}

fn cmd_date(_args: &[&str]) -> Result<(), &'static str> {
    println!("Current date: 2024-03-23");
    Ok(())
}

fn cmd_time(_args: &[&str]) -> Result<(), &'static str> {
    println!("Current time: 12:00:00");
    Ok(())
}

fn cmd_ver(_args: &[&str]) -> Result<(), &'static str> {
    println!("ScreammOS Version 1.0.0");
    println!("Copyright (c) 2024");
    Ok(())
}

fn cmd_help(args: &[&str]) -> Result<(), &'static str> {
    let commands = get_retro_commands();
    
    if args.is_empty() {
        println!("ScreammOS Commands:");
        println!("------------------");
        for cmd in commands {
            println!("{:10} - {}", cmd.name, cmd.description);
        }
        println!("\nFor more information on a specific command, type HELP command-name");
        return Ok(());
    }
    
    let command_name = args[0].to_lowercase();
    for cmd in commands {
        if cmd.name == command_name {
            println!("{}", cmd.name.to_uppercase());
            println!("{}", cmd.description);
            println!("Usage: {}", cmd.usage);
            return Ok(());
        }
    }
    
    Err("Command not found")
}

pub fn handle_retro_command(command: &str, fs: &mut SimpleFileSystem) {
    let args: Vec<&str> = command.split_whitespace().collect();
    if args.is_empty() {
        return;
    }

    match args[0].to_uppercase().as_str() {
        "EDIT" => {
            if args.len() < 2 {
                println!("Usage: EDIT <filename>");
                return;
            }
            let filename = args[1];
            if let Some(file_index) = fs.find_file(filename) {
                let file_type = fs.get_file_type(file_index);
                match file_type {
                    FileType::File => {
                        // TODO: Implementera text editor
                        println!("Editing file: {}", filename);
                    }
                    _ => println!("Cannot edit: {} is not a file", filename),
                }
            } else {
                println!("File not found: {}", filename);
            }
        }
        "VIEW" => {
            if args.len() < 2 {
                println!("Usage: VIEW <filename>");
                return;
            }
            let filename = args[1];
            if let Some(file_index) = fs.find_file(filename) {
                let file_type = fs.get_file_type(file_index);
                match file_type {
                    FileType::File => {
                        if let Some(content) = fs.read_file(filename) {
                            println!("{}", content);
                        } else {
                            println!("Error reading file: {}", filename);
                        }
                    }
                    _ => println!("Cannot view: {} is not a file", filename),
                }
            } else {
                println!("File not found: {}", filename);
            }
        }
        "CAT" => {
            if args.len() < 2 {
                println!("Usage: CAT <filename>");
                return;
            }
            let filename = args[1];
            if let Some(file_index) = fs.find_file(filename) {
                let file_type = fs.get_file_type(file_index);
                match file_type {
                    FileType::File => {
                        if let Some(content) = fs.read_file(filename) {
                            print!("{}", content);
                        } else {
                            println!("Error reading file: {}", filename);
                        }
                    }
                    _ => println!("Cannot cat: {} is not a file", filename),
                }
            } else {
                println!("File not found: {}", filename);
            }
        }
        "LS" => {
            let mut current_dir = SimpleString::new();
            if args.len() > 1 {
                current_dir.push_str(args[1]);
            } else {
                current_dir.push_str("/");
            }

            let mut found = false;
            for i in 0..fs.get_file_count() {
                let filename = fs.get_filename(i);
                let file_type = fs.get_file_type(i);
                let file_size = fs.get_file_size(i);

                let type_str = match file_type {
                    FileType::File => "File",
                    FileType::Directory => "Dir",
                    FileType::Symlink => "Link",
                };

                println!("{:>8} {:>8} {}", file_size, type_str, filename);
                found = true;
            }

            if !found {
                println!("No files found");
            }
        }
        "CD" => {
            if args.len() < 2 {
                println!("Usage: CD <directory>");
                return;
            }
            let dir = args[1];
            if let Some(file_index) = fs.find_file(dir) {
                let file_type = fs.get_file_type(file_index);
                match file_type {
                    FileType::Directory => {
                        // TODO: Implementera CD
                        println!("Changed directory to: {}", dir);
                    }
                    _ => println!("Not a directory: {}", dir),
                }
            } else {
                println!("Directory not found: {}", dir);
            }
        }
        "PWD" => {
            // TODO: Implementera PWD
            println!("/");
        }
        "MKDIR" => {
            if args.len() < 2 {
                println!("Usage: MKDIR <directory>");
                return;
            }
            let dir = args[1];
            if fs.find_file(dir).is_some() {
                println!("Directory already exists: {}", dir);
                return;
            }
            if fs.create_directory(dir) {
                println!("Created directory: {}", dir);
            } else {
                println!("Failed to create directory: {}", dir);
            }
        }
        "RM" => {
            if args.len() < 2 {
                println!("Usage: RM <file>");
                return;
            }
            let file = args[1];
            if fs.delete_file(file) {
                println!("Deleted: {}", file);
            } else {
                println!("Failed to delete: {}", file);
            }
        }
        "CP" => {
            if args.len() < 3 {
                println!("Usage: CP <source> <destination>");
                return;
            }
            let source = args[1];
            let dest = args[2];
            if let Some(content) = fs.read_file(source) {
                if fs.write_file(dest, &content) {
                    println!("Copied {} to {}", source, dest);
                } else {
                    println!("Failed to copy {} to {}", source, dest);
                }
            } else {
                println!("Failed to read source file: {}", source);
            }
        }
        "MV" => {
            if args.len() < 3 {
                println!("Usage: MV <source> <destination>");
                return;
            }
            let source = args[1];
            let dest = args[2];
            if let Some(content) = fs.read_file(source) {
                if fs.write_file(dest, &content) {
                    if fs.delete_file(source) {
                        println!("Moved {} to {}", source, dest);
                    } else {
                        println!("Failed to delete source file: {}", source);
                    }
                } else {
                    println!("Failed to move {} to {}", source, dest);
                }
            } else {
                println!("Failed to read source file: {}", source);
            }
        }
        "HELP" => {
            println!("Available commands:");
            println!("  EDIT <file>     - Edit a file");
            println!("  VIEW <file>     - View a file");
            println!("  CAT <file>      - Display file contents");
            println!("  LS [dir]        - List files and directories");
            println!("  CD <dir>        - Change directory");
            println!("  PWD             - Print working directory");
            println!("  MKDIR <dir>     - Create a directory");
            println!("  RM <file>       - Remove a file");
            println!("  CP <src> <dst>  - Copy a file");
            println!("  MV <src> <dst>  - Move a file");
            println!("  HELP            - Show this help message");
        }
        _ => println!("Unknown command: {}", args[0]),
    }
} 