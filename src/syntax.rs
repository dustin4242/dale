use regex::Regex;
use std::collections::HashMap;
use toml::{map::Map, Value};

pub struct Syntax {
    replace: HashMap<String, String>,
    options: Vec<bool>,
    custom: Vec<(String, String)>,
}

pub fn load_syntax(plugin: Option<toml::Table>) -> Option<Syntax> {
    match plugin {
        Some(syntax) => {
            let empty_table = Value::Table(Map::new());
            let empty_array = Value::Array(Vec::new());
            let mut replace_syntax = HashMap::new();
            let basic_table = syntax.get("basic").unwrap_or(&empty_table);
            let keywords = basic_table
                .get("keywords")
                .unwrap_or(&empty_array)
                .as_array()
                .expect("keywords in syntax toml is not an array");
            let types = basic_table
                .get("types")
                .unwrap_or(&empty_array)
                .as_array()
                .expect("keywords in syntax toml is not an array");
            for key in keywords {
                let replace = key
                    .as_str()
                    .expect("keywords in syntax toml is not an array of strings");
                replace_syntax.insert(
                    format!(r"\b{replace}\b"),
                    format!("\x1b[32m{replace}\x1b[37m"),
                );
            }
            for key_type in types {
                let replace = key_type
                    .as_str()
                    .expect("keywords in syntax toml is not an array of strings");
                replace_syntax.insert(
                    format!(r"\b{replace}\b"),
                    format!("\x1b[33m{replace}\x1b[37m"),
                );
            }
            let numbers = basic_table
                .get("numbers".to_string())
                .unwrap_or(&Value::Boolean(false))
                .as_bool()
                .expect("Option numbers is not a bool in syntax toml");
            let strings = basic_table
                .get("strings".to_string())
                .unwrap_or(&Value::Boolean(false))
                .as_bool()
                .expect("Option strings is not a bool in syntax toml");
            let functions = basic_table
                .get("functions".to_string())
                .unwrap_or(&Value::Boolean(false))
                .as_bool()
                .expect("Option functions is not a bool in syntax toml");
            let custom_table = syntax.get("custom").unwrap();
            let mut custom_replace = vec![];
            match custom_table.as_table() {
                Some(table) => {
                    for syntax in table {
                        custom_replace
                            .push((syntax.0.to_owned(), syntax.1.as_str().unwrap().to_owned()));
                    }
                }
                None => (),
            };
            Some(Syntax {
                replace: replace_syntax,
                options: vec![numbers, strings, functions],
                custom: custom_replace,
            })
        }
        None => None,
    }
}

pub fn highlight(plugin: &Syntax, mut file: String) -> String {
    for replace in plugin.replace.clone() {
        file = Regex::new(replace.0.as_str())
            .unwrap()
            .replace_all(&file, replace.1)
            .to_string()
    }
    basic_op(
        plugin.options.get(0).unwrap(),
        &mut file,
        r"\b\d+(\.\d+)?\b",
        "magenta",
        0,
    );
    basic_op(
        plugin.options.get(1).unwrap(),
        &mut file,
        "\"+[^\"]*\"*",
        "magenta",
        0,
    );
    basic_op(
        plugin.options.get(2).unwrap(),
        &mut file,
        r"[\w\d]+\(+",
        "cyan",
        -1,
    );
    custom_op(plugin.custom.clone(), &mut file);
    file
}

fn basic_op(
    option: &bool,
    file: &mut String,
    regex: &'static str,
    mut color: &'static str,
    offset: i32,
) {
    if option.to_owned() {
        match color {
            "red" => color = "\x1b[31m",
            "green" => color = "\x1b[32m",
            "yellow" => color = "\x1b[33m",
            "blue" => color = "\x1b[34m",
            "magenta" => color = "\x1b[35m",
            "cyan" => color = "\x1b[36m",
            x => panic!("Unknown Color In Syntax: {}", x),
        }
        let option_regex = Regex::new(regex).unwrap();
        let temp_file = file.clone();
        let mut finds = option_regex.find_iter(&temp_file);
        let mut i = 0;
        loop {
            match finds.next() {
                Some(find) => {
                    file.insert_str(find.start() + 10 * i, color);
                    file.insert_str(find.end() + offset as usize + 5 * ((2 * i) + 1), "\x1b[37m");
                    i += 1
                }
                None => break,
            }
        }
    }
}
fn custom_op(custom_replace: Vec<(String, String)>, file: &mut String) {
    for syntax in custom_replace {
        let custom_regex = Regex::new(syntax.0.as_str()).unwrap();
        let mut color = syntax.1.as_str();
        match color {
            "red" => color = "\x1b[31m",
            "green" => color = "\x1b[32m",
            "yellow" => color = "\x1b[33m",
            "blue" => color = "\x1b[34m",
            "magenta" => color = "\x1b[35m",
            "cyan" => color = "\x1b[36m",
            "white" => color = "\x1b[37m",
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
}
