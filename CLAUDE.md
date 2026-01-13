# Courage - Project Guidelines

## Development Practices

### Test-Driven Development
This project uses TDD. When implementing features:
1. Write failing tests first
2. Implement the minimum code to pass
3. Refactor while keeping tests green

Run tests with:
```bash
cargo test
```

### Linting
Always run Rust linting at the end of a session:
```bash
cargo clippy -- -D warnings
cargo fmt --check
```

## Architecture

### Input Methods
The system supports multiple input methods for triggering relationship database updates:

1. **Telegram Bot** (existing) - Voice memos and text messages via Telegram
2. **Terminal UI** (planned) - Direct CLI interaction using Ratatui

### TUI Framework
We use [Ratatui](https://ratatui.rs/) (formerly tui-rs) for building the terminal user interface. This provides an alternative way to interact with Courage without going through Telegram.

Key Ratatui patterns:
- Immediate mode rendering
- Event-driven input handling
- Composable widgets

## Project Structure

```
src/
├── main.rs           # Entry point
├── lib.rs            # Library exports
├── config.rs         # Configuration
├── telegram/         # Telegram bot interface
├── llm/              # Claude AI integration
├── transcription/    # Whisper transcription
├── people/           # Person data management
├── state/            # Conversation state
└── utils/            # Shared utilities
```

## Dependencies

Key crates:
- `teloxide` - Telegram bot framework
- `reqwest` - HTTP client
- `tokio` - Async runtime
- `ratatui` - Terminal UI (for CLI interface)
- `crossterm` - Terminal manipulation (backend for ratatui)
