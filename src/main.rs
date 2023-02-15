use console::Term;
use std::{env, fs, io::Write, process::exit};

struct Screen {
    line_top: usize,
    line_bottom: usize,
    top: bool,
    bottom: bool,
}

fn main() {
    let file_path = format!(
        "{}/{}",
        env::current_dir().unwrap().to_str().unwrap(),
        match env::args().nth(1) {
            Some(x) => x,
            None => panic!("Didn't Supply A File To Edit"),
        }
    );

    let mut file: Vec<String> = match fs::read_to_string(file_path.to_owned()) {
        Ok(x) => x,
        Err(_e) => panic!("File Supplied Doesn't Exist"),
    }
    .split("\n")
    .map(|x| x.to_string())
    .collect();
    let mut term = Term::stdout();
    let mut line = 0;
    let mut pos = file[0].len();
    let mut screen: Screen;
    screen = Screen {
        line_top: 0,
        line_bottom: if file.len() < term.size().0 as usize {
            file.len()
        } else {
            term.size().0 as usize
        },
        top: false,
        bottom: false,
    };
    term.clear_screen().unwrap();
    term.write_all(
        file[screen.line_top..screen.line_bottom]
            .join("\n")
            .as_bytes(),
    )
    .unwrap();
    term.move_cursor_to(pos, line).unwrap();
    loop {
        match term.read_key() {
            Ok(console::Key::Char(x)) => {
                file[line].insert(pos, x);
                pos += 1;
            }
            Ok(x) => match x {
                console::Key::Backspace => {
                    if line != 0 && file[line] == "".to_string() {
                        file.remove(line);
                        line -= 1;
                        pos = file[line].len();
                    } else if file[line] != "".to_string() {
                        file[line].remove(pos - 1);
                        pos -= 1;
                    }
                }
                console::Key::Enter => {
                    line += 1;
                    file.insert(line, "".to_string());
                    pos = 0;
                }
                console::Key::Tab => {
                    file[line].push_str("    ");
                    pos += 4;
                }
                console::Key::ArrowUp => {
                    if line != 0 {
                        line -= 1;
                        pos = file[line].len();
                    }
                }
                console::Key::ArrowDown => {
                    if line + 1 != file.len() {
                        line += 1;
                        pos = file[line].len();
                    }
                }
                console::Key::ArrowRight => {
                    if pos != file[line].len() {
                        pos += 1;
                    }
                }
                console::Key::ArrowLeft => {
                    if pos != 0 {
                        pos -= 1;
                    }
                }
                console::Key::Escape => {
                    fs::write(file_path, file.join("\n"))
                        .expect("Was Unable To Save File Contents");
                    exit(0)
                }
                _ => (),
            },
            Err(x) => panic!("{}", x),
        }
        term.clear_screen().unwrap();
        if screen.line_top > line {
            screen.top = true;
        }
        if screen.line_bottom < line {
            screen.bottom = true;
        }
        match (screen.top, screen.bottom) {
            (true, _) => {
                screen.line_top -= 1;
                screen.line_bottom -= 1;
            }
            (_, true) => {
                screen.line_top += 1;
                screen.line_bottom += 1;
            }
            (false, false) => (),
        };
        if term.size().0 as usize >= file.len() || screen.line_bottom >= file.len() {
            screen.line_top = 0;
            screen.line_bottom = file.len();
            term.write_all(
                file[screen.line_top..screen.line_bottom]
                    .join("\n")
                    .as_bytes(),
            )
            .unwrap();
        } else {
            term.write_all(
                file[screen.line_top + 1..screen.line_bottom + 1]
                    .join("\n")
                    .as_bytes(),
            )
            .unwrap();
        }
        term.move_cursor_to(pos, line - screen.line_top).unwrap();
        screen.top = false;
        screen.bottom = false;
    }
}
