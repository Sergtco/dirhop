use std::{
    collections::BTreeMap,
    env,
    fs::{self, DirEntry},
    io::{self, Write},
};

use crossterm::{
    ExecutableCommand, cursor,
    event::{Event, KeyModifiers},
    execute,
    style::Stylize,
    terminal,
};

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

fn restore() -> io::Result<()> {
    clear()?;

    terminal::disable_raw_mode()
}

fn clear() -> io::Result<()> {
    execute!(
        io::stderr(),
        cursor::RestorePosition,
        terminal::Clear(terminal::ClearType::FromCursorDown)
    )?;

    Ok(())
}

fn draw_list(ans: &str, binds: &BTreeMap<String, DirEntry>) -> io::Result<()> {
    let mut stderr = io::stderr();
    stderr.execute(cursor::SavePosition)?;

    for (label, entry) in binds.iter() {
        let label = label.strip_prefix(ans).unwrap_or(&label);
        write!(
            stderr,
            "[{}{}]{}\r\n",
            ans.blue(),
            label,
            entry.path().to_string_lossy()
        )?;
    }
    Ok(())
}

fn match_binds(ans: &str, binds: BTreeMap<String, DirEntry>) -> BTreeMap<String, DirEntry> {
    binds
        .into_iter()
        .filter(|(label, _)| label.starts_with(ans))
        .collect()
}

fn main() -> io::Result<()> {
    let mut args = env::args();
    args.next();

    let path = match args.next() {
        Some(val) => val,
        None => ".".to_string(),
    };

    let entries = fs::read_dir(&path)?.filter_map(|entry| entry.ok());
    let labels = Labeler::new().into_iter();

    let mut binds = labels.zip(entries.into_iter()).collect::<BTreeMap<_, _>>();

    terminal::enable_raw_mode()?;

    let mut ans = String::new();
    draw_list(&ans, &binds)?;

    while ans.len() < 2 {
        match crossterm::event::read()? {
            Event::Key(event) => {
                if event.code.is_char('c') && event.modifiers == KeyModifiers::CONTROL {
                    restore()?;
                }
                if let Some(key) = event.code.as_char() {
                    if event.modifiers.is_empty() {
                        ans.push(key);
                        binds = match_binds(&ans, binds);
                        clear()?;
                        draw_list(&ans, &binds)?;
                    }
                }
            }
            _ => {}
        }
    }

    clear()?;
    terminal::disable_raw_mode()?;

    if let Some(entry) = binds.get(&ans) {
        println!("{}", entry.path().to_string_lossy());
    } else {
        println!("Wrong label!");
    }
    Ok(())
}
