# A Beginner's Tour of Rust: The Courage Codebase

## Part 1: Project Structure & Cargo.toml

Let's start with `Cargo.toml` - this is Rust's package manifest (like `package.json` in Node.js):

```toml
[package]
name = "courage"
version = "0.1.0"
edition = "2021"   # Rust editions - 2021 is the latest stable
```

**Edition** is unique to Rust. Every 3 years, Rust releases a new "edition" that can introduce breaking changes. Your code opts into an edition, so old code keeps working.

Dependencies are declared like this:
```toml
tokio = { version = "1", features = ["full"] }
```

The `features` array enables optional functionality. Rust crates (packages) use **feature flags** to let you include only what you need - this keeps binaries small.

---

## Part 2: main.rs - The Entry Point

```rust
mod config;
mod llm;
mod people;
// ... more modules
```

### `mod` - Module Declarations

`mod config;` tells Rust: "Look for a file called `config.rs` or a folder `config/mod.rs` and include it as a module." This is how Rust organizes code into namespaces.

```rust
use anyhow::Result;
use clap::Parser;
```

### `use` - Bringing Items Into Scope

`use` imports items so you don't have to write the full path every time. Without it, you'd write `anyhow::Result` everywhere instead of just `Result`.

```rust
/// Courage - Personal Relationship Database with Voice Input
#[derive(Parser, Debug)]
#[command(name = "courage")]
struct Args {
    /// Run in TUI mode instead of Telegram bot mode
    #[arg(long)]
    tui: bool,
}
```

### Doc Comments (`///`)

Triple-slash comments are **documentation comments**. They become part of the generated documentation (run `cargo doc` to see).

### `#[derive(...)]` - Procedural Macros

This is one of Rust's most powerful features. `#[derive(Parser, Debug)]` automatically generates code:
- `Parser` (from clap) - generates CLI argument parsing code
- `Debug` - generates code to print the struct for debugging (`{:?}` format)

The compiler literally writes the boring boilerplate for you!

### `#[command(...)]` and `#[arg(...)]` - Attribute Macros

These configure how `clap` parses arguments. `#[arg(long)]` means `--tui` flag (long form).

```rust
#[tokio::main]
async fn main() -> Result<()> {
```

### `#[tokio::main]` - The Async Runtime

Rust doesn't have a built-in async runtime. `#[tokio::main]` transforms your `async fn main()` into a regular `fn main()` that sets up the Tokio runtime. It's macro magic that expands roughly to:

```rust
fn main() -> Result<()> {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { /* your code */ })
}
```

### `async fn` and `Result<()>`

- `async fn` - This function can use `.await` and be paused/resumed
- `Result<()>` - Returns either `Ok(())` (success, no value) or an error. The `()` is Rust's "unit type" (like `void` but it's an actual type)

```rust
let args = Args::parse();
```

### `let` - Variable Binding

Variables are **immutable by default** in Rust. This is intentional - it prevents bugs. Use `let mut` for mutable variables.

```rust
let log_filter = if args.tui {
    "courage=warn"
} else {
    "courage=info,teloxide=warn"
};
```

### `if` as an Expression

In Rust, `if` returns a value! This is called an **expression**. The last value in each branch (no semicolon) becomes the result. This is more elegant than:
```rust
let log_filter;
if args.tui {
    log_filter = "courage=warn";
} else {
    log_filter = "courage=info,teloxide=warn";
}
```

```rust
tracing_subscriber::registry()
    .with(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| log_filter.into()),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();
```

### Method Chaining & The Builder Pattern

This is a common Rust idiom. Each method returns `self` (or a modified version), allowing you to chain calls. It reads like: "Create a registry, add an env filter, add a formatting layer, then initialize."

### `.unwrap_or_else(|_| ...)` - Handling Results

- `try_from_default_env()` returns a `Result`
- `unwrap_or_else` says: "If it's an error, run this closure instead"
- `|_|` is a closure (anonymous function). The `_` means "I'm ignoring this parameter"
- `.into()` converts the string into the expected type (Rust figures out which type)

```rust
let config = Config::from_env()?;
```

### The `?` Operator - Error Propagation

This is Rust's elegant error handling. `?` means:
- If `from_env()` returns `Ok(value)`, unwrap it and continue
- If it returns `Err(e)`, **immediately return** that error from the current function

It's shorthand for:
```rust
let config = match Config::from_env() {
    Ok(c) => c,
    Err(e) => return Err(e),
};
```

