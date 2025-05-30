use std::{
    fmt::Display,
    io::{self, Write},
};

use crossterm::{
    QueueableCommand, cursor, execute, queue,
    style::{self, StyledContent, Stylize},
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::util::{MatcherPage, Rect};

#[derive(Debug)]
pub struct Renderer {
    buf: io::Stderr,
    bounds: Rect,
}

impl Renderer {
    pub fn new_with_bounds(bounds: Rect) -> io::Result<Self> {
        terminal::enable_raw_mode()?;

        let mut buf = io::stderr();
        execute!(
            buf,
            EnterAlternateScreen,
            cursor::MoveTo(bounds.x, bounds.y),
            cursor::Hide
        )?;

        Ok(Self { buf, bounds })
    }

    pub fn draw_list<'a, T, Style, D>(
        &mut self,
        input: &str,
        header: impl Display,
        page: MatcherPage<T>,
        style: Style,
    ) -> io::Result<()>
    where
        T: Display + Clone + 'a,
        Style: Fn(&T) -> StyledContent<D>,
        D: Display,
    {
        self.clear_rect(self.bounds)?;

        queue!(self.buf, cursor::MoveTo(self.bounds.x, self.bounds.y),)?;

        self.buf.queue(style::Print(format!("{}\r\n", header)))?;
        self.buf.queue(style::Print("\r\n"))?;

        for (ind, bind) in page.iter().enumerate() {
            let row = ind % self.bounds.height as usize;
            let col = ind / self.bounds.height as usize;
            let (mut rest, mut typed) = (bind.1.as_str(), "");
            if let Some(right) = rest.strip_prefix(input) {
                rest = right;
                typed = input;
            }
            queue!(
                self.buf,
                cursor::MoveTo(
                    (self.bounds.x as usize + col * (page.item_size() + 3)) as u16,
                    self.bounds.y + 2 + row as u16
                ),
                style::Print(format!("[{}{}]{}", typed.blue(), rest, style(bind.0))),
            )?
        }

        self.buf.flush()?;
        Ok(())
    }

    pub fn restore(&mut self) -> io::Result<()> {
        self.clear_all()?;
        execute!(self.buf, LeaveAlternateScreen, cursor::Show)?;
        terminal::disable_raw_mode()
    }

    fn clear_rect(&mut self, area: Rect) -> io::Result<()> {
        let spaces = " ".repeat(area.width as usize);
        for y in self.bounds.y..(self.bounds.y + self.bounds.height) {
            queue!(
                self.buf,
                cursor::MoveTo(self.bounds.x, y),
                style::Print(spaces.clone()),
            )?;
        }

        self.buf.flush()?;
        Ok(())
    }

    pub fn clear_all(&mut self) -> io::Result<()> {
        execute!(
            self.buf,
            cursor::MoveTo(self.bounds.x, self.bounds.y),
            terminal::Clear(ClearType::FromCursorDown)
        )?;

        Ok(())
    }
}
