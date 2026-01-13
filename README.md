# Courage

A personal relationship database with voice input. Send voice memos to a Telegram bot, and it will transcribe, process, and update your Obsidian-compatible markdown files.

## Features

- **Voice Input**: Send voice memos via Telegram
- **Transcription**: Automatic transcription with OpenAI Whisper
- **Smart Processing**: Claude AI extracts people and relationship data
- **Markdown Files**: Creates/updates Obsidian-compatible markdown files
- **Bidirectional Relationships**: Automatically links people both ways
- **Clarification**: Asks when input is ambiguous (e.g., multiple "Johns")

## Architecture

```
Telegram Voice Memo
        ↓
Download audio file
        ↓
Transcribe (OpenAI Whisper)
        ↓
Read existing People/*.md
        ↓
Process (Claude Haiku)
        ↓
Create/Update markdown files
        ↓
Confirm via Telegram
```

## Setup

### 1. Create a Telegram Bot

1. Message [@BotFather](https://t.me/botfather) on Telegram
2. `/newbot` and follow prompts
3. Copy the bot token

### 2. Get API Keys

- **OpenAI**: https://platform.openai.com/api-keys
- **Anthropic**: https://console.anthropic.com/

### 3. Configure Environment

```bash
cp .env.example .env
# Edit .env with your keys:
# - TELEGRAM_BOT_TOKEN
# - OPENAI_API_KEY
# - ANTHROPIC_API_KEY
# - PEOPLE_DIR (path to your People folder)
```

### 4. Build and Run

```bash
# Build
cargo build --release

# Run
./target/release/courage
```

Or run in development mode:

```bash
cargo run
```

## Usage

### Voice Commands

Send voice messages to your bot:

- "Add a new person John Smith, he works at Google in Wellington"
- "John's birthday is March 15th"
- "Sarah is John's wife, she's a designer"
- "Had coffee with John, he's training for a marathon now"

### Text Commands

You can also type messages directly:

- "Update John Smith: new job at Apple"
- "John's favorite movie is The Matrix"

### Clarification

When input is ambiguous:

```
You: "Update John's phone number"
Bot: "I found two Johns:
     1. John Smith - works at Google
     2. John Davies - met at conference
     Which one?"
You: "John Smith"
Bot: "✅ Updated John Smith"
```

## Person File Format

Files are stored in `People/` as markdown:

```markdown
---
first_name: John
last_name: Smith
location: Wellington
work: Software Engineer at Google
birthday: 1985-03-15
created: 2025-01-02
updated: 2025-01-02
---

# John Smith

## Relationships
**Parents:** 
**Children:** 
**Siblings:** 
**Partner:** [[Sarah Smith]]
**Friends:** [[Bob Jones]]

## How We Met
Met at a tech conference in 2024

## Hobbies & Passions
- Marathon training (added 2025-01-02)

## Other Notes
Loves Italian food
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `TELEGRAM_BOT_TOKEN` | Yes | Telegram bot token from BotFather |
| `OPENAI_API_KEY` | Yes | OpenAI API key for Whisper |
| `ANTHROPIC_API_KEY` | Yes | Anthropic API key for Claude |
| `PEOPLE_DIR` | No | Path to People folder (default: `./People`) |
| `CLAUDE_MODEL` | No | Claude model (default: `claude-3-5-haiku-20241022`) |
| `RUST_LOG` | No | Log level (default: `courage=info`) |

## Development

```bash
# Run with debug logging
RUST_LOG=courage=debug cargo run

# Run tests
cargo test

# Check without building
cargo check
```

## License

MIT
