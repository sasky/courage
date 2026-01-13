//! TUI module for Courage
//!
//! Provides a terminal user interface as an alternative to the Telegram bot.

mod app;
mod terminal;

pub use app::App;
pub use terminal::{init, restore};
