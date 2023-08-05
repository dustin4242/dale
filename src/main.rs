use console::Term;
use std::{env, fs};

mod screen;
use screen::Screen;

fn main() {
    let temp_path = env::args().nth(1).expect("Didn't Supply A File To Edit");
    let isrootdir = temp_path.starts_with("/");
    let ishomedir = temp_path.starts_with("~");
    let file_path = if isrootdir {
        temp_path
    } else if ishomedir {
        format!(
            "{}/{}",
            env::var("HOME").expect("You Do Not Have A HOME Path Set In Env"),
            temp_path.get(1..).unwrap(),
        )
    } else {
        format!(
            "{}/{}",
            env::current_dir().unwrap().to_str().unwrap(),
            temp_path
        )
    };
    let mut file: Vec<String> = fs::read_to_string(file_path.to_owned())
        .expect(format!("File Supplied Doesnt Exist: {}", file_path).as_str())
        .replace("\t", "    ")
        .split("\n")
        .map(|x| x.to_string())
        .collect();
    file.pop();

    let mut term = Term::stdout();
    let mut screen = Screen::new(
        match file.len() < term.size().0 as usize - 1 {
            true => file.len(),
            false => term.size().0 as usize - 1,
        },
        file_path.to_owned(),
    );
    screen.write_term(&mut term, &file);
    loop {
        if term.size().0 as usize >= file.len() {
            screen.line_top = 0;
            screen.line_bottom = file.len();
        }
        screen.write_term(&mut term, &file);
        screen.handle_key(&mut term, &mut file, &file_path);
    }
}
