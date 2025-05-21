use std::{
    fmt::{self},
    io::{self, Write},
    thread, time,
};

use crossterm::{
    ExecutableCommand, QueueableCommand, cursor, execute, queue,
    style::{self, StyledContent, Stylize},
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::util::Bind;

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
    pub fn new_fullscreen() -> io::Result<Self> {
        let (width, height) = terminal::size()?;
        Ok(Renderer::new_with_bounds(Rect {
            x: 0,
            y: 0,
            width,
            height,
        })?)
    }

    pub fn new_with_bounds(bounds: Rect) -> io::Result<Self> {
        terminal::enable_raw_mode()?;

        let mut stderr = io::stderr();
        execute!(
            stderr,
            EnterAlternateScreen,
            cursor::MoveTo(bounds.x, bounds.y)
        )?;

        Ok(Self { stderr, bounds })
    }

    pub fn draw_list<'a, T, Style, D>(
        &mut self,
        ans: &str,
        header: impl fmt::Display,
        binds: impl IntoIterator<Item = &'a Bind<T>>,
        style: Style,
    ) -> io::Result<()>
    where
        T: fmt::Display + Clone + 'a,
        Style: Fn(&T) -> StyledContent<D>,
        D: fmt::Display,
    {
        self.clear_rect(self.bounds)?;

        queue!(self.stderr, cursor::MoveTo(self.bounds.x, self.bounds.y),)?;

        self.stderr.queue(style::Print(format!("{}\r\n", header)))?;
        self.stderr.queue(style::Print("\r\n"))?;

        let mut items = Vec::new();
        let mut max_item_size = 0;
        for bind in binds.into_iter() {
            let label = bind.label.strip_prefix(ans).unwrap_or(&bind.label);
            let item = format!("[{}{}]{}", ans.blue(), label, style(&bind.item));
            max_item_size = max_item_size.max(item.len());
            items.push(item);
        }

        for (col_offset, chunk) in items.chunks((self.bounds.height - 1).into()).enumerate() {
            for (row_offset, item) in chunk.iter().enumerate() {
                queue!(
                    self.stderr,
                    cursor::MoveTo(
                        (self.bounds.x as usize + col_offset * max_item_size) as u16,
                        self.bounds.y + 2 + row_offset as u16
                    ),
                    style::Print(item),
                )?
            }
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
