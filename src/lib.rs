pub mod app;
pub mod args;
mod matcher;
mod tui;
mod util;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}
