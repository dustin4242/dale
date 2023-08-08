use crossterm::terminal;
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
    let file_name = file_path.split("/").last().unwrap();
    let _plugin = if !file_name.contains(".") {
        let plugin_extension = file_name.split(".").last().unwrap();
        let plugin_file =
            fs::read_to_string(format!("{}.toml", plugin_extension)).unwrap_or("".to_owned());
        toml::from_str(&plugin_file).expect(
            format!("Plugin Toml File Unable To Be Parsed For .{plugin_extension} Files").as_str(),
        )
    };
    let mut file: Vec<String> = fs::read_to_string(file_path.to_owned())
        .expect(format!("File Supplied Doesnt Exist: {}", file_path).as_str())
        .replace("\t", "    ")
        .split("\n")
        .map(|x| x.to_string())
        .collect();
    file.pop();

    let term_size = terminal::size().unwrap();

    let mut screen = Screen::new(
        match file.len() < term_size.1 as usize - 1 {
            true => file.len(),
            false => term_size.1 as usize - 1,
        },
        file_path.split("/").last().unwrap().to_owned(),
    );
    screen.write_term(&file);
    loop {
        if term_size.1 as usize >= file.len() {
            screen.line_top = 0;
            screen.line_bottom = file.len();
        }
        screen.handle_event(&mut file, &file_path);
        screen.write_term(&file);
    }
}