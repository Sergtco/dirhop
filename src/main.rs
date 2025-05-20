use std::{
    collections::HashMap,
    env,
    error::Error,
    fs,
    io::{self},
    process::exit,
    result,
};

use args::{Opts, usage};
use crossterm::{
    event::{Event, KeyEvent, KeyModifiers},
    terminal,
    tty::IsTty,
};

mod args;
mod tui;
mod util;
use tui::{Rect, Renderer};
use util::{Bind, Binds, Labeler, match_prefix};

type Result<T> = result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    if !io::stderr().is_tty() {
        eprintln!("This is not a terminal!");
        exit(1);
    }

    let opts = Opts::from_args(env::args()).unwrap_or_else(|err| {
        eprintln!("Argument error: {err}.");
        usage();
        exit(1)
    });

    let mut entries = fs::read_dir(&opts.base_path)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|e| {
            e.file_name()
                .map(|filename| !filename.to_string_lossy().starts_with(".") || opts.show_hidden)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    entries.sort();

    let labels = Labeler::new().into_iter();

    let mut binds = labels
        .zip(entries.into_iter())
        .map(|(label, path)| Bind { label, path })
        .collect::<Binds>();

    let mut renderer = {
        let (x, y) = (0, 0);
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
                        binds = match_prefix(&binds, &ans);

                        renderer.draw_list(&ans, &binds)?;
                    }
                }
            }
            _ => {}
        }
    }

    renderer.restore()?;
    let bind_map = binds
        .into_iter()
        .map(|bind| (bind.label, bind.path))
        .collect::<HashMap<_, _>>();
    if let Some(entry) = bind_map.get(&ans) {
        println!("{}", entry.to_string_lossy());
    } else {
        println!("Wrong label!");
    }
    Ok(())
}
