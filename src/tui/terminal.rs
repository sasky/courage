//! Terminal setup and teardown utilities

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, stdout};

/// Initialize the terminal for TUI mode
///
/// This enables raw mode and enters the alternate screen buffer.
pub fn init() -> Result<ratatui::DefaultTerminal> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let terminal = ratatui::init();
    Ok(terminal)
}

/// Restore the terminal to its original state
///
/// This disables raw mode and leaves the alternate screen buffer.
pub fn restore() -> Result<()> {
    ratatui::restore();
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    // Terminal tests are difficult to run in CI/test environments
    // because they require an actual terminal. These are integration
    // tests that should be run manually.

    #[test]
    fn test_module_compiles() {
        // Ensures the module compiles correctly
        assert!(true);
    }
}
