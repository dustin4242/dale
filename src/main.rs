use console::Term;
use std::{
    env, fs,
    io::{Error, Write},
};

struct Screen {
    line: usize,
    pos: usize,
    line_top: usize,
    line_bottom: usize,
}

fn main() -> Result<(), Error> {
    let file_path = format!(
        "{}/{}",
        env::current_dir()?.to_str().unwrap(),
        env::args().nth(1).expect("Didn't Supply A File To Edit")
    );
    let mut file: Vec<String> = fs::read_to_string(file_path.to_owned())
        .expect("File Supplied Doesnt Exist")
        .split("\n")
        .map(|x| x.to_string())
        .collect();

    let mut term = Term::stdout();
    let mut screen = Screen {
        line: 0,
        pos: file[0].len(),
        line_top: 0,
        line_bottom: if file.len() < term.size().0 as usize {
            file.len()
        } else {
            term.size().0 as usize
        },
    };
    write_screen(&mut term, &screen, &file);
    loop {
        match term.read_key()? {
            console::Key::UnknownEscSeq(x) => match x[0].to_string().as_str() {
                "s" => fs::write(&file_path, file.join("\n"))
                    .expect("Was Unable To Save File Contents"),
                _ => (),
            },
            console::Key::Char(x) => add_char(&mut screen, &mut file, x),
            console::Key::Backspace => remove_char(&mut screen, &mut file),
            console::Key::Enter => create_newline(&mut screen, &mut file),
            console::Key::Tab => {
                file[screen.line].push_str("    ");
                screen.pos += 4;
            }
            console::Key::ArrowUp => {
                if screen.line != 0 {
                    screen.line -= 1;
                    if file[screen.line].len() < screen.pos {
                        screen.pos = file[screen.line].len();
                    }
                }
            }
            console::Key::ArrowDown => {
                if screen.line + 1 < file.len() {
                    screen.line += 1;
                    if file[screen.line].len() < screen.pos {
                        screen.pos = file[screen.line].len();
                    }
                }
            }
            console::Key::ArrowRight => {
                if screen.pos != file[screen.line].len() {
                    screen.pos += 1;
                }
            }
            console::Key::ArrowLeft => {
                if screen.pos != 0 {
                    screen.pos -= 1;
                }
            }
            console::Key::Escape => {
                term.clear_screen()?;
                return Ok(());
            }
            _ => (),
        }
        if term.size().0 as usize >= file.len() {
            screen.line_top = 0;
            screen.line_bottom = file.len();
        }
        match (
            screen.line_top > screen.line,
            screen.line_bottom <= screen.line,
        ) {
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
        write_screen(&mut term, &screen, &file);
    }
}

// Seperated Functions
fn add_char(screen: &mut Screen, file: &mut Vec<String>, char: char) {
    file[screen.line].insert(screen.pos, char);
    screen.pos += 1;
}

fn remove_char(screen: &mut Screen, file: &mut Vec<String>) {
    if screen.line != 0 && screen.pos == 0 {
        let current_line = file[screen.line].clone();
        screen.pos = file[screen.line - 1].len();
        file[screen.line - 1].push_str(current_line.as_str());
        file.remove(screen.line);
        screen.line -= 1;
    } else {
        file[screen.line].remove(screen.pos - 1);
        screen.pos -= 1;
    }
}

fn create_newline(screen: &mut Screen, file: &mut Vec<String>) {
    let current_line = file[screen.line].clone();
    let new_lines = current_line.split_at(screen.pos);
    file[screen.line] = new_lines.0.to_string();
    file.insert(screen.line + 1, new_lines.1.to_string());
    screen.line += 1;
    screen.pos = 0;
}

fn write_screen(term: &mut Term, screen: &Screen, file: &Vec<String>) {
    term.clear_screen().unwrap();
    term.write_all(
        file[screen.line_top..screen.line_bottom]
            .join("\n")
            .as_bytes(),
    )
    .unwrap();
    term.move_cursor_to(screen.pos, screen.line - screen.line_top)
        .unwrap();
}
