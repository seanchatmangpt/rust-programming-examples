# Modular Architecture Patterns

Modular architecture is the foundation of maintainable, scalable software systems. This chapter explores how to organize code, manage dependencies, and design for evolution.

## Why Modularity Matters

Modular code provides several critical benefits:

- **Maintainability**: Changes to one module rarely cascade to others
- **Testability**: Modules can be tested in isolation
- **Reusability**: Well-designed modules serve multiple contexts
- **Team Scalability**: Different teams can own different modules
- **Compilation Speed**: Only modified modules need recompilation

## Cohesion and Coupling

**Cohesion** measures how closely related the elements within a module are. High cohesion means a module does one thing well.

**Coupling** measures the degree of interdependence between modules. Low coupling means modules can change independently.

## Module Organization

### Single-File vs Multi-File Crates

**Single-file modules** work well for small, focused functionality:

```rust
// src/lib.rs - Everything in one file
pub mod parser {
    pub fn parse(input: &str) -> Result<Ast, ParseError> {
        // Implementation
    }

    #[cfg(test)]
    mod tests {
        // Tests alongside code
    }
}
```

**Multi-file modules** scale better for complex systems:

```
src/
├── lib.rs              # Crate root
├── parser/
│   ├── mod.rs          # Parser module root
│   ├── lexer.rs        # Tokenization
│   └── ast.rs          # Abstract syntax tree
├── compiler/
│   ├── mod.rs          # Compiler module root
│   └── codegen.rs      # Code generation
└── runtime/
    └── vm.rs           # Virtual machine
```

### Public API Boundaries

Define clear public interfaces:

```rust
// src/lib.rs
pub mod api;           // Fully public module
mod internal;          // Private to crate
pub(crate) mod shared; // Visible within crate only

pub use api::Request;  // Re-export
pub use api::Response;
```

### Visibility Levels

| Visibility | Syntax | Scope |
|------------|--------|-------|
| Public | `pub` | Anywhere |
| Crate | `pub(crate)` | Within crate |
| Parent | `pub(super)` | Parent module |
| Private | (default) | Current module |

## Dependency Management

### Reducing Circular Dependencies

When modules create circular dependencies, extract shared types into a base module.

**Strategy**: Move shared types to a common module both depend on.

### Dependency Injection Patterns

```rust
pub trait Database: Send + Sync {
    fn query(&self, sql: &str) -> Result<Vec<Row>, DbError>;
}

pub struct UserService<D: Database> {
    db: D,
}

impl<D: Database> UserService<D> {
    pub fn new(db: D) -> Self {
        Self { db }
    }
}
```

### Feature Flags for Optional Modules

```toml
[features]
default = ["json"]
json = ["serde_json"]
yaml = ["serde_yaml"]
```

```rust
#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "yaml")]
pub mod yaml;
```

### Workspace Organization

For large projects, use Cargo workspaces:

```toml
[workspace]
members = [
    "crates/core",
    "crates/parser",
    "crates/cli",
]
```

## Breaking Monolithic Structures

Transform large, tightly coupled code into modular components:

```rust
// BEFORE: Monolithic struct
pub struct Application {
    config: Config,
    database: Database,
    cache: Cache,
    http_client: HttpClient,

    pub fn handle_request(&mut self, req: Request) { /* 500 lines */ }
    pub fn process_data(&mut self, data: Data) { /* 300 lines */ }
}

// AFTER: Separated concerns
pub mod config { /* ... */ }
pub mod storage { /* ... */ }
pub mod http { /* ... */ }

pub struct Application<S: Storage> {
    config: Config,
    storage: S,
    client: HttpClient,
}
```

## Module Communication

### Error Propagation Across Modules

```rust
pub mod parser {
    #[derive(Error, Debug)]
    pub enum ParseError {
        #[error("unexpected token")]
        UnexpectedToken,
    }
}

pub mod compiler {
    use super::parser;

    #[derive(Error, Debug)]
    pub enum CompileError {
        #[error("parse error: {0}")]
        Parse(#[from] parser::ParseError),
    }
}
```

### Message Passing

Use channels for decoupled communication:

```rust
use std::sync::mpsc;

pub enum Message {
    Process(String),
    Shutdown,
}

pub fn spawn_worker(rx: mpsc::Receiver<Message>) {
    while let Ok(msg) = rx.recv() {
        match msg {
            Message::Process(data) => { /* process */ }
            Message::Shutdown => break,
        }
    }
}
```

## Testing Modular Systems

### Integration Testing

```rust
#[test]
fn test_full_command_flow() {
    let parser = parser::parse(...).unwrap();
    let registry = commands::Registry::new();
    let cmd = registry.get(...).unwrap();
    cmd.execute(...).unwrap();
}
```

## Best Practices

### Right-Sizing Modules

- **Too small**: One function per module creates navigation overhead
- **Too large**: Defeats the purpose of modularity
- **Just right**: 100-500 lines per module

### Avoiding Deep Hierarchies

Limit module depth to maintain navigability.

### Documentation for Modules

```rust
//! # Parser Module
//!
//! This module handles command-line argument parsing.
//!
//! ## Architecture
//! [architectural description]
//!
//! ## Usage
//! [usage examples]
```

## Summary

Modular architecture requires upfront investment but pays dividends in maintainability, testability, and team productivity. Start with clear boundaries, evolve based on real usage patterns, and refactor when modules grow beyond their intended scope.

Key principles:
- Each module has a single, clear responsibility
- Public APIs are minimal and stable
- Dependencies flow in one direction
- Modules can be tested in isolation
- Error types compose across boundaries
