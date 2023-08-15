use std::collections::HashMap;
use std::io::stdout;
use std::{fs, process::exit};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use crossterm::{cursor, event, execute, style, terminal};
use crossterm::{event::Event, terminal::ClearType::All};
use regex::Regex;

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

    fn add_char(&mut self, file: &mut Vec<String>, char: char) {
        file.get_mut(self.line).unwrap().insert(self.pos, char);
        self.pos += 1;
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
            }
            (_, _) => {
                file[self.line].remove(self.pos - 1);
                self.pos -= 1;
            }
        }
    }

    fn newline(&mut self, file: &mut Vec<String>) {
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

    pub fn write_term(&mut self, file: &Vec<String>, plugin: Option<toml::Value>) {
        let mut stdout = stdout();
        let size = terminal::size().unwrap();
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
        print_file = syntax_highlight(plugin, print_file);
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
                execute!(stdout(), terminal::Clear(All), cursor::MoveTo(0, 0)).unwrap();
                exit(0);
            }
            _ => (),
        }
    }
}
fn syntax_highlight(plugin: Option<toml::Value>, mut file: String) -> String {
    let syntax = plugin.unwrap();
    let mut replace_syntax = HashMap::new();
    let basic_table = syntax.get("basic").unwrap();
    let keywords = basic_table.get("keywords").unwrap().as_array().unwrap();
    for key in keywords {
        let replace = key.as_str().unwrap();
        replace_syntax.insert(
            format!(r"\b{replace}\b"),
            format!("\x1b[32m{replace}\x1b[37m"),
        );
    }
    for replace in replace_syntax {
        file = Regex::new(replace.0.as_str())
            .unwrap()
            .replace_all(&file, replace.1)
            .to_string()
    }
    let strings = basic_table.get("strings").unwrap().as_bool().unwrap();
    if strings {
        let quote_regex = Regex::new("\"+[^\"]*\"*").unwrap();
        let temp_file = file.clone();
        let mut quotes = quote_regex.find_iter(&temp_file);
        let mut i = 0;
        loop {
            match quotes.next() {
                Some(find) => {
                    file.insert_str(find.start() + 10 * i, "\x1b[35m");
                    file.insert_str(find.end() + 5 * ((2 * i) + 1), "\x1b[37m");
                    i += 1
                }
                None => break,
            }
        }
    }
    let functions_op = basic_table.get("functions").unwrap().as_bool().unwrap();
    if functions_op {
        let paren_regex = Regex::new(r"[\w\d]+\(+").unwrap();
        let temp_file = file.clone();
        let mut functions = paren_regex.find_iter(&temp_file);
        let mut i = 0;
        loop {
            match functions.next() {
                Some(find) => {
                    file.insert_str(find.start() + 10 * i, "\x1b[36m");
                    file.insert_str(find.end() - 1 + 5 * ((2 * i) + 1), "\x1b[37m");
                    i += 1
                }
                None => break,
            }
        }
    }
    let custom_table = syntax.get("custom").unwrap().as_table().unwrap();
    for syntax in custom_table {
        let custom_regex = Regex::new(syntax.0).unwrap();
        let mut color = syntax.1.as_str().unwrap();
        match color {
            "red" => color = "\x1b[31m",
            "green" => color = "\x1b[32m",
            "yellow" => color = "\x1b[33m",
            "blue" => color = "\x1b[34m",
            "magenta" => color = "\x1b[35m",
            "cyan" => color = "\x1b[36m",
            x => panic!("Unknown Color In Syntax: {}", x),
        }
        let temp_file = file.clone();
        let mut functions = custom_regex.find_iter(&temp_file);
        let mut i = 0;
        loop {
            match functions.next() {
                Some(find) => {
                    file.insert_str(find.start() + 10 * i, color);
                    file.insert_str(find.end() + 5 * ((2 * i) + 1), "\x1b[37m");
                    i += 1
                }
                None => break,
            }
        }
    }
    file
}
