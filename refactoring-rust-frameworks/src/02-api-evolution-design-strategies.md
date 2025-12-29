# API Evolution & Design Strategies

A comprehensive guide to designing, evolving, and maintaining Rust APIs for long-term success. This chapter focuses on how to improve APIs during refactoring while maintaining backward compatibility with existing users.

## The Cost of Poor API Design

API design is one of the most consequential decisions a framework developer makes. A poorly designed API accumulates technical debt not just in your codebase, but in every project that depends on it.

**Real-world impact metrics:**
- Each breaking change in a widely-used crate affects hundreds of downstream projects
- Poor API ergonomics lead to 3-5x more support questions
- Migration guides for breaking changes require 10-20x the effort of the original change

## Principles of Good API Design

Effective Rust APIs share common characteristics:

**Consistency** reduces cognitive load. **Discoverability** enables users to find functionality through IDE autocompletion. **Intuitiveness** ensures APIs behave as expected. **Extensibility** allows adding new functionality without breaking existing code.

## Refactoring Patterns for APIs

### Derive Macros for Reducing Boilerplate

Custom derive macros eliminate repetitive code and ensure consistency. Instead of manually implementing traits for each type, derive macros generate the boilerplate automatically.

### Builder Pattern Evolution

The builder pattern allows API evolution without breaking changes:

```rust
// Stage 1: Simple constructor
impl Client {
    pub fn new(url: &str) -> Self { /* ... */ }
}

// Stage 2: Add builder for optional configuration
impl Client {
    pub fn new(url: &str) -> Self { /* ... */ }  // Preserved!
    pub fn builder() -> ClientBuilder { /* ... */ }
}

// Stage 3: Type-safe builder (prevents invalid states)
pub struct ClientBuilder<State> {
    url: Option<String>,
    timeout: Option<Duration>,
    _state: PhantomData<State>,
}
```

### Type-State Patterns for Safety

Type-state patterns encode state machines in the type system, preventing invalid operations at compile time.

### Trait-Based Abstraction and Polymorphism

Traits enable API evolution by allowing new implementations without modifying existing code.

### Feature Flags for API Expansion

Cargo features allow optional API extensions:

```toml
[features]
default = []
async = ["tokio", "async-trait"]
serde = ["dep:serde"]
full = ["async", "serde"]
```

## Breaking Changes & Deprecation

### Identifying When Breaking Changes Are Necessary

**Valid reasons for breaking changes:**
- Security vulnerabilities that cannot be fixed otherwise
- Fundamental design flaws causing widespread issues
- Alignment with ecosystem standards
- Performance improvements requiring API restructuring

### Deprecation Timeline Strategies

A well-planned deprecation follows a predictable timeline with grace periods allowing users time to migrate.

### Migration Guides and Automation Tools

Effective migration requires documentation and tooling:

```markdown
# Migrating from v1.x to v2.0

## Quick Reference

| v1.x Pattern | v2.0 Pattern | Notes |
|--------------|--------------|-------|
| `Client::new(config)` | `Client::builder().config(c).build()` | Structural change |
| `client.send(req)` | `client.execute(req)` | Method rename |
| `Response.body` field | `Response.body()` method | Field to method |
```

## Semantic Versioning

### MAJOR.MINOR.PATCH Semantics

```
MAJOR.MINOR.PATCH
  │     │     │
  │     │     └── Bug fixes (backward compatible)
  │     │
  │     └──────── New features (backward compatible)
  │
  └────────────── Breaking changes
```

### SemVer in Practice

- **MAJOR**: Breaking changes requiring code updates
- **MINOR**: New features that don't break existing code
- **PATCH**: Bug fixes with no API changes

## Pre-releases and Stability

Pre-release versions signal instability:

```toml
version = "2.0.0-alpha.1"  # No stability guarantees
version = "2.0.0-beta.3"   # Feature-complete, testing
version = "2.0.0-rc.1"     # Release candidate
```

## Case Study: clap-noun-verb API Improvements

The transition from imperative command definition to declarative derive-based design demonstrates effective API evolution.

### Initial API (v1.0)

```rust
// Manual, imperative command definition
let matches = App::new("myapp")
    .subcommand(
        SubCommand::with_name("user")
            .subcommand(
                SubCommand::with_name("create")
                    .arg(Arg::with_name("name").required(true))
            )
    )
    .get_matches();
```

**Problems:**
- Verbose boilerplate
- Easy to mismatch argument names
- No compile-time validation
- Difficult to maintain as commands grow

### Evolved API (v2.0)

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "myapp")]
enum Cli {
    /// Manage users
    User(UserCommand),
}

#[derive(Parser)]
enum UserCommand {
    /// Create a new user
    Create(CreateUser),
    /// Delete an existing user
    Delete(DeleteUser),
}

#[derive(Parser)]
struct CreateUser {
    #[arg(short, long)]
    name: String,

    #[arg(short, long)]
    email: String,
}
```

**Improvements:**
- Type-safe argument access
- Compile-time validation
- Self-documenting
- Reduced boilerplate

## Best Practices

### Designing for Evolution

**1. Use `#[non_exhaustive]` liberally**

```rust
#[non_exhaustive]
pub struct Options {
    pub timeout: Duration,
}
```

**2. Prefer builders over constructors with many parameters**

**3. Return `impl Trait` for flexibility**

**4. Accept generic parameters**

### Documentation as API Contract

Document not just what, but why and when:

```rust
/// Creates a new client with the specified configuration.
///
/// # Panics
///
/// Panics if `config.timeout` is zero.
///
/// # Stability
///
/// This constructor has been stable since v1.0. For more configuration
/// options, consider using [`Client::builder()`] instead.
pub fn new(config: Config) -> Self { /* ... */ }
```

### Changelog and Communication

Maintain a CHANGELOG.md following Keep a Changelog format, documenting what changed and why.

## Summary

Effective API evolution balances innovation with stability. The key principles are:

1. **Design for extensibility** - Use `#[non_exhaustive]`, builders, and traits
2. **Evolve incrementally** - Add new APIs before removing old ones
3. **Communicate clearly** - Deprecation warnings, changelogs, migration guides
4. **Respect SemVer** - Users depend on version numbers meaning something
5. **Automate where possible** - Migration tools reduce friction

Remember: your API is a user interface. Treat it with the same care you would give any UX design, and your users will reward you with loyalty and adoption.
