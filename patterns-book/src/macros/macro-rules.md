# Pattern: Declarative Macros with macro_rules!

## Context

You are writing Rust code and find yourself repeating similar code patterns across multiple types or contexts. You need a way to generate code at compile time that follows a consistent structure but varies in specific details. The repetition might involve trait implementations, data structure definitions, or complex initialization patterns that differ only in type parameters or literal values.

You have access to Rust's macro system and want to use declarative macros (macro_rules!) rather than procedural macros, which require separate crates and more complex tooling.

## Problem

**How do you eliminate code duplication and generate repetitive code patterns at compile time while maintaining type safety and clarity?**

Writing the same code pattern repeatedly is error-prone and violates the DRY (Don't Repeat Yourself) principle. Manual duplication makes maintenance difficult—when you need to change the pattern, you must update every instance. However, runtime solutions like functions or traits cannot always capture the needed flexibility, especially when you need to:

- Generate code for multiple different types
- Create different token sequences based on patterns
- Build domain-specific syntax that looks unlike normal Rust
- Perform computations at compile time
- Generate both code and documentation

## Forces

- **Type Safety**: Generated code must be type-checked by the compiler; macros should produce valid Rust code
- **Compile-Time Execution**: Code generation happens during compilation, not runtime—no runtime overhead
- **Pattern Matching**: The macro system uses pattern matching on token trees, not runtime values
- **Hygiene**: Variables introduced by macros should not interfere with user code
- **Debugging Difficulty**: Macro expansion errors can be cryptic; generated code is harder to debug
- **Cognitive Load**: Complex macros can be difficult to understand and maintain
- **Repetition vs Abstraction**: Balance between eliminating duplication and keeping code readable
- **Fragment Specifiers**: You must choose appropriate fragment types (expr, ty, ident, etc.) for each macro parameter

## Solution

**Use `macro_rules!` to define pattern-based code generation with declarative rules that match token sequences and produce expanded code.**

Define a macro with one or more rules, each consisting of:
1. A pattern that matches input tokens
2. A template that generates output tokens

### Structure

```rust
macro_rules! macro_name {
    // Rule 1: pattern => expansion
    (pattern1) => {
        expansion1
    };
    // Rule 2: another pattern => another expansion
    (pattern2) => {
        expansion2
    };
    // Additional rules as needed...
}
```

### Real Example: Implementing From Traits for Multiple Numeric Types

From `/home/user/rust-programming-examples/json-macro/src/lib.rs`:

```rust
macro_rules! impl_from_num_for_json {
    ( $( $t:ident )* ) => {
        $(
            impl From<$t> for Json {
                fn from(n: $t) -> Json {
                    Json::Number(n as f64)
                }
            }
        )*
    };
}

impl_from_num_for_json!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128
                        usize isize f32 f64);
```

This macro:
- **Pattern**: `$( $t:ident )*` matches zero or more identifiers (type names)
- **Expansion**: For each captured identifier `$t`, generates a complete `From` trait implementation
- **Result**: 14 trait implementations from a single macro invocation

Without the macro, you would write:

```rust
// Without macro: 14 repetitive implementations
impl From<u8> for Json {
    fn from(n: u8) -> Json {
        Json::Number(n as f64)
    }
}

impl From<i8> for Json {
    fn from(n: i8) -> Json {
        Json::Number(n as f64)
    }
}

// ... 12 more identical implementations
```

### Components Explained

**Fragment Specifiers**: Constrain what tokens can match

```rust
macro_rules! type_safe_patterns {
    // ident: an identifier (variable/type name)
    ($name:ident) => { ... };

    // expr: an expression (2 + 2, foo(), etc.)
    ($value:expr) => { ... };

    // ty: a type (i32, Vec<String>, etc.)
    ($type_name:ty) => { ... };

    // tt: a single token tree (most flexible)
    ($anything:tt) => { ... };
}
```

**Repetition Patterns**: Handle multiple items

```rust
macro_rules! repetition_examples {
    // Zero or more: $( ... )*
    ($( $item:expr ),*) => { ... };

    // One or more: $( ... )+
    ($( $item:expr ),+) => { ... };

    // Optional: $( ... )?
    ($( $optional:expr )?) => { ... };
}
```

### Making Macros Visible

**Internal Use (Same Crate)**:
```rust
// lib.rs or module file
macro_rules! internal_macro {
    () => { /* ... */ };
}

// Use macro attribute for module visibility
#[macro_use]
mod macros;
```

**External Use (Other Crates)**:
```rust
// Export macro for other crates
#[macro_export]
macro_rules! public_macro {
    () => { /* ... */ };
}
```

From the json-macro project structure:

```rust
// lib.rs
#[macro_use] mod macros;  // Import macros from macros.rs

// macros.rs
#[macro_export]
macro_rules! json {
    // Rules...
}
```

## Resulting Context

### Benefits

**Eliminated Duplication**: The 14 numeric type implementations are defined once, generated 14 times. A change to the pattern updates all implementations automatically.

**Type Safety**: The compiler verifies all generated code. If the macro produces invalid Rust, compilation fails with a clear error location.

**Zero Runtime Cost**: All expansion happens at compile time. The generated code is identical to hand-written code—no function calls, no indirection.

**Maintainability**: Adding support for new numeric types requires only adding the type name to the invocation list.

### New Challenges

**Debugging Complexity**: When macro-generated code has errors, the error messages reference the expanded code, not the macro definition. Use `cargo expand` to see what the macro generates:

```bash
cargo install cargo-expand
cargo expand
```

**Cognitive Overhead**: Developers must understand both the macro syntax and what it generates. Complex macros require careful documentation.

**Compile Time**: Large macros or frequent macro invocations can increase compilation time, though usually negligibly.

### When You Have This Pattern

You can now:
- Define additional macros for other repetitive patterns
- Combine macros to build more complex code generation
- Use **RECURSIVE MACRO** to handle nested structures
- Apply **FRAGMENT SPECIFIERS** to constrain inputs
- Build **DSL CONSTRUCTION** for domain-specific syntax

## Related Patterns

- **RECURSIVE MACRO**: Extend macro_rules! to handle nested or recursive structures
- **FRAGMENT SPECIFIERS**: Use appropriate fragment types for type safety and clarity
- **MACRO HYGIENE**: Understand how Rust prevents variable name conflicts in macros
- **DSL CONSTRUCTION**: Build domain-specific languages using macro syntax

## Known Uses

### Standard Library

**vec!** - Creates vectors with inline syntax:
```rust
let v = vec![1, 2, 3, 4, 5];
// Expands to:
// {
//     let mut temp_vec = Vec::new();
//     temp_vec.push(1);
//     temp_vec.push(2);
//     ...
//     temp_vec
// }
```

**println!** - Formatted output:
```rust
println!("Hello, {}!", name);
// Expands to complex format_args! and write! invocations
```

**assert_eq!** - Test assertions:
```rust
assert_eq!(actual, expected);
// Expands to panic with detailed message on mismatch
```

### Ecosystem Examples

**serde_json::json!** - Similar to our example, creates JSON values:
```rust
let value = serde_json::json!({
    "name": "John",
    "age": 30
});
```

**lazy_static!** - Initialize static variables with runtime code:
```rust
lazy_static! {
    static ref HASHMAP: HashMap<u32, &'static str> = {
        let mut m = HashMap::new();
        m.insert(0, "zero");
        m
    };
}
```

**diesel::table!** - Define database table schemas:
```rust
table! {
    users (id) {
        id -> Integer,
        name -> Text,
    }
}
```

### json-macro Project

The complete macro_rules! infrastructure supporting the JSON DSL (see **DSL CONSTRUCTION** and **RECURSIVE MACRO** patterns for the full implementation).

## Examples

### Example 1: Simple Constant Wrapper

```rust
macro_rules! create_const {
    ($name:ident, $value:expr) => {
        const $name: i32 = $value;
    };
}

create_const!(MAX_SIZE, 100);
create_const!(MIN_SIZE, 10);

// Expands to:
// const MAX_SIZE: i32 = 100;
// const MIN_SIZE: i32 = 10;
```

### Example 2: Implementing a Trait for Multiple Types

```rust
trait Describable {
    fn describe(&self) -> &'static str;
}

macro_rules! impl_describable {
    ($($t:ty => $desc:expr),*) => {
        $(
            impl Describable for $t {
                fn describe(&self) -> &'static str {
                    $desc
                }
            }
        )*
    };
}

impl_describable!(
    i32 => "a 32-bit integer",
    String => "a heap-allocated string",
    bool => "a boolean value"
);
```

### Example 3: Creating Enum Variants with Associated Data

```rust
macro_rules! create_error_enum {
    ($enum_name:ident { $($variant:ident($type:ty)),* }) => {
        enum $enum_name {
            $(
                $variant($type),
            )*
        }
    };
}

create_error_enum!(MyError {
    Io(std::io::Error),
    Parse(std::num::ParseIntError),
    Custom(String)
});

// Expands to:
// enum MyError {
//     Io(std::io::Error),
//     Parse(std::num::ParseIntError),
//     Custom(String),
// }
```

## Anti-Patterns to Avoid

### Over-Engineering Simple Cases

```rust
// ❌ BAD: Macro for something simple
macro_rules! add_one {
    ($x:expr) => { $x + 1 };
}
let result = add_one!(5);

// ✅ GOOD: Just use a function
fn add_one(x: i32) -> i32 { x + 1 }
let result = add_one(5);
```

**Why?** Functions are clearer, easier to debug, and provide better error messages. Use macros only when functions cannot achieve the same result.

### Unclear Fragment Specifiers

```rust
// ❌ BAD: Using tt for everything
macro_rules! unclear {
    ($x:tt, $y:tt) => {
        $x + $y  // What if user passes non-numeric tokens?
    };
}

// ✅ GOOD: Use specific fragment specifier
macro_rules! clear {
    ($x:expr, $y:expr) => {
        $x + $y  // Compiler enforces these are expressions
    };
}
```

### Ignoring Hygiene

```rust
// ❌ BAD: Macro that creates name conflicts
macro_rules! bad_temp {
    ($value:expr) => {{
        let temp = $value;  // 'temp' might conflict with user code
        temp * 2
    }};
}

// ✅ GOOD: Rely on hygiene or use unique names
macro_rules! good_temp {
    ($value:expr) => {{
        let __macro_temp = $value;  // Less likely to conflict
        __macro_temp * 2
    }};
}
```

## Implementation Checklist

When creating a macro_rules! macro:

- [ ] Identify the repetitive code pattern you want to eliminate
- [ ] Choose appropriate fragment specifiers (expr, ty, ident, tt, etc.)
- [ ] Write the macro with clear patterns and expansions
- [ ] Add #[macro_export] if the macro should be public
- [ ] Test the macro with various inputs
- [ ] Use cargo expand to verify the expansion is correct
- [ ] Document what the macro does and provide usage examples
- [ ] Consider edge cases and error conditions
- [ ] Test that hygiene works correctly (variable names don't conflict)

## Further Reading

- **The Rust Book, Chapter 19.6**: "Macros" - https://doc.rust-lang.org/book/ch19-06-macros.html
- **The Little Book of Rust Macros**: https://veykril.github.io/tlborm/
- **Rust Reference: Macros By Example**: https://doc.rust-lang.org/reference/macros-by-example.html
- **cargo-expand**: Tool to see macro expansions - https://github.com/dtolnay/cargo-expand
