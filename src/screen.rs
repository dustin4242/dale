use std::char;
use std::io::stdout;
use std::process::Command;
use std::{fs, process::exit};

use crate::syntax::Syntax;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use crossterm::{cursor, event, execute, style, terminal};
use crossterm::{event::Event, terminal::ClearType::All};

pub struct Screen {
    pub line: usize,
    pub pos: usize,
    pub line_top: usize,
    pub line_bottom: usize,
    pub info_line: String,
    pub changed: bool,
    pub screen_update: bool,
    pub syntax: Option<Syntax>,
}

impl Screen {
    pub fn new(line_bottom: usize, info_line: String) -> Screen {
        Screen {
            line: 0,
            pos: 0,
            line_top: 0,
            line_bottom,
            info_line,
            changed: false,
            screen_update: true,
            syntax: None,
        }
    }

    fn add_char(&mut self, file: &mut Vec<String>, char: char) {
        file.get_mut(self.line).unwrap().insert(self.pos, char);
        self.pos += 1;
        self.changed = true;
        self.screen_update = true;
    }

    fn remove_char(&mut self, file: &mut Vec<String>) {
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
                self.changed = true;
                self.screen_update = true;
            }
            (_, true) => {
                file[self.line].remove(self.pos - 1);
                self.pos -= 1;
                self.changed = true;
                self.screen_update = true;
            }
        }
    }

    fn newline(&mut self, file: &mut Vec<String>) {
        let line_length = file[self.line].len();
        if self.pos != line_length {
            let current_line = file[self.line].clone();
            let new_lines = current_line.split_at(self.pos);
            file[self.line] = new_lines.0.to_string();
            file.insert(self.line + 1, new_lines.1.to_string());
        } else if self.pos == 0 {
            file.insert(self.line, String::new());
        } else {
            file.insert(self.line + 1, String::new());
        }
        self.line += 1;
        self.pos = 0;
        if self.line == self.line_bottom {
            self.line_top += 1;
            self.line_bottom += 1;
        }
        self.changed = true;
        self.screen_update = true;
    }

    pub fn write_term(&mut self, file: &Vec<String>) -> Option<()> {
        if self.screen_update == true {
            let mut print_file = if self.line_bottom < file.len() {
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
            if self.syntax.is_some() {
                print_file = crate::syntax::highlight(self.syntax.as_ref()?, print_file)?;
            }
            execute!(
                stdout(),
                terminal::Clear(All),
                cursor::MoveTo(0, 0),
                style::Print(format!("\x1b[\x35 q{print_file}\n")),
            )
            .unwrap();
            write_infoline(format!("{}", self.info_line.to_owned()));
            execute!(
                stdout(),
                cursor::MoveTo(self.pos as u16, (self.line - self.line_top) as u16)
            )
            .unwrap();
            self.screen_update = false;
        }
        Some(())
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
                Ok(_) => {
                    self.changed = false;
                    self.info_line = "Saved Contents".to_owned()
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char('r'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.command_handler().unwrap(),
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
        use KeyCode as KC;
        match key {
            KC::Char(c) => self.add_char(file, c),
            KC::Backspace => self.remove_char(file),
            KC::Enter => self.newline(file),
            KC::Tab => {
                file[self.line].push_str("    ");
                self.pos += 4;
                self.screen_update = true;
            }
            KC::Up => {
                if self.line > 0 {
                    self.line -= 1;
                    if file[self.line].len() < self.pos {
                        self.pos = file[self.line].len();
                    }
                    if self.line_top > self.line {
                        self.line_top -= 1;
                        self.line_bottom -= 1;
                    }
                    self.screen_update = true;
                }
            }
            KC::Down => {
                if self.line + 1 < file.len() {
                    self.line += 1;
                    if file[self.line].len() < self.pos {
                        self.pos = file[self.line].len();
                    }
                    if self.line_bottom - 1 < self.line {
                        self.line_top += 1;
                        self.line_bottom += 1;
                    }
                    self.screen_update = true;
                }
            }
            KC::Right => {
                if self.pos < file[self.line].len() {
                    self.pos += 1;
                    self.screen_update = true;
                }
            }
            KC::Left => {
                if self.pos != 0 {
                    self.pos -= 1;
                    self.screen_update = true;
                }
            }
            KC::Esc => {
                if self.changed == true {
                    let output = "Unsaved Changes are you sure you want to exit? (y/n): ";
                    let mut answer = String::new();
                    write_infoline(format!("{output}{}", answer.to_owned()));
                    loop {
                        match event::read().expect("Unable To Read Events") {
                            Event::Key(key) => match key.code {
                                KeyCode::Char(c) => {
                                    answer.push(c);
                                    write_infoline(format!("{output}{}", answer.to_owned()));
                                }
                                KeyCode::Backspace => {
                                    answer.pop();
                                    write_infoline(format!("{output}{}", answer.to_owned()));
                                }
                                KeyCode::Esc => return,
                                KeyCode::Enter => break,
                                _ => (),
                            },
                            _ => (),
                        }
                    }
                    match answer.to_lowercase().as_str() {
                        "y" | "yes" => (),
                        _ => {
                            self.screen_update = true;
                            return;
                        }
                    }
                }
                execute!(stdout(), terminal::Clear(All), cursor::MoveTo(0, 0)).unwrap();
                terminal::disable_raw_mode().unwrap();
                exit(0);
            }
            _ => (),
        }
    }
    pub fn command_handler(&mut self) -> Result<(), std::io::Error> {
        let mut stdout = stdout();
        let mut command = String::new();
        self.screen_update = true;
        write_infoline("Command: ".to_string());
        loop {
            match event::read().expect("Unable To Read Events") {
                Event::Key(key) => match key.code {
                    KeyCode::Char(c) => {
                        command.push(c);
                        write_infoline(format!("Command: {}", command.to_owned()));
                    }
                    KeyCode::Backspace => {
                        command.pop();
                        write_infoline(format!("Command: {}", command.to_owned()));
                    }
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Enter => break,
                    _ => (),
                },
                _ => (),
            }
        }
        if command != "".to_owned() {
            execute!(stdout, terminal::Clear(All), cursor::MoveTo(0, 0))?;
            let mut command_args: Vec<&str> = command.split(" ").collect();
            terminal::disable_raw_mode()?;
            match Command::new(command_args.remove(0))
                .args(command_args)
                .spawn()
            {
                Ok(mut x) => {
                    x.wait()?;
                    execute!(stdout, style::Print("Press ESC to return to editor."))?;
                    loop {
                        terminal::enable_raw_mode()?;
                        match event::read().expect("Unable To Read Events") {
                            Event::Key(key) => match key.code {
                                KeyCode::Esc => break,
                                _ => (),
                            },
                            _ => (),
                        }
                    }
                }
                Err(x) => self.info_line = x.to_string(),
            }
        }
        Ok(())
    }
}
fn write_infoline(info: String) {
    let size = terminal::size().unwrap();
    let rest_of_screen = size.0.checked_sub((info.len()) as u16).unwrap_or(0) as usize;
    let infoline_color = "\x1b[44m";
    execute!(
        stdout(),
        cursor::MoveTo(0, size.1 - 1),
        style::Print(format!(
            "{infoline_color}\x1b[30m{info}{}\x1b[37m\x1b[40m",
            " ".repeat(rest_of_screen)
        )),
        cursor::MoveTo(info.len() as u16, size.1)
    )
    .unwrap();
}
