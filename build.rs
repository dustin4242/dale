use std::{env, fs::create_dir, process::Command};

fn main() {
    let home = env::var("HOME");
    let cwd = env::current_dir();
    match (home, cwd) {
        (Ok(home), Ok(cwd)) => {
            create_dir(format!("{home}/.config/dale")).unwrap_or_default();
            Command::new("cp")
                .args([
                    "-r",
                    format!("{}/plugins", cwd.to_str().unwrap()).as_str(),
                    format!("{home}/.config/dale").as_str(),
                ])
                .spawn()
                .unwrap();
        }
        _ => (),
    }
}
