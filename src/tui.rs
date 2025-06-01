use std::{
    fmt::Display,
    io::{self, Write},
};

use crossterm::{
    cursor, execute, queue,
    style::{self, StyledContent, Stylize},
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::Rect;

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

    pub fn restore(&mut self) -> io::Result<()> {
        self.clear_all()?;
        execute!(self.buf, LeaveAlternateScreen, cursor::Show)?;
        terminal::disable_raw_mode()
    }

    pub fn draw<D>(&mut self, bounds: Rect, item: &D) -> io::Result<()>
    where
        D: Draw,
    {
        item.draw(bounds, &mut self.buf)
    }

    pub fn redraw<D>(&mut self, bounds: Rect, item: &D) -> io::Result<()>
    where
        D: Draw,
    {
        self.clear_rect(bounds)?;
        self.draw(bounds, item)?;
        self.buf.flush()
    }

    fn clear_rect(&mut self, bounds: Rect) -> io::Result<()> {
        let spaces = " ".repeat(bounds.width as usize);
        for y in bounds.y..(bounds.y + bounds.height) {
            queue!(
                self.buf,
                cursor::MoveTo(bounds.x, y),
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

pub trait Draw {
    fn draw<B>(&self, bounds: Rect, buf: &mut B) -> io::Result<()>
    where
        B: Write;
}

impl<T: Draw> Draw for &T {
    fn draw<B>(&self, bounds: Rect, buf: &mut B) -> io::Result<()>
    where
        B: Write,
    {
        (*self).draw(bounds, buf)
    }
}

pub type Style<T> = fn(&T) -> StyledContent<String>;

#[derive(Debug)]
pub struct Text {
    text: String,
    style: Style<String>,
}

impl<T: Display> From<T> for Text {
    fn from(value: T) -> Self {
        Text {
            text: value.to_string(),
            style: |s| s.clone().yellow(),
        }
    }
}

impl Draw for Text {
    fn draw<B>(&self, bounds: Rect, buf: &mut B) -> io::Result<()>
    where
        B: Write,
    {
        queue!(
            buf,
            cursor::MoveTo(bounds.x, bounds.y),
            style::Print((self.style)(&self.text))
        )?;

        Ok(())
    }
}
