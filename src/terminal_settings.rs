// RIP the ~ATH code that once lives here
use std::io::{self, Stdout};

use crossterm::{cursor, event, execute, terminal};
use ratatui::backend::CrosstermBackend;

use crate::DynErrResult;

pub fn setup(stdout: &mut CrosstermBackend<Stdout>) -> DynErrResult<()> {
    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        terminal::Clear(terminal::ClearType::All),
        terminal::DisableLineWrap,
        event::EnableMouseCapture,
        cursor::Hide,
    )?;
    Ok(())
}
pub fn cleanup() -> DynErrResult<()> {
    terminal::disable_raw_mode()?;
    execute!(
        io::stdout(),
        cursor::Show,
        event::DisableMouseCapture,
        terminal::EnableLineWrap,
        terminal::LeaveAlternateScreen,
    )?;
    Ok(())
}
