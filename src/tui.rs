use std::{
    io::{self, Write},
    path::PathBuf,
};

use crossterm::{
    ExecutableCommand, cursor, execute, queue,
    style::{self, Stylize},
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::Binds;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug)]
pub struct Renderer {
    stderr: io::Stderr,
    bounds: Rect,
}

impl Renderer {
    pub fn with_bounds(bounds: Rect) -> io::Result<Self> {
        terminal::enable_raw_mode()?;

        let mut stderr = io::stderr();
        stderr.execute(EnterAlternateScreen)?;

        Ok(Self { stderr, bounds })
    }

    pub fn draw_list(&mut self, ans: &str, binds: &Binds) -> io::Result<()> {
        self.clear_rect(self.bounds)?;

        queue!(self.stderr, cursor::MoveTo(self.bounds.x, self.bounds.y),)?;
        let mut parent = PathBuf::default();

        for bind in binds.iter().take(self.bounds.height as usize - 1) {
            let label = bind.label.strip_prefix(ans).unwrap_or(&bind.label);
            let new_parent = bind.path.parent().unwrap_or(&parent);

            if new_parent != parent {
                parent = new_parent.to_path_buf();
                queue!(
                    self.stderr,
                    style::Print(format!("{}\r\n", parent.to_string_lossy())),
                )?;
            }

            let filename = bind
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let entry = match bind.path.is_dir() {
                true => (filename + "/").bold().dark_blue(),
                false => filename.stylize(),
            };

            queue!(
                self.stderr,
                style::Print(format!("    [{}{}]{}\r\n", ans.blue(), label, entry)),
            )?;
        }

        self.stderr.flush()?;
        Ok(())
    }

    pub fn restore(&mut self) -> io::Result<()> {
        self.clear_all()?;
        self.stderr.execute(LeaveAlternateScreen)?;
        terminal::disable_raw_mode()
    }

    fn clear_rect(&mut self, area: Rect) -> io::Result<()> {
        let spaces = " ".repeat(area.width as usize);
        for y in self.bounds.y..(self.bounds.y + self.bounds.height) {
            queue!(
                self.stderr,
                cursor::MoveTo(self.bounds.x, y),
                style::Print(spaces.clone()),
            )?;
        }

        self.stderr.flush()?;
        Ok(())
    }

    pub fn clear_all(&mut self) -> io::Result<()> {
        execute!(
            self.stderr,
            cursor::MoveTo(self.bounds.x, self.bounds.y),
            terminal::Clear(ClearType::FromCursorDown)
        )?;

        Ok(())
    }
}
