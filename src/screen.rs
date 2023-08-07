use console::Term;
use regex::Regex;
use std::{fs, io::Write, process::exit};

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
        file[self.line].insert(self.pos, char);
        self.pos += 1;
    }
    pub fn remove_char(&mut self, file: &mut Vec<String>) {
        if self.line != 0 && self.pos == 0 {
            let current_line = file[self.line].clone();
            self.pos = file[self.line - 1].len();
            file[self.line - 1].push_str(current_line.as_str());
            file.remove(self.line);
            self.line -= 1;
        } else {
            file[self.line].remove(self.pos - 1);
            self.pos -= 1;
        }
    }
    pub fn newline(&mut self, file: &mut Vec<String>) {
        let current_line = file[self.line].clone();
        let new_lines = current_line.split_at(self.pos);
        file[self.line] = new_lines.0.to_string();
        file.insert(self.line + 1, new_lines.1.to_string());
        self.line += 1;
        self.pos = 0;
    }
    pub fn write_term(&mut self, term: &mut Term, file: &Vec<String>) {
        let size = term.size();
        term.clear_screen().unwrap();
        let re = Regex::new(r"\buse\b").unwrap();
        let print_file = format!("\n{}", file[self.line_top..self.line_bottom].join("\n"));
        term.write_all(
            re.replace_all(&print_file, "\x1b[34muse\x1b[37m")
                .as_bytes(),
        )
        .unwrap();
        let rest_of_screen = (size.1 as usize).checked_sub(self.info_line.len()).unwrap();
        term.move_cursor_to(0, (size.0 - 1) as usize).unwrap();
        term.write_all(
            format!(
                "\x1b[41m\x1b[30m\n{}{}\x1b[37m\x1b[40m",
                self.info_line,
                " ".repeat(rest_of_screen)
            )
            .as_bytes(),
        )
        .unwrap();
        term.move_cursor_to(self.pos, self.line - self.line_top)
            .unwrap();
    }
    pub fn handle_key(&mut self, term: &mut Term, file: &mut Vec<String>, file_path: &String) {
        match term.read_key().unwrap() {
            console::Key::UnknownEscSeq(x) => match x[0].to_string().as_str() {
                "s" => match fs::write(file_path, file.join("\n")) {
                    Err(_) => self.info_line = "Unable To Save Contents".to_owned(),
                    Ok(_) => self.info_line = "Saved Contents".to_owned(),
                },
                x => panic!("{x}"),
            },
            console::Key::Char(x) => self.add_char(file, x),
            console::Key::Backspace => self.remove_char(file),
            console::Key::Enter => self.newline(file),
            console::Key::Tab => {
                file[self.line].push_str("    ");
                self.pos += 4;
            }
            console::Key::ArrowUp => {
                if self.line != 0 {
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
            console::Key::ArrowDown => {
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
            console::Key::ArrowRight => {
                if self.pos != file[self.line].len() {
                    self.pos += 1;
                }
            }
            console::Key::ArrowLeft => {
                if self.pos != 0 {
                    self.pos -= 1;
                }
            }
            console::Key::Escape => {
                term.clear_screen().unwrap();
                exit(0);
            }
            _ => (),
        }
    }
}
