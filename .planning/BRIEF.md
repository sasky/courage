# Courage TUI - Project Brief

## Vision
Add a terminal user interface (TUI) as an alternative input method for Courage. Users can interact with their personal relationship database directly from the terminal without needing Telegram.

## Problem
Currently, Courage only accepts input via Telegram bot. This requires:
- Phone/Telegram app access
- Internet connectivity for Telegram
- Context switching away from terminal

Developers and power users often prefer staying in the terminal.

## Solution
A Ratatui-based TUI that provides direct text input to the Courage system.

### UI Layout
```
┌─────────────────────────────────────────┐
│                                         │
│  [Response feed - scrolls down from     │
│   top, newest messages appear here]     │
│                                         │
│                                         │
├─────────────────────────────────────────┤
│  > [Input field - user types here]      │
└─────────────────────────────────────────┘
```

### Behavior
- **Input field** at center/bottom of screen
- **Response feed** flows down from center/top
- Input clears after submission, ready for next message
- Responses appear in the feed as they come back

## Current State
- Existing Rust codebase with modular architecture
- Telegram bot fully functional
- LLM processing, transcription, and people management already implemented
- Core logic can be reused - TUI is a new input interface

## Success Criteria
- [ ] User can launch TUI with `cargo run --tui` or similar flag
- [ ] Text input is processed through existing LLM pipeline
- [ ] Responses display in scrolling feed
- [ ] Input clears after each submission
- [ ] Clean exit with Ctrl+C or 'q'

## Constraints
- Must use Ratatui for TUI
- Must integrate with existing processing pipeline (no duplication)
- TDD approach required
