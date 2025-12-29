# Documentation & DX During Refactoring

Refactoring a Rust framework is fundamentally a communication challenge. When you restructure APIs, rename modules, or redesign abstractions, you create a knowledge gap between what users knew and what they need to learn.

## Why DX Matters During Refactoring

Poor documentation during refactoring leads to:
- User frustration and abandonment
- Increased support burden on maintainers
- Fragmented community knowledge
- Slower adoption of improved APIs

Excellent documentation enables:
- Smooth migration paths for existing users
- Clear understanding of why changes were made
- Confidence in the framework's stability
- Community contributions aligned with new architecture

## Documentation Types

### API Documentation

Rust's built-in documentation through `///` comments serves dual duty:

```rust
/// Executes a command with the noun-verb pattern.
///
/// This method replaces the legacy `run_command` function, providing
/// better type safety and clearer semantics.
///
/// # Examples
///
/// ```rust
/// use myframework::Command;
///
/// let result = Command::new("user")
///     .verb("create")
///     .arg("--name", "alice")
///     .execute()?;
/// ```
///
/// # Migration from `run_command`
///
/// If you were previously using `run_command(...)`,
/// the equivalent with the new API is shown above.
///
/// # Errors
///
/// Returns [`CommandError::InvalidNoun`] if the noun is not registered.
pub fn execute(&self) -> Result<Output, CommandError> {
    // implementation
}
```

### Architecture Documentation

```markdown
# Architecture: Noun-Verb Command Pattern

## Overview

The noun-verb pattern organizes commands around resources (nouns)
and actions (verbs).

## Design Rationale

The flat command structure had several problems:

1. **Discoverability**: Users couldn't easily find related commands
2. **Consistency**: Different commands used different naming
3. **Extensibility**: Adding resources required many commands

## The New Model

[diagrams and examples]

## Migration Impact

[affected areas]
```

### Migration Guides

A migration guide is the most critical documentation artifact:

```markdown
# Migration Guide: v3.x to v4.0

## Overview

Version 4.0 introduces the noun-verb command pattern.
**Estimated migration time**: 30 minutes for small CLIs

## Quick Reference

| v3.x Pattern | v4.0 Pattern | Notes |
|--------------|--------------|-------|
| `App::new()` | `App::new()` | Same |
| `.subcommand(...)` | `.noun(...)` | New structure |

## Step-by-Step Migration

### Step 1: Update Command Structure

**Before (v3.x):**
```rust
let app = App::new("mycli")
    .subcommand(SubCommand::with_name("create"));
```

**After (v4.0):**
```rust
let app = App::new("mycli")
    .noun(Noun::new("user").verb(Verb::new("create")));
```
```

### Changelog

Maintain CHANGELOG.md following Keep a Changelog format:

```markdown
## [4.0.0] - 2025-01-15

### Added
- Noun-verb command pattern

### Changed
- **BREAKING**: `SubCommand` renamed to `Verb`
- Minimum Rust version updated to 1.70.0

### Deprecated
- `Arg::with_name()` - use `Arg::new()` instead

### Removed
- `App::subcommand_matches()`
```

## Maintaining DX During Refactoring

### Compile-Tested Documentation

```rust
/// Creates a new noun with the given name.
///
/// ```rust
/// # use myframework::Noun;
/// let noun = Noun::new("user");
/// assert_eq!(noun.name(), "user");
/// ```
pub fn new(name: &str) -> Self {
    // If this API changes, the doctest fails
}
```

### Deprecation Notices

```rust
#[deprecated(
    since = "4.0.0",
    note = "Use App::noun() instead. See migration guide."
)]
pub fn subcommand(self, subcmd: SubCommand) -> Self {
    // Shim implementation using new API
    self.noun(Noun::from_legacy(subcmd))
}
```

### Error Messages as Documentation

```rust
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error(
        "Unknown command '{command}'.\n\n\
         Hint: Commands now use noun-verb structure.\n\
         Instead of 'user-create', try 'user create'.\n\n\
         Available nouns: {available_nouns}"
    )]
    UnknownCommand {
        command: String,
        available_nouns: String,
    },
}
```

## The Diataxis Framework

The Diataxis framework organizes documentation into four types:

### Tutorials (Learning-Oriented)

```markdown
# Tutorial: Building Your First Noun-Verb CLI

In this tutorial, you'll build a simple CLI tool that manages users.

## Prerequisites

- Rust 1.70 or later
- Basic familiarity with Cargo

## Step 1: Create Your Project

```bash
cargo new userctl
cd userctl
```

[steps continue...]
```

### How-To Guides (Task-Oriented)

```markdown
# How to: Migrate Subcommand Matching

## The Problem

In v3.x, you matched subcommands like this:
```rust
match matches.subcommand() {
    ("create", Some(sub_m)) => handle_create(sub_m),
    _ => show_help(),
}
```

## The Solution

Use the new `route` method:
```rust
app.route(|noun, verb, matches| {
    match (noun, verb) {
        ("user", "create") => handle_user_create(matches),
        _ => show_help(),
    }
});
```
```

### Explanations (Understanding-Oriented)

Explain why changes were made and what problems they solve.

### Reference (Information-Oriented)

Exhaustive, accurate reference documentation.

## Code Examples

### Runnable Examples

```
examples/
├── 01_basic_cli.rs
├── 02_noun_verb.rs
├── 03_arguments.rs
└── migration_from_v3.rs
```

### Compile-Fail Examples

```rust
//! This example demonstrates a common mistake.
//!
//! ```compile_fail
//! use myframework::Verb;
//!
//! // ERROR: Cannot add verb without noun
//! let app = App::new("example")
//!     .verb(Verb::new("create")); // This won't compile!
//! ```
```

## Tool Integration

### cargo doc Configuration

```toml
[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

### mdBook Setup

Structure documentation book with progression from basics to advanced topics.

## Best Practices

### Progressive Disclosure

Layer information from simple to complex.

### Learning Paths

Guide users through progressive learning:

```markdown
## Learning Path

### Beginner (30 minutes)
1. [Quick Start](./quickstart.md)
2. [Tutorial: User Manager](./tutorials/user-manager.md)

### Intermediate (2 hours)
3. [Understanding Noun-Verb](./explanation/noun-verb.md)
4. [How-To Guides](./howto/)

### Advanced (4+ hours)
5. [Architecture](./explanation/architecture.md)
6. [Reference: Full API](./reference/api.md)
```

## Summary

Documentation during refactoring is not an afterthought—it is an integral part of the refactoring itself. By treating documentation as a first-class deliverable, you:

1. **Reduce user friction** during transition
2. **Communicate intent** behind decisions
3. **Build trust** through clear timelines
4. **Enable contributions** aligned with new design

The investment in documentation pays dividends in reduced support burden, faster adoption, and a healthier ecosystem.
