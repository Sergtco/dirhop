use std::{env, fs};

fn main() {
    let mut args = env::args();
    args.next();

    let path = match args.next() {
        Some(val) => val,
        None => ".".to_string(),
    };

    let entries = fs::read_dir(&path).unwrap();
    for entry in entries {
        if let Ok(entry) = entry {
            println!("{}{}", path, entry.file_name().to_str().unwrap());
        }
    }
}
