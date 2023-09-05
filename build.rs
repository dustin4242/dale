use std::{env, process::Command};

fn main() {
    let home = env::var("HOME").unwrap();
    let cwd = env::current_dir().unwrap();
    println!("cargo:rerun-if-changed={home}/.config/dale/plugins");
    Command::new("cp")
        .args([
            "-r",
            format!("{}/plugins", cwd.to_str().unwrap()).as_str(),
            format!("{home}/.config/dale").as_str(),
        ])
        .spawn()
        .unwrap();
}
