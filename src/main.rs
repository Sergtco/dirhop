use std::{
    collections::BTreeMap,
    env,
    error::Error,
    fs,
    io::{self},
    path::PathBuf,
    process::exit,
    result,
    str::FromStr,
};

use crossterm::{
    cursor,
    event::{Event, KeyEvent, KeyModifiers},
    terminal,
    tty::IsTty,
};

mod tui;
use tui::{Rect, Renderer};

struct Labeler(u16);

impl Labeler {
    const TAG_INDEX_LIMIT: u16 = 675;

    fn new() -> Self {
        Self(0)
    }
}

impl Iterator for Labeler {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        if self.0 > Self::TAG_INDEX_LIMIT {
            return None;
        }

        let first_letter = (self.0 / 26 + 97) as u8;
        let second_letter = (self.0 % 26 + 97) as u8;
        self.0 += 1;
        Some(String::from_utf8(vec![first_letter, second_letter]).expect("valid utf8 chars"))
    }
}
type Binds = BTreeMap<String, PathBuf>;

fn match_binds(ans: &str, binds: Binds) -> Binds {
    binds
        .into_iter()
        .filter(|(label, _)| label.starts_with(ans))
        .collect()
}

type Result<T> = result::Result<T, Box<dyn Error>>;

#[derive(Debug, Default)]
struct Opts {
    program_name: String,
    base_path: PathBuf,
    show_hidden: bool,
}

impl Opts {
    pub fn from_args(mut args: impl Iterator<Item = String>) -> Result<Self> {
        let mut conf = Self::default();
        conf.program_name = args.next().ok_or("Couldn't get program name")?;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" => conf.show_hidden = true,
                "--help" => {
                    usage();
                    exit(0)
                }
                someflag if someflag.starts_with("-") => {
                    return Err(format!("wrong flag: {someflag}").into());
                }
                rest => conf.base_path = PathBuf::from_str(&rest)?,
            }
        }

        if conf.base_path.to_string_lossy().len() == 0 {
            conf.base_path = ".".into();
        }

        Ok(conf)
    }
}

fn usage() {
    println!("USAGE:");
    println!("dirhop [PATH] [FLAGS]");
    println!("");
    println!("-h show hidden files");
}

fn main() -> Result<()> {
    if !io::stderr().is_tty() {
        eprintln!("This is not a terminal, Babe!");
        exit(1);
    }

    let opts = Opts::from_args(env::args()).unwrap_or_else(|err| {
        eprintln!("Argument error: {err}.");
        usage();
        exit(1)
    });

    let entries = fs::read_dir(&opts.base_path)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|e| {
            e.file_name()
                .map(|filename| !filename.to_string_lossy().starts_with(".") || opts.show_hidden)
                .unwrap_or(false)
        });

    let labels = Labeler::new().into_iter();

    let mut binds = labels.zip(entries.into_iter()).collect::<Binds>();

    let mut renderer = {
        let (x, y) = cursor::position()?;
        let (width, height) = terminal::size()?;
        Renderer::with_bounds(Rect {
            x,
            y,
            width: width - x - 1,
            height: height - y - 1,
        })?
    };

    let mut ans = String::new();
    renderer.draw_list(&ans, &binds)?;

    while ans.len() < 2 {
        match crossterm::event::read()? {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => {
                if code.is_char('c') && modifiers == KeyModifiers::CONTROL {
                    renderer.restore()?;
                    return Ok(());
                }

                if let Some(key) = code.as_char() {
                    if modifiers.is_empty() {
                        ans.push(key);
                        binds = match_binds(&ans, binds);

                        renderer.draw_list(&ans, &binds)?;
                    }
                }
            }
            _ => {}
        }
    }

    renderer.restore()?;

    if let Some(entry) = binds.get(&ans) {
        println!("{}", entry.to_string_lossy());
    } else {
        println!("Wrong label!");
    }
    Ok(())
}
