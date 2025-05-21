use std::{
    collections::HashMap,
    env,
    error::Error,
    fs,
    io::{self},
    path::PathBuf,
    process::exit,
    result,
};

use args::{Opts, usage};
use crossterm::{
    event::{Event, KeyEvent, KeyModifiers},
    style::Stylize,
    tty::IsTty,
};

mod args;
mod tui;
mod util;
use tui::Renderer;
use util::{Binds, DisplayablePathBuf};

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
        .map(|pb| DisplayablePathBuf::from(pb))
        .collect::<Vec<_>>();
    entries.sort();

    let mut binds = Binds::new(&entries)?;

    let mut renderer = Renderer::new_fullscreen()?;

    let mut input = String::new();

    let Some(entry) = entries.get(0) else { exit(0) };

    let parent = fs::canonicalize(entry.get().parent().unwrap_or(&PathBuf::new()))
        .unwrap_or(PathBuf::default());

    let style = |bind: &&DisplayablePathBuf| match bind.get().is_dir() {
        true => bind.to_string().blue(),
        false => bind.to_string().stylize(),
    };

    renderer.draw_list(
        &input,
        parent.to_string_lossy().yellow(),
        binds.iter(),
        style,
    )?;

    while input.len() < 2 {
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
                        input.push(key);
                        binds.match_prefix(&input);

                        renderer.draw_list(
                            &input,
                            parent.to_string_lossy().yellow(),
                            binds.iter(),
                            style,
                        )?;
                    }
                }
            }
            _ => {}
        }
    }

    renderer.restore()?;
    let bind_map = binds
        .into_iter()
        .map(|bind| (bind.label, bind.item))
        .collect::<HashMap<_, _>>();

    if let Some(entry) = bind_map.get(&input) {
        println!("{}", entry.get().to_string_lossy());
    } else {
        println!("Wrong label!");
    }
    Ok(())
}
