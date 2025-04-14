use std::io::{self, Write};

use crossterm::{
    ExecutableCommand, cursor, execute, queue,
    style::{self, Stylize},
    terminal::{self, ClearType},
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

        Ok(Self {
            stderr: io::stderr(),
            bounds,
        })
    }

    pub fn draw_list(&mut self, ans: &str, binds: &Binds) -> io::Result<()> {
        self.clear_rect(self.bounds)?;
        self.stderr
            .execute(cursor::MoveTo(self.bounds.x, self.bounds.y))?;

        for (label, entry) in binds.iter().take(self.bounds.height as usize) {
            let label = label.strip_prefix(ans).unwrap_or(&label);
            let entry = match entry.is_dir() {
                true => entry.to_string_lossy().to_string().bold().dark_blue(),
                false => entry.to_string_lossy().to_string().stylize(),
            };

            queue!(
                self.stderr,
                style::Print(format!("[{}{}]{}\r\n", ans.blue(), label, entry)),
            )?;
        }

        self.stderr.flush()?;
        Ok(())
    }

    pub fn restore(&mut self) -> io::Result<()> {
        self.clear_all()?;
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
