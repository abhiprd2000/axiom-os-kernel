use crate::vga_buffer::{self, Color};
use crate::{print, println};
use alloc::string::String;
use alloc::vec::Vec;

pub struct Editor {
    filename: String,
    lines: Vec<String>,
    current_line: String,
    modified: bool,
}

impl Editor {
    pub fn new(filename: &str) -> Self {
        vga_buffer::clear_screen();
        vga_buffer::println_colored("=== AXIOM TEXT EDITOR ===", Color::Yellow, Color::Black);
        vga_buffer::println_colored(
            &alloc::format!("File: {}", filename),
            Color::LightCyan,
            Color::Black,
        );
        vga_buffer::println_colored(
            "Ctrl+S = save  Ctrl+Q = quit  Enter = new line",
            Color::LightGray,
            Color::Black,
        );
        println!("─────────────────────────────────────────");
        println!("");

        Editor {
            filename: String::from(filename),
            lines: Vec::new(),
            current_line: String::new(),
            modified: false,
        }
    }

    pub fn handle_char(&mut self, c: char) -> bool {
        match c {
            '\x11' => {
                // Ctrl+Q
                if self.modified {
                    vga_buffer::println_colored(
                        "[editor] unsaved changes — press Ctrl+S to save or Ctrl+Q again to quit",
                        Color::LightRed,
                        Color::Black,
                    );
                    self.modified = false; // second Ctrl+Q exits
                    return false;
                }
                return true; // quit
            }
            '\x13' => {
                // Ctrl+S
                self.save();
                return false;
            }
            '\n' | '\r' => {
                self.lines.push(self.current_line.clone());
                self.current_line.clear();
                self.modified = true;
                println!("");
                print!("  ");
            }
            '\x08' => {
                // Backspace
                if !self.current_line.is_empty() {
                    self.current_line.pop();
                    crate::vga_buffer::WRITER.lock().backspace();
                    self.modified = true;
                }
            }
            c if c as u8 >= 32 => {
                self.current_line.push(c);
                print!("{}", c);
                self.modified = true;
            }
            _ => {}
        }
        false
    }

    fn save(&mut self) {
        // Flush current line
        if !self.current_line.is_empty() {
            self.lines.push(self.current_line.clone());
            self.current_line.clear();
        }
        // Build full content
        let mut content = String::new();
        for line in &self.lines {
            content.push_str(line);
            content.push('\n');
        }
        // Save to VFS with provenance
        crate::shell::VFS
            .lock()
            .create(&self.filename, content.as_bytes());
        // Save to ATA disk
        let saved = crate::ata::write_sector(2, &{
            let mut sector = [0u8; 512];
            let bytes = content.as_bytes();
            let len = bytes.len().min(500);
            let name = self.filename.as_bytes();
            let nlen = name.len().min(11) as u8;
            sector[0] = nlen;
            sector[1..1 + nlen as usize].copy_from_slice(&name[..nlen as usize]);
            sector[12] = len as u8;
            sector[13..13 + len].copy_from_slice(&bytes[..len]);
            sector
        });
        if saved {
            vga_buffer::println_colored(
                &alloc::format!(
                    "\n[editor] SAVED: {} ({} bytes) to disk",
                    self.filename,
                    content.len()
                ),
                Color::LightGreen,
                Color::Black,
            );
        } else {
            vga_buffer::println_colored(
                "\n[editor] saved to VFS only (no persistent disk)",
                Color::Yellow,
                Color::Black,
            );
        }
        self.modified = false;
    }
}
