use crossterm::terminal;
use std::{env, fs};

mod screen;
mod syntax;
use screen::Screen;
use syntax::load_syntax;

fn main() {
    let temp_path = env::args().nth(1);
    let (isrootdir, ishomedir, file_path): (bool, bool, String);
    match temp_path {
        None => {
            println!("Usage: dale [FILE_PATH]");
            return;
        }
        Some(temp_path) => {
            isrootdir = temp_path.starts_with("/");
            ishomedir = temp_path.starts_with("~");
            file_path = if isrootdir {
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
        }
    }
    let file_name = file_path.split("/").last().unwrap();
    let plugin: Option<toml::Table> = if file_name.contains(".") {
        let plugins_dir = format!("{}/.config/dale/plugins", env::var("HOME").unwrap());
        let plugin_extension = file_name.split(".").last().unwrap();
        let plugin_file =
            fs::read_to_string(format!("{}/{}.toml", plugins_dir, plugin_extension).as_str()).ok();
        match plugin_file {
            Some(x) => Some(
                x.parse::<toml::Table>().expect(
                    format!("Plugin Toml File Unable To Be Parsed For .{plugin_extension} Files")
                        .as_str(),
                ),
            ),
            None => None,
        }
    } else {
        None
    };
    let mut file: Vec<String> = fs::read_to_string(file_path.to_owned())
        .unwrap_or("".to_string())
        .replace("\t", "    ")
        .split("\n")
        .map(|x| x.to_string())
        .collect();
    if file.len() > 1 {
        file.pop();
    }

    let term_size = terminal::size().unwrap();

    let mut screen = Screen::new(
        match file.len() < term_size.1 as usize - 1 {
            true => file.len(),
            false => term_size.1 as usize - 1,
        },
        file_path.split("/").last().unwrap().to_owned(),
    );
    screen.syntax = load_syntax(plugin);
    screen.write_term(&file);
    loop {
        if term_size.1 as usize > file.len() {
            screen.line_top = 0;
            screen.line_bottom = file.len();
        }
        screen.handle_event(&mut file, &file_path);
        screen.write_term(&file);
    }
}
