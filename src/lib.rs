//! Courage - Personal Relationship Database with Voice Input
//!
//! This library provides the core functionality for managing a personal
//! relationship database using voice memos via Telegram, with AI-powered
//! processing using OpenAI Whisper and Claude.

pub mod config;
pub mod llm;
pub mod people;
pub mod state;
pub mod telegram;
pub mod transcription;
pub mod tui;
pub mod utils;
