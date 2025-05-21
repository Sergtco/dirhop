use std::{
    fmt,
    io::{self, Write},
    ops::Div,
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
            let ans = if label.len() == 2 { "" } else { ans };
            let item = (ans, label, &bind.item);
            let item_size = format!("[{}{}]{}", ans, label, bind.item).len();
            max_item_size = max_item_size.max(item_size + 1);
            items.push(item);
        }

        let mut chunk_size = self.bounds.height;
        for size in (self.bounds.height.div(2)..self.bounds.height).rev() {
            let n_cols = items.len().div_ceil(size.into());
            if n_cols * max_item_size > self.bounds.width.into() {
                break;
            }
            chunk_size = size;
        }

        for (col_ind, chunk) in items.chunks(chunk_size.into()).enumerate() {
            for (row_ind, item) in chunk.iter().enumerate() {
                queue!(
                    self.stderr,
                    cursor::MoveTo(
                        (self.bounds.x as usize + col_ind * max_item_size) as u16,
                        self.bounds.y + 2 + row_ind as u16
                    ),
                    style::Print(format!("[{}{}]{}", item.0.blue(), item.1, style(item.2))),
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
