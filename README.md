# Courage

Personal relationship database with voice input. Track interactions, notes, and memories about the people in your life.

## Features

- **Voice Input**: Send voice messages to capture notes hands-free
- **LLM-Powered**: Intelligent parsing and organization of your notes
- **People Database**: Markdown-based storage for easy editing and version control
- **Dual Interface**: Use via Telegram bot or terminal TUI

## Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- OpenAI API key (for transcription and LLM features)
- Telegram Bot Token (optional, for bot mode)

## Configuration

Create a `.env` file in the project root:

```bash
# Required
OPENAI_API_KEY=your_openai_api_key

# Optional - for Telegram bot mode
TELOXIDE_TOKEN=your_telegram_bot_token

# Optional - customize paths and models
PEOPLE_DIR=./people
LLM_MODEL=gpt-4o
```

## Building & Running

All commands should be run from the `courage/` directory.

```bash
cd courage
```

### Build

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

### Run

```bash
# Run TUI mode (terminal interface)
cargo run -- --tui

# Run Telegram bot mode
cargo run

# Run release build
cargo run --release -- --tui
```

### Install Locally

```bash
# Install to ~/.cargo/bin
cargo install --path .

# Then run from anywhere
courage --tui
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_name

# Run tests in release mode
cargo test --release
```

### Code Quality

```bash
# Run clippy (linter)
cargo clippy

# Run clippy with warnings as errors
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check formatting without changes
cargo fmt --check
```

### Other Useful Commands

```bash
# Check compilation without building
cargo check

# View documentation
cargo doc --open

# Update dependencies
cargo update

# Show dependency tree
cargo tree

# Clean build artifacts
cargo clean
```

## Project Structure

```
courage/
├── src/
│   ├── main.rs          # Entry point, CLI args
│   ├── config.rs        # Configuration management
│   ├── lib.rs           # Library exports
│   ├── llm/             # LLM integration (OpenAI)
│   ├── people/          # People database (markdown files)
│   ├── state/           # Conversation state management
│   ├── telegram/        # Telegram bot interface
│   ├── transcription/   # Voice transcription (Whisper)
│   ├── tui/             # Terminal UI (ratatui)
│   └── utils/           # Shared utilities
├── tests/               # Integration tests
└── Cargo.toml           # Dependencies
```

## Usage

### TUI Mode

Launch with `--tui` flag for an interactive terminal interface:

```bash
cargo run -- --tui
```

### Telegram Bot Mode

Run without flags to start the Telegram bot:

```bash
cargo run
```

Then interact with your bot on Telegram to add notes about people via text or voice messages.

## License

MIT
