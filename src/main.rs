use console::Term;
use std::{
    env, fs,
    io::{Error, Write},
    process::exit,
};

struct Screen {
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

    let (mut line, mut pos) = (0, file[0].len());
    let mut term = Term::stdout();
    let mut screen = Screen {
        line_top: 0,
        line_bottom: if file.len() < term.size().0 as usize {
            file.len()
        } else {
            term.size().0 as usize
        },
    };
    write_screen((&mut term, &screen), (line, pos), &file)?;
    loop {
        match term.read_key() {
            Ok(console::Key::Char(x)) => {
                file[line].insert(pos, x);
                pos += 1;
            }
            Ok(console::Key::Backspace) => {
                if line != 0 && pos == 0 {
                    pos = file[line - 1].len();
                    let currentline = file[line].clone();
                    file[line - 1].push_str(currentline.as_str());
                    file.remove(line);
                    line -= 1;
                } else {
                    file[line].remove(pos - 1);
                    pos -= 1;
                }
            }
            Ok(console::Key::Enter) => {
                let currentline = file[line].clone();
                let newlines = currentline.split_at(pos);
                file[line] = newlines.0.to_string();
                file.insert(line + 1, newlines.1.to_string());
                line += 1;
                pos = 0;
            }
            Ok(console::Key::Tab) => {
                file[line].push_str("    ");
                pos += 4;
            }
            Ok(console::Key::ArrowUp) => {
                if line != 0 {
                    line -= 1;
                    if file[line].len() < pos {
                        pos = file[line].len();
                    }
                }
            }
            Ok(console::Key::ArrowDown) => {
                if line + 1 < file.len() {
                    line += 1;
                    if file[line].len() < pos {
                        pos = file[line].len();
                    }
                }
            }
            Ok(console::Key::ArrowRight) => {
                if pos != file[line].len() {
                    pos += 1;
                }
            }
            Ok(console::Key::ArrowLeft) => {
                if pos != 0 {
                    pos -= 1;
                }
            }
            Ok(console::Key::Escape) => {
                fs::write(file_path, file.join("\n")).expect("Was Unable To Save File Contents");
                exit(0)
            }
            Err(x) => panic!("{}", x),
            _ => (),
        }
        if term.size().0 as usize >= file.len() {
            screen.line_top = 0;
            screen.line_bottom = file.len();
        }
        match (screen.line_top > line, screen.line_bottom <= line) {
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
        write_screen((&mut term, &screen), (line, pos), &file)?;
    }
}

// Seperated Functions
fn write_screen(
    (term, screen): (&mut Term, &Screen),
    (line, pos): (usize, usize),
    file: &Vec<String>,
) -> Result<(), Error> {
    term.clear_screen()?;
    term.write_all(
        file[screen.line_top..screen.line_bottom]
            .join("\n")
            .as_bytes(),
    )?;
    term.move_cursor_to(pos, line - screen.line_top)?;
    Ok(())
}