```rust
tokio::fs::create_dir_all(&config.people_dir).await?;
```

### `.await` - Waiting for Async Operations

`await` pauses the function until the async operation completes. The runtime can run other tasks while waiting. The `&` borrows `people_dir` (we'll explain borrowing soon).

```rust
if args.tui {
    run_tui(config)?;
} else {
    let bot = CourageBot::new(config);
    bot.run().await?;
}

Ok(())
```

### `Ok(())` - Explicit Success

Functions returning `Result` must explicitly return success. `Ok(())` means "success with no value."

---

## Part 3: config.rs - Structs and Impl Blocks

```rust
use anyhow::{Context, Result};
use std::path::PathBuf;
```

### Standard Library Imports

`std::path::PathBuf` is Rust's cross-platform path type. It handles `/` vs `\` automatically.

```rust
#[derive(Debug, Clone)]
pub struct Config {
    /// Telegram bot token
    pub telegram_token: String,
    pub openai_api_key: String,
    pub people_dir: PathBuf,
    pub llm_model: String,
}
```

### `pub` - Visibility

By default, everything in Rust is **private**. `pub` makes it public:
- `pub struct Config` - the struct is public
- `pub telegram_token` - the field is public

### `#[derive(Clone)]`

This generates a `.clone()` method. In Rust, assignment **moves** values by default (we'll see why). `Clone` lets you explicitly copy when needed.

### `String` vs `&str`

Rust has two main string types:
- `String` - owned, growable, heap-allocated
- `&str` - borrowed reference to string data (a "string slice")

Structs usually use `String` because they need to **own** their data.

```rust
impl Config {
    pub fn from_env() -> Result<Self> {
```

### `impl` Blocks - Adding Methods to Structs

`impl Config { ... }` adds methods to the `Config` struct. This is how Rust does object-oriented programming (without inheritance).

### `Self` - The Implementing Type

Inside `impl Config`, `Self` is an alias for `Config`. It's useful when the type name is long or might change.

```rust
        dotenvy::dotenv().ok();
```

### `.ok()` - Discarding Errors

`.ok()` converts `Result<T, E>` to `Option<T>`, discarding any error. Here we don't care if `.env` doesn't exist.

```rust
        let telegram_token =
            std::env::var("TELEGRAM_BOT_TOKEN").context("TELEGRAM_BOT_TOKEN must be set")?;
```

### `.context()` - Adding Error Context

From `anyhow`, `.context()` wraps an error with additional information. When the error prints, you'll see "TELEGRAM_BOT_TOKEN must be set" along with the original error.

```rust
        let people_dir = std::env::var("PEOPLE_DIR").unwrap_or_else(|_| "./People".to_string());
        let people_dir = PathBuf::from(people_dir);
```

### Variable Shadowing

Rust lets you redeclare a variable with the same name. This is called **shadowing**. It's useful for transformations:
1. First `people_dir` is a `String`
2. Second `people_dir` shadows it as a `PathBuf`

This is idiomatic and clearer than using different names like `people_dir_str` and `people_dir_path`.

```rust
        Ok(Self {
            telegram_token,
            openai_api_key,
            people_dir,
            llm_model,
        })
```

### Field Init Shorthand

When a variable has the same name as a field, you can write just `telegram_token` instead of `telegram_token: telegram_token`.

---

## Part 4: people/types.rs - Serde and Traits

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersonFrontmatter {
    pub first_name: String,
    pub last_name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub disambiguator: String,
```

### `Serialize` and `Deserialize` - The Serde Framework

Serde is Rust's serialization framework. These derive macros generate code to convert your struct to/from JSON, YAML, TOML, etc.

### `#[serde(...)]` - Field-Level Configuration

- `default` - Use `Default::default()` if the field is missing when deserializing
- `skip_serializing_if = "String::is_empty"` - Don't include this field in output if it's empty

This keeps your YAML/JSON clean - empty fields are omitted.

### `Default` Trait

`#[derive(Default)]` generates a `default()` method. For `String`, default is `""`. For `Vec`, it's `[]`. For numbers, it's `0`.

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Relationships {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub parents: String,
    // ...
}
```

```rust
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields are part of public API
pub struct Person {
    pub filename: String,
    pub frontmatter: PersonFrontmatter,
    pub relationships: Relationships,
    pub content: String,
}
```

### `#[allow(dead_code)]`

Rust warns about unused code. This attribute suppresses that warning. The comment explains why - these fields are part of the public API even if not used internally.

```rust
impl Person {
    pub fn display_name(&self) -> String {
```

### `&self` - Method Receiver

`&self` means this method borrows the struct immutably. There are three forms:
- `&self` - immutable borrow (read-only access)
- `&mut self` - mutable borrow (read-write access)
- `self` - takes ownership (consumes the struct)

```rust
        let base = format!(
            "{} {}",
            self.frontmatter.first_name, self.frontmatter.last_name
        );
```

### `format!` Macro

Like `println!` but returns a `String` instead of printing. The `{}` is a placeholder that uses the `Display` trait.

```rust
        if self.frontmatter.disambiguator.is_empty() {
            base
        } else {
            format!("{} ({})", base, self.frontmatter.disambiguator)
        }
```

### Implicit Returns

The last expression in a function (without a semicolon) is the return value. No `return` keyword needed!

```rust
impl PeopleContext {
    pub fn to_prompt_list(&self) -> String {
        if self.people.is_empty() {
            return "(No people in database yet)".to_string();
        }

        self.people
            .iter()
            .map(|p| format!("- {}", p.summary()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
```

### Iterators and Functional Programming

This is idiomatic Rust! Let's break it down:

1. `.iter()` - creates an iterator over `&Person` references
2. `.map(|p| ...)` - transforms each person into a string
3. `.collect::<Vec<_>>()` - collects into a `Vec`. The `_` means "infer the element type"
4. `.join("\n")` - joins the strings with newlines

### Turbofish `::<>`

`::<Vec<_>>` is called the "turbofish" (because it looks like a fish: `::<>`). It provides type hints when Rust can't infer the type.

---

## Part 5: state/conversation.rs - Concurrency & Smart Pointers

```rust
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
```

### Concurrency Primitives

- `Arc` - Atomically Reference Counted pointer (shared ownership across threads)
- `RwLock` - Reader-Writer Lock (multiple readers OR one writer)

```rust
#[derive(Debug, Clone)]
pub struct PendingConversation {
    pub transcript: String,
    pub question: String,
    pub created_at: DateTime<Utc>,
    pub chat_id: i64,
}
```

### Primitive Types

`i64` is a signed 64-bit integer. Rust has explicit sizes:
- `i8`, `i16`, `i32`, `i64`, `i128` - signed integers
- `u8`, `u16`, `u32`, `u64`, `u128` - unsigned integers
- `f32`, `f64` - floats
- `isize`, `usize` - pointer-sized (for indexing)

```rust
#[derive(Debug, Clone)]
pub struct ConversationState {
    pending: Arc<RwLock<HashMap<i64, PendingConversation>>>,
    timeout: Duration,
}
```

### `Arc<RwLock<T>>` - Thread-Safe Shared State

This is a common pattern for shared mutable state:

1. `HashMap<i64, PendingConversation>` - the actual data
2. `RwLock<...>` - protects the HashMap from data races
3. `Arc<...>` - allows multiple owners (cloning shares the same data)

When you `.clone()` a `ConversationState`, both copies point to the **same** HashMap!

```rust
impl ConversationState {
    pub fn new() -> Self {
        Self {
            pending: Arc::new(RwLock::new(HashMap::new())),
            timeout: Duration::hours(1),
        }
    }
```

### Constructors

Rust doesn't have constructors. By convention, we use `new()` or `from_*()` methods.

```rust
    pub async fn set_pending(&self, chat_id: i64, transcript: String, question: String) {
        let conversation = PendingConversation {
            transcript,
            question,
            created_at: Utc::now(),
            chat_id,
        };

        let mut pending = self.pending.write().await;
        pending.insert(chat_id, conversation);
    }
```

### Async Lock Acquisition

`.write().await` acquires an exclusive write lock. This is async because it might need to wait for other tasks to release the lock.

### Struct Initialization with Field Shorthand

Notice `transcript` instead of `transcript: transcript`. When the variable name matches the field name, you can use this shorthand.

```rust
    pub async fn take_pending(&self, chat_id: i64) -> Option<PendingConversation> {
        let mut pending = self.pending.write().await;

        if let Some(conversation) = pending.remove(&chat_id) {
```

### `Option<T>` - Nullable Values

Rust has no `null`. Instead, `Option<T>` is either:
- `Some(value)` - contains a value
- `None` - no value

### `if let` - Pattern Matching

`if let Some(conversation) = ...` is pattern matching. It means: "If this is `Some`, extract the value into `conversation`."

```rust
            if Utc::now() - conversation.created_at < self.timeout {
                return Some(conversation);
            }
            tracing::debug!("Pending conversation for {} expired", chat_id);
        }

        None
    }
```

### Implicit `None` Return

The function returns `Option<...>`. If we don't return `Some(...)`, we fall through to `None`.

```rust
    pub async fn cleanup_expired(&self) {
        let mut pending = self.pending.write().await;
        let now = Utc::now();

        pending.retain(|chat_id, conv| {
            let expired = now - conv.created_at >= self.timeout;
            if expired {
                tracing::debug!("Cleaning up expired conversation for {}", chat_id);
            }
            !expired
        });
    }
```

### `.retain()` - Filtering In Place

`retain` keeps only elements where the closure returns `true`. It's like `filter` but modifies in place.

### Closure Syntax

`|chat_id, conv| { ... }` is a closure. The parameters are between `|...|`. Rust infers their types from context.

```rust
impl Default for ConversationState {
    fn default() -> Self {
        Self::new()
    }
}
```

### Implementing Traits Manually

Here we implement the `Default` trait by calling `new()`. This lets users write `ConversationState::default()`.

---

## Part 6: llm/types.rs - Enums and Type Conversions

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Ready,
    ClarificationNeeded,
    Error,
}
```

### Enums - Sum Types

Rust enums are **algebraic data types**. Each variant is a distinct state. Unlike C enums, Rust enums can hold data (we'll see that later).

### `#[serde(rename_all = "snake_case")]`

When serializing, `ClarificationNeeded` becomes `"clarification_needed"`. This matches JSON conventions.

### `PartialEq` - Equality Comparison

This derive lets you use `==` and `!=` with the enum.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonAction {
    #[serde(rename = "type")]
    pub action_type: ActionType,
    pub filename: String,
    #[serde(default)]
    pub fields: HashMap<String, String>,
```

### `#[serde(rename = "type")]`

The JSON uses `"type"` but `type` is a reserved keyword in Rust. This renames the field for serialization only.

```rust
impl From<ActionRelationships> for Relationships {
    fn from(ar: ActionRelationships) -> Self {
        Relationships {
            parents: ar.parents.unwrap_or_default(),
            children: ar.children.unwrap_or_default(),
            // ...
        }
    }
}
```

### The `From` Trait - Type Conversion

Implementing `From<A> for B` lets you convert `A` to `B`. You get:
- `B::from(a)` - explicit conversion
- `a.into()` - when Rust knows the target type (also implements `Into` automatically)

### `.unwrap_or_default()`

For `Option<String>`:
- `Some("hello")` → `"hello"`
- `None` → `""` (the default for String)

---

## Part 7: tui/app.rs - Ratatui and Event Handling

```rust
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
```

### Nested Imports

You can import multiple items from nested modules in one statement.

```rust
#[derive(Debug)]
pub struct App {
    #[allow(dead_code)]
    config: Config,
    should_quit: bool,
}
```

### Private Fields by Default

Note there's no `pub` on these fields. They're private to this module. Only methods can access them.

```rust
    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }
```

### `&mut self` - Mutable Borrow

This method can modify `self.should_quit`. The `&mut terminal` parameter is also mutably borrowed.

### Closure Capturing `self`

`|frame| self.draw(frame)` is a closure that captures `self`. Rust figures out that it needs `&self` (immutable borrow).

```rust
    fn draw(&self, frame: &mut Frame) {
        let chunks = Layout::vertical([
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(frame.area());
```

### Array Syntax

`[Constraint::Min(3), Constraint::Length(3)]` is an array literal. Arrays have fixed size known at compile time.

```rust
        let feed = Paragraph::new(Text::raw("Welcome to Courage TUI\n\nPress 'q' to quit"))
            .block(
                Block::default()
                    .title(" Feed ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        frame.render_widget(feed, chunks[0]);
```

### Builder Pattern

Ratatui uses the builder pattern extensively:
1. `Block::default()` - start with defaults
2. `.title(...)` - set the title
3. `.borders(...)` - add borders
4. `.border_style(...)` - style the borders

Each method returns `Self` for chaining.

```rust
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
```

### `match` - Pattern Matching

`match` is Rust's powerful pattern matching. It must be **exhaustive** - you must handle all cases. `_ => {}` is the catch-all that does nothing.

### `KeyCode::Char('q')` - Enum with Data

This shows enums holding data. `Char` variant contains a `char` value.

---

## Part 8: utils/http.rs - Async and Generics

```rust
pub async fn handle_api_response(response: Response, api_name: &str) -> Result<Response> {
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("{} API error {}: {}", api_name, status, error_text);
    }
    Ok(response)
}
```

### `&str` Parameter

`&str` is a string slice - a borrowed reference to string data. It's more flexible than `String` because you can pass:
- `&String` (automatically coerces)
- String literals `"like this"`
- Substrings

### `anyhow::bail!` Macro

`bail!` is a macro that creates an error and returns early. It's equivalent to:
```rust
return Err(anyhow::anyhow!("{} API error {}: {}", api_name, status, error_text));
```

```rust
pub async fn parse_json_response<T: serde::de::DeserializeOwned>(
    response: Response,
    context: &str,
) -> Result<T> {
    response
        .json()
        .await
        .context(format!("Failed to parse {}", context))
}
```

### Generics with Trait Bounds

`<T: serde::de::DeserializeOwned>` means:
- `T` is a generic type parameter
- `T` must implement the `DeserializeOwned` trait

This function works with any type that can be deserialized from JSON!

### Returning Generic Types

`-> Result<T>` - the caller decides what type `T` is based on how they use the result:

```rust
let config: Config = parse_json_response(resp, "config").await?;
let users: Vec<User> = parse_json_response(resp, "users").await?;
```

---

## Part 9: Tests - The `#[cfg(test)]` Module

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_person_display_name_simple() {
        let person = Person {
            filename: "John Smith".to_string(),
            frontmatter: PersonFrontmatter {
                first_name: "John".to_string(),
                last_name: "Smith".to_string(),
                ..Default::default()
            },
            // ...
        };
        assert_eq!(person.display_name(), "John Smith");
    }
```

### `#[cfg(test)]` - Conditional Compilation

This module only compiles when running tests (`cargo test`). It's not included in the release binary.

### `use super::*`

`super` refers to the parent module. `*` imports everything from it.

### `..Default::default()` - Struct Update Syntax

This fills remaining fields with their default values. Super handy for tests where you only care about certain fields.

### `#[test]` Attribute

Marks a function as a test. Run with `cargo test`.

### `assert_eq!` Macro

Asserts two values are equal. If not, the test fails with a helpful diff.

```rust
    #[tokio::test]
    async fn test_set_and_take_pending() {
        let state = ConversationState::new();

        state
            .set_pending(
                12345,
                "John likes pizza".to_string(),
                "Which John?".to_string(),
            )
            .await;

        assert!(state.has_pending(12345).await);
```

### `#[tokio::test]` - Async Tests

For testing async functions, use `#[tokio::test]` instead of `#[test]`. It sets up a Tokio runtime for the test.

---

## Part 10: The lib.rs File - Public API

```rust
//! Courage - Personal Relationship Database with Voice Input
//!
//! This library provides the core functionality...

pub mod config;
pub mod llm;
pub mod people;
// ...
```

### Module Doc Comments (`//!`)

`//!` comments document the containing item (the module/crate itself), not the next item.

### `pub mod` - Public Modules

`pub mod config` makes the entire `config` module public. External crates can access `courage::config::Config`.

---

## Key Rust Concepts Summary

### Ownership & Borrowing
- Every value has exactly one owner
- When the owner goes out of scope, the value is dropped (freed)
- You can **borrow** with `&` (immutable) or `&mut` (mutable)
- You can have many `&` OR one `&mut`, never both

### The Type System
- Strong, static typing with inference
- Enums can hold data (algebraic data types)
- Traits define shared behavior (like interfaces)
- Generics with trait bounds for type-safe polymorphism

### Error Handling
- No exceptions - use `Result<T, E>` and `Option<T>`
- `?` operator for ergonomic error propagation
- `anyhow` for application code, `thiserror` for libraries

### Async/Await
- First-class async support
- Needs a runtime (Tokio is most popular)
- `.await` suspends until completion

### Memory Safety Without GC
- No garbage collector - deterministic cleanup
- Compile-time checks prevent data races
- Zero-cost abstractions

---

This codebase is a great example of idiomatic Rust. It uses:
- Proper error handling with `anyhow` and `?`
- Async/await for I/O operations
- Serde for serialization
- Builder patterns for configuration
- Tests alongside the code
- Clear module organization
