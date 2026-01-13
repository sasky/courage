# Courage TUI - Roadmap

## Milestone: v1.0 - TUI Input Interface

### Phase 01: Foundation
**Goal:** Set up Ratatui dependencies and basic app structure

- Add ratatui and crossterm to Cargo.toml
- Create TUI module structure (`src/tui/`)
- Implement basic app loop with terminal setup/teardown
- Add CLI flag to choose TUI vs Telegram mode

**Deliverable:** App launches in TUI mode, shows empty screen, exits cleanly

---

### Phase 02: UI Layout
**Goal:** Implement the two-panel layout (feed + input)

- Create input widget at bottom of screen
- Create response feed widget at top
- Handle basic keyboard input (typing, backspace, enter)
- Implement layout with proper sizing

**Deliverable:** User sees split layout, can type text (no processing yet)

---

### Phase 03: Input Processing
**Goal:** Connect TUI input to existing LLM pipeline

- Extract shared processing logic from Telegram handler
- Create common `MessageProcessor` trait/interface
- Wire TUI input to processor
- Display responses in feed

**Deliverable:** Text input is processed, responses appear in feed

---

### Phase 04: Polish
**Goal:** Refine UX and add finishing touches

- Input clears after submission
- Feed scrolls properly with history
- Loading indicator while processing
- Error display for failed requests
- Keyboard shortcuts (Ctrl+C to quit)

**Deliverable:** Polished, usable TUI experience

---

## Status Tracking

| Phase | Status | Plan | Summary |
|-------|--------|------|---------|
| 01-foundation | in_progress | [01-01-PLAN](phases/01-foundation/01-01-PLAN.md) | [01-01-SUMMARY](phases/01-foundation/01-01-SUMMARY.md) |
| 02-ui-layout | pending | - | - |
| 03-input-processing | pending | - | - |
| 04-polish | pending | - | - |
