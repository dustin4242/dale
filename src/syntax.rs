use regex::Regex;
use std::collections::HashMap;
use toml::Value;

pub fn highlight(plugin: Option<toml::Value>, mut file: String) -> String {
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
    let types = basic_table.get("types").unwrap().as_array().unwrap();
    for key_type in types {
        let replace = key_type.as_str().unwrap();
        replace_syntax.insert(
            format!(r"\b{replace}\b"),
            format!("\x1b[33m{replace}\x1b[37m"),
        );
    }
    for replace in replace_syntax {
        file = Regex::new(replace.0.as_str())
            .unwrap()
            .replace_all(&file, replace.1)
            .to_string()
    }
    basic_op(basic_table, &mut file, "numbers", r"\b\d+\b", "magenta", 0);
    basic_op(
        basic_table,
        &mut file,
        "strings",
        "\"+[^\"]*\"*",
        "magenta",
        0,
    );
    basic_op(
        basic_table,
        &mut file,
        "functions",
        r"[\w\d]+\(+",
        "cyan",
        -1,
    );
    custom_op(&syntax, &mut file);
    file
}

fn basic_op<T: ToString>(
    table: &Value,
    file: &mut String,
    option_name: T,
    regex: &'static str,
    mut color: &'static str,
    offset: i32,
) {
    let option = table
        .get(option_name.to_string())
        .unwrap()
        .as_bool()
        .unwrap();
    match color {
        "red" => color = "\x1b[31m",
        "green" => color = "\x1b[32m",
        "yellow" => color = "\x1b[33m",
        "blue" => color = "\x1b[34m",
        "magenta" => color = "\x1b[35m",
        "cyan" => color = "\x1b[36m",
        x => panic!("Unknown Color In Syntax: {}", x),
    }
    if option {
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
fn custom_op(syntax: &Value, file: &mut String) {
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
}
