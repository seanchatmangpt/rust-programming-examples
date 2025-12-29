# Type-Driven Refactoring & Safety

Rust's type system provides uniquely powerful tools for type-driven design. A well-designed type hierarchy can make illegal states unrepresentable, guide users toward correct usage patterns, and catch configuration errors at compile time.

## Eliminating Impossible States

The principle of making illegal states unrepresentable is foundational to robust API design.

### The Newtype Pattern

Newtypes wrap primitive types to create distinct semantic types:

```rust
/// A validated command name (non-empty, alphanumeric with hyphens)
pub struct CommandName(String);

impl CommandName {
    pub fn new(s: impl Into<String>) -> Result<Self, ValidationError> {
        let s = s.into();
        if s.is_empty() {
            return Err(ValidationError::EmptyCommandName);
        }
        if !s.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(ValidationError::InvalidCommandName(s));
        }
        Ok(CommandName(s))
    }
}
```

### Phantom Types for Compile-Time Validation

Phantom types carry type-level information without runtime representation:

```rust
use std::marker::PhantomData;

pub struct Unvalidated;
pub struct Validated;

pub struct Arg<State> {
    name: String,
    _state: PhantomData<State>,
}

impl Arg<Unvalidated> {
    pub fn new(name: impl Into<String>) -> Self {
        Arg {
            name: name.into(),
            _state: PhantomData,
        }
    }

    pub fn validate(self) -> Result<Arg<Validated>, ValidationError> {
        Ok(Arg {
            name: self.name,
            _state: PhantomData,
        })
    }
}
```

### The Typestate Pattern

Typestate extends phantom types to model state machines:

```rust
pub struct Empty;
pub struct HasName;
pub struct Ready;

pub struct AppBuilder<State> {
    name: Option<String>,
    version: Option<String>,
    _state: PhantomData<State>,
}

impl AppBuilder<Empty> {
    pub fn new() -> Self {
        AppBuilder {
            name: None,
            version: None,
            _state: PhantomData,
        }
    }

    pub fn name(self, name: impl Into<String>) -> AppBuilder<HasName> {
        AppBuilder {
            name: Some(name.into()),
            version: self.version,
            _state: PhantomData,
        }
    }
}

impl AppBuilder<HasName> {
    pub fn build(self) -> App {
        App {
            name: self.name.expect("type system guarantees this"),
            version: self.version,
        }
    }
}
```

This ensures `build()` can only be called after `name()` has been invoked.

## Trait Refactoring

### Associated Types vs Generic Parameters

```rust
// Generic parameter: multiple implementations per type
trait Parser<T> {
    fn parse(&self, input: &str) -> Result<T, ParseError>;
}

// Associated type: one implementation per type
trait ValueParser {
    type Value;
    fn parse(&self, input: &str) -> Result<Self::Value, ParseError>;
}
```

**Guidelines**: Use **associated types** when the relationship is functional (each implementor has exactly one output type). Use **generic parameters** when relational (implementors may handle multiple types).

### Generic Associated Types (GATs)

```rust
trait ArgCollection {
    type Iter<'a>: Iterator<Item = &'a Arg> where Self: 'a;

    fn iter(&self) -> Self::Iter<'_>;
}

impl ArgCollection for Vec<Arg> {
    type Iter<'a> = std::slice::Iter<'a, Arg>;

    fn iter(&self) -> Self::Iter<'_> {
        self.as_slice().iter()
    }
}
```

### Trait Objects and Dynamic Dispatch

```rust
pub trait Validator: Send + Sync {
    fn validate(&self, value: &str) -> Result<(), ValidationError>;
    fn description(&self) -> &str;
}

pub struct Arg {
    name: String,
    validators: Vec<Box<dyn Validator>>,
}

impl Arg {
    pub fn validator(mut self, v: impl Validator + 'static) -> Self {
        self.validators.push(Box::new(v));
        self
    }
}
```

## Compile-Time Validation

### Const Functions and Const Generics

```rust
pub struct ArgBuffer<const N: usize> {
    args: [Option<String>; N],
    count: usize,
}

impl<const N: usize> ArgBuffer<N> {
    pub const fn new() -> Self {
        const NONE: Option<String> = None;
        ArgBuffer {
            args: [NONE; N],
            count: 0,
        }
    }
}
```

### Declarative Macros for Validation

```rust
macro_rules! command {
    (
        name: $name:expr,
        about: $about:expr,
        $(args: [$($arg:expr),* $(,)?],)?
    ) => {{
        let mut cmd = Command::new($name).about($about);
        $($(cmd = cmd.arg($arg);)*)?
        cmd
    }};

    ($($tt:tt)*) => {
        compile_error!("Command definition requires 'name' and 'about' fields")
    };
}
```

### Procedural Macros for Code Generation

Derive macros can generate type-safe code from declarations:

```rust
#[derive(TypedArgs)]
pub struct MyArgs {
    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    config: String,
}
```

## Error Types & Result Handling

### Sum Types for Errors

```rust
#[derive(Debug)]
pub enum ParseError {
    UnknownCommand { name: String, suggestions: Vec<String> },
    MissingRequiredArg { arg: String, command: String },
    InvalidValue { arg: String, value: String, expected: String },
    ConflictingArgs { args: Vec<String> },
}
```

### Error Context and Cause Chains

```rust
#[derive(Debug)]
pub struct ContextualError {
    context: String,
    source: Box<dyn std::error::Error + Send + Sync>,
}

impl ContextualError {
    pub fn new(context: impl Into<String>, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        ContextualError {
            context: context.into(),
            source: Box::new(source),
        }
    }
}
```

## Type-Safe Error Handling

```rust
pub trait ResultExt<T, E> {
    fn context(self, ctx: impl Into<String>) -> Result<T, ContextualError>
    where
        E: std::error::Error + Send + Sync + 'static;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn context(self, ctx: impl Into<String>) -> Result<T, ContextualError>
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        self.map_err(|e| ContextualError::new(ctx, e))
    }
}
```

## Testing Type Safety

### Compile-Fail Tests

The `trybuild` crate enables testing that invalid code fails to compile:

```rust
#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/required-with-default.rs");
    t.pass("tests/ui/valid-configurations.rs");
}
```

### Miri for Undefined Behavior

```bash
cargo +nightly miri test
```

Miri catches use-after-free, uninitialized memory, and data races.

## Performance Impact

### Monomorphization Trade-offs

Generic code is monomorphized, generating specialized versions for each type:

**Benefits**: Zero-cost abstraction, full optimization, inlining
**Costs**: Increased binary size, longer compile times

### Mitigating Code Bloat

1. **Strategic use of trait objects** where performance is non-critical
2. **Outline cold paths** into non-generic functions
3. **Use concrete types** in hot paths

## Best Practices

### When to Add Types for Safety

**Add types when:**
- Invalid states can be constructed
- Runtime validation is required
- API misuse is common
- The domain has clear semantic distinctions

**Avoid over-engineering when:**
- Type complexity exceeds problem complexity
- Types significantly impair ergonomics
- Invariants are trivial or temporary
- Performance overhead is unacceptable

### The Type Safety Spectrum

```
Less Type Safety                    More Type Safety
      │                                   │
  String ─► Newtype ─► Phantom ─► Typestate ─► GADTs
```

Choose your position based on correctness criticality, API surface area, and user sophistication.

## Summary

Type-driven refactoring transforms runtime failures into compile-time errors. By making illegal states unrepresentable, you shift the burden of correctness from vigilant programmers to the infallible compiler.

The investment in type-driven design pays dividends in fewer bugs, clearer APIs, and more confident refactoring.
