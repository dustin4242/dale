use std::io::stdout;
use std::{fs, process::exit};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use crossterm::{cursor, event, execute, style, terminal};
use crossterm::{event::Event, terminal::ClearType::All};

pub struct Screen {
    pub line: usize,
    pub pos: usize,
    pub line_top: usize,
    pub line_bottom: usize,
    pub info_line: String,
}

impl Screen {
    pub fn new(line_bottom: usize, info_line: String) -> Screen {
        Screen {
            line: 0,
            pos: 0,
            line_top: 0,
            line_bottom,
            info_line,
        }
    }

    pub fn add_char(&mut self, file: &mut Vec<String>, char: char) {
        file.get_mut(self.line).unwrap().insert(self.pos, char);
        self.pos += 1;
    }

    pub fn remove_char(&mut self, file: &mut Vec<String>) {
        match (self.line != 0, self.pos != 0) {
            (false, false) => (),
            (true, false) => {
                let current_line = file[self.line].clone();
                self.pos = file[self.line - 1].len();
                file[self.line - 1].push_str(current_line.as_str());
                file.remove(self.line);
                self.line -= 1;
                if self.line_bottom > file.len() {
                    self.line_bottom -= 1;
                    self.line_top -= if self.line_top != 0 { 1 } else { 0 };
                }
            }
            (_, _) => {
                file[self.line].remove(self.pos - 1);
                self.pos -= 1;
            }
        }
    }

    pub fn newline(&mut self, file: &mut Vec<String>) {
        let current_line = file[self.line].clone();
        let new_lines = current_line.split_at(self.pos);
        file[self.line] = new_lines.0.to_string();
        file.insert(self.line + 1, new_lines.1.to_string());
        self.line += 1;
        self.pos = 0;
        if self.line == self.line_bottom {
            self.line_top += 1;
            self.line_bottom += 1;
        }
    }

    pub fn write_term(&mut self, file: &Vec<String>) {
        let mut stdout = stdout();
        let size = terminal::size().unwrap();
        let print_file = if self.line_bottom < file.len() {
            format!("\n{}", file[self.line_top..self.line_bottom].join("\n"))
        } else {
            self.line_top -= if self.line_top >= self.line_bottom - file.len() {
                self.line_bottom - file.len()
            } else {
                self.line_top
            };
            self.line_bottom = file.len();
            format!("\n{}", file[self.line_top..self.line_bottom].join("\n"))
        };
        let rest_of_screen = (size.0 as usize)
            .checked_sub(self.info_line.len())
            .expect("Can't Get InfoLine To End Of Screen");
        execute!(
            stdout,
            terminal::Clear(All),
            cursor::MoveTo(0, 0),
            style::Print(format!("\x1b[\x35 q{print_file}")),
            cursor::MoveTo(0, size.1 - 1),
            style::Print(format!(
                "\x1b[44m\x1b[30m\n{}{}\x1b[37m\x1b[40m",
                self.info_line,
                " ".repeat(rest_of_screen)
            )),
            cursor::MoveTo(self.pos as u16, (self.line - self.line_top) as u16)
        )
        .unwrap();
    }

    pub fn handle_event(&mut self, file: &mut Vec<String>, file_path: &String) {
        terminal::enable_raw_mode().unwrap();
        match event::read().expect("Unable To Read Events") {
            Event::Key(KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => match fs::write(file_path, format!("{}\n", file.join("\n"))) {
                Err(_) => self.info_line = "Unable To Save Contents".to_owned(),
                Ok(_) => self.info_line = "Saved Contents".to_owned(),
            },
            Event::Key(key) => self.handle_key(key.code, file, file_path),
            Event::Resize(_, y) => {
                self.line_bottom = self.line_top + y as usize - 1;
                if self.line >= self.line_bottom {
                    self.line = self.line_bottom - 1;
                }
            }
            x => todo!("unknown event: {x:?}"),
        }
        terminal::disable_raw_mode().unwrap();
    }

    pub fn handle_key(&mut self, key: event::KeyCode, file: &mut Vec<String>, _file_path: &String) {
        match key {
            KeyCode::Char(c) => self.add_char(file, c),
            KeyCode::Backspace => self.remove_char(file),
            KeyCode::Enter => self.newline(file),
            KeyCode::Tab => {
                file[self.line].push_str("    ");
                self.pos += 4;
            }
            KeyCode::Up => {
                if self.line > 0 {
                    self.line -= 1;
                    if file[self.line].len() < self.pos {
                        self.pos = file[self.line].len();
                    }
                    if self.line_top > self.line {
                        self.line_top -= 1;
                        self.line_bottom -= 1;
                    }
                }
            }
            KeyCode::Down => {
                if self.line + 1 < file.len() {
                    self.line += 1;
                    if file[self.line].len() < self.pos {
                        self.pos = file[self.line].len();
                    }
                    if self.line_bottom - 1 < self.line {
                        self.line_top += 1;
                        self.line_bottom += 1;
                    }
                }
            }
            KeyCode::Right => {
                if self.pos < file[self.line].len() {
                    self.pos += 1;
                }
            }
            KeyCode::Left => {
                if self.pos != 0 {
                    self.pos -= 1;
                }
            }
            KeyCode::Esc => {
                execute!(stdout(), terminal::Clear(All)).unwrap();
                exit(0);
            }
            _ => (),
        }
    }
}
