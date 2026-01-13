//! TUI Application state and logic

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::config::Config;

/// Main application state for the TUI
#[derive(Debug)]
pub struct App {
    /// Application configuration
    #[allow(dead_code)]
    config: Config,
    /// Whether the app should quit
    should_quit: bool,
}

impl App {
    /// Create a new App instance
    pub fn new(config: Config) -> Self {
        Self {
            config,
            should_quit: false,
        }
    }

    /// Run the main application loop
    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    /// Draw the UI
    fn draw(&self, frame: &mut Frame) {
        let chunks = Layout::vertical([
            Constraint::Min(3),    // Feed area
            Constraint::Length(3), // Input area
        ])
        .split(frame.area());

        // Feed area (top)
        let feed = Paragraph::new(Text::raw("Welcome to Courage TUI\n\nPress 'q' to quit")).block(
            Block::default()
                .title(" Feed ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
        frame.render_widget(feed, chunks[0]);

        // Input area (bottom)
        let input = Paragraph::new(Text::raw("")).block(
            Block::default()
                .title(" Input ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );
        frame.render_widget(input, chunks[1]);
    }

    /// Handle keyboard and other events
    fn handle_events(&mut self) -> Result<()> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => self.should_quit = true,
                        KeyCode::Esc => self.should_quit = true,
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_config() -> Config {
        Config {
            telegram_token: "test_token".to_string(),
            openai_api_key: "test_openai".to_string(),
            people_dir: PathBuf::from("/tmp/test_people"),
            llm_model: "gpt-4o-mini".to_string(),
        }
    }

    #[test]
    fn test_app_new() {
        let config = test_config();
        let app = App::new(config);
        assert!(!app.should_quit);
    }
}
