use std::io::{self, Stdout};

use crossterm::{cursor, event, execute, terminal};
use ratatui::backend::CrosstermBackend;

use crate::DynErrResult;

pub struct TerminalSettings {}
impl TerminalSettings {
    pub fn setup_terminal(stdout: &mut CrosstermBackend<Stdout>) -> DynErrResult<Self> {
        terminal::enable_raw_mode()?;
        #[cfg(not(debug_assertions))]
        execute!(
            stdout,
            terminal::EnterAlternateScreen,
            terminal::Clear(terminal::ClearType::All),
            terminal::DisableLineWrap,
            event::EnableMouseCapture,
            cursor::Hide,
        )?;
        #[cfg(debug_assertions)]
        execute!(
            stdout,
            terminal::DisableLineWrap,
            event::EnableMouseCapture,
            cursor::Hide,
        )?;
        Ok(Self{})
    }
}
impl Drop for TerminalSettings {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        #[cfg(not(debug_assertions))]
        let _ = execute!(
            io::stdout(),
            terminal::LeaveAlternateScreen,
            terminal::EnableLineWrap,
            event::DisableMouseCapture,
            cursor::Show,
        );
        #[cfg(debug_assertions)]
        let _ = execute!(
            io::stdout(),
            terminal::EnableLineWrap,
            event::DisableMouseCapture,
            cursor::Show,
        );
    }
}