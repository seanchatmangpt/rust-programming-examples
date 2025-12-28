# How to Write Declarative Macros

## Overview

This guide teaches you how to write `macro_rules!` macros in Rust. We'll build a JSON literal macro step by step, showing you how to create compile-time code generation that feels like built-in language syntax.

## Prerequisites

- Understanding of Rust syntax and types
- Familiarity with pattern matching
- Basic knowledge of Rust's token trees

## Why Macros?

Macros let you write code that writes code. They're similar to Python decorators or metaprogramming, but:
- Execute at **compile time**, not runtime
- Generate code based on **patterns**, not strings
- Type-checked by the compiler
- Zero runtime overhead

Use macros when you need:
- Custom syntax (like `println!` or `vec!`)
- Compile-time code generation
- To eliminate repetitive boilerplate
- Type-generic operations that traits can't express

## The Goal: A JSON Macro

We want to write JSON literals in Rust code like this:

```rust
let config = json!({
    "name": "MyApp",
    "version": 1.0,
    "features": ["auth", "logging"],
    "settings": {
        "debug": true
    }
});
```

This should expand to proper Rust code that creates a `Json` enum.

## Step 1: Define Your Data Structure

First, create the types your macro will generate:

```rust
use std::collections::HashMap;

#[derive(Clone, PartialEq, Debug)]
pub enum Json {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Box<HashMap<String, Json>>)
}
```

## Step 2: Create Basic Macro Rules

Start with simple patterns using `macro_rules!`:

```rust
#[macro_export]
macro_rules! json {
    (null) => {
        $crate::Json::Null
    };
}
```

Breaking this down:
- `#[macro_export]`: Makes the macro available to other crates
- `macro_rules! json`: Defines a macro named `json`
- `(null)`: Pattern to match
- `=>`: Separates pattern from expansion
- `$crate::Json::Null`: Code to generate (use `$crate` for the current crate)

**Usage:**
```rust
let value = json!(null);  // Expands to: Json::Null
```

## Step 3: Handle Simple Values

Add patterns for basic JSON types:

```rust
#[macro_export]
macro_rules! json {
    (null) => {
        $crate::Json::Null
    };

    ($other:tt) => {
        $crate::Json::from($other)
    };
}
```

The `$other:tt` pattern:
- `$other` is a **metavariable** (captures input)
- `tt` is a **fragment specifier** (matches any single token tree)

Token tree types:
- `tt` - Any single token or group in `(...)`/`[...]`/`{...}`
- `expr` - An expression like `1 + 2` or `foo()`
- `ident` - An identifier like `foo` or `bar`
- `ty` - A type like `String` or `Vec<u8>`
- `literal` - A literal like `42` or `"hello"`

For this to work, implement `From` conversions:

```rust
impl From<bool> for Json {
    fn from(b: bool) -> Json {
        Json::Boolean(b)
    }
}

impl From<String> for Json {
    fn from(s: String) -> Json {
        Json::String(s)
    }
}

impl<'a> From<&'a str> for Json {
    fn from(s: &'a str) -> Json {
        Json::String(s.to_string())
    }
}

// For all number types
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

impl_from_num_for_json!(u8 i8 u16 i16 u32 i32 u64 i64 usize isize f32 f64);
```

**Usage:**
```rust
let n = json!(42);        // Json::from(42)
let b = json!(true);      // Json::from(true)
let s = json!("hello");   // Json::from("hello")
```

## Step 4: Handle Arrays with Repetition

Use repetition patterns to handle arrays:

```rust
#[macro_export]
macro_rules! json {
    (null) => {
        $crate::Json::Null
    };

    ([ $( $element:tt ),* ]) => {
        $crate::Json::Array(vec![ $( json!($element) ),* ])
    };

    ($other:tt) => {
        $crate::Json::from($other)
    };
}
```

Repetition syntax:
- `$( ... )*` - Repeat zero or more times
- `$( ... )+` - Repeat one or more times
- `$( ... ),*` - Repeat with comma separator (trailing comma optional)
- `$( ... ),+` - Repeat with comma separator (at least one)

This pattern matches:
- `[ ]` - Empty array
- `[ 1 ]` - Single element
- `[ 1, 2, 3 ]` - Multiple elements
- `[ "a", true, null ]` - Mixed types

The expansion:
1. Capture each element as `$element`
2. For each captured element, recursively call `json!($element)`
3. Collect results into a `Vec`
4. Wrap in `Json::Array`

**Usage:**
```rust
let arr = json!([1, 2, 3]);
// Expands to:
// Json::Array(vec![
//     json!(1),
//     json!(2),
//     json!(3)
// ])
// Which becomes:
// Json::Array(vec![
//     Json::from(1),
//     Json::from(2),
//     Json::from(3)
// ])
```

## Step 5: Handle Objects

Add a pattern for JSON objects:

```rust
#[macro_export]
macro_rules! json {
    (null) => {
        $crate::Json::Null
    };

    ([ $( $element:tt ),* ]) => {
        $crate::Json::Array(vec![ $( json!($element) ),* ])
    };

    ({ $( $key:tt : $value:tt ),* }) => {
        {
            let mut fields = $crate::macros::Box::new(
                $crate::macros::HashMap::new()
            );
            $(
                fields.insert(
                    $crate::macros::ToString::to_string($key),
                    json!($value)
                );
            )*
            $crate::Json::Object(fields)
        }
    };

    ($other:tt) => {
        $crate::Json::from($other)
    };
}
```

The object pattern:
- Matches `{ "key": value, "key2": value2 }`
- Uses two `tt` metavariables: `$key` and `$value`
- Repeats with `$( ... ),*` for multiple entries

You need to export helpers for macro hygiene:

```rust
// In macros.rs or lib.rs
pub use std::collections::HashMap;
pub use std::boxed::Box;
pub use std::string::ToString;
```

**Usage:**
```rust
let obj = json!({
    "name": "Rust",
    "version": 1.0,
    "stable": true
});

// Expands to:
// {
//     let mut fields = Box::new(HashMap::new());
//     fields.insert("name".to_string(), json!("Rust"));
//     fields.insert("version".to_string(), json!(1.0));
//     fields.insert("stable".to_string(), json!(true));
//     Json::Object(fields)
// }
```

## Step 6: Pattern Ordering Matters

Macro patterns are tried in order, top to bottom. More specific patterns must come before general ones:

```rust
#[macro_export]
macro_rules! json {
    // Specific patterns first
    (null) => { ... };
    ([ $( $element:tt ),* ]) => { ... };
    ({ $( $key:tt : $value:tt ),* }) => { ... };

    // General pattern last (catches everything else)
    ($other:tt) => { ... };
}
```

If you put `($other:tt)` first, it would match everything and the other patterns would never run.

## Complete Macro Example

Here's the full macro:

```rust
// lib.rs or macros.rs
pub use std::collections::HashMap;
pub use std::boxed::Box;
pub use std::string::ToString;

#[macro_export]
macro_rules! json {
    (null) => {
        $crate::Json::Null
    };

    ([ $( $element:tt ),* ]) => {
        $crate::Json::Array(vec![ $( json!($element) ),* ])
    };

    ({ $( $key:tt : $value:tt ),* }) => {
        {
            let mut fields = $crate::macros::Box::new(
                $crate::macros::HashMap::new());
            $(
                fields.insert($crate::macros::ToString::to_string($key),
                              json!($value));
            )*
            $crate::Json::Object(fields)
        }
    };

    ($other:tt) => {
        $crate::Json::from($other)
    };
}
```

**Full usage example:**
```rust
let students = json!([
    {
        "name": "Jim Blandy",
        "class_of": 1926,
        "major": "Tibetan throat singing"
    },
    {
        "name": "Jason Orendorff",
        "class_of": 1702,
        "major": "Knots"
    }
]);
```

## Macro Recursion

Notice how `json!` calls itself recursively:

```rust
([ $( $element:tt ),* ]) => {
    $crate::Json::Array(vec![ $( json!($element) ),* ])
    //                        ^^^^^ Recursive call
};
```

This allows nested structures:

```rust
json!([1, [2, 3], [4, [5, 6]]])
// First expansion:
// Json::Array(vec![json!(1), json!([2, 3]), json!([4, [5, 6]])])
// Second expansion:
// Json::Array(vec![
//     Json::from(1),
//     Json::Array(vec![json!(2), json!(3)]),
//     Json::Array(vec![json!(4), json!([5, 6])])
// ])
// And so on...
```

## Debugging Macros

Macros can be hard to debug. Here are helpful techniques:

### 1. Use `cargo expand`

Install and run `cargo-expand` to see macro expansions:

```bash
cargo install cargo-expand
cargo expand
```

### 2. Use `trace_macros!`

Enable macro tracing in your code:

```rust
#![feature(trace_macros)]

fn main() {
    trace_macros!(true);
    let x = json!([1, 2, 3]);
    trace_macros!(false);
}
```

### 3. Use `log_syntax!`

Print tokens during macro expansion:

```rust
macro_rules! debug {
    ($($tt:tt)*) => {
        log_syntax!($($tt)*)
    };
}
```

### 4. Simplify and Test Incrementally

Build macros step by step:

```rust
// Start simple
macro_rules! json { (null) => { Json::Null }; }

// Add tests
#[test]
fn test_null() {
    assert_eq!(json!(null), Json::Null);
}

// Then add more patterns
macro_rules! json {
    (null) => { Json::Null };
    ($other:tt) => { Json::from($other) };
}

#[test]
fn test_numbers() {
    assert_eq!(json!(42), Json::Number(42.0));
}
```

## Macro Hygiene

Rust macros are **hygienic**: variables in macros don't accidentally capture user variables:

```rust
let fields = "Fields, W.C.";
let role = json!({
    "name": "Larson E. Whipsnade",
    "actor": fields  // Uses outer `fields` variable
});

// Inside the macro, we also have:
// let mut fields = Box::new(HashMap::new());
// But Rust renames one of them automatically!
```

Rust handles this automatically by renaming variables to avoid conflicts.

To **intentionally** capture or use items from user's scope, use `$crate::` for crate items and avoid using local variables.

## Advanced Patterns

### Multiple Patterns for Same Input

```rust
macro_rules! calculate {
    ($left:expr + $right:expr) => {
        $left + $right
    };
    ($left:expr - $right:expr) => {
        $left - $right
    };
}

let x = calculate!(5 + 3);  // 8
let y = calculate!(5 - 3);  // 2
```

### Optional Elements

```rust
macro_rules! create_point {
    ($x:expr, $y:expr) => {
        Point { x: $x, y: $y, z: 0 }
    };
    ($x:expr, $y:expr, $z:expr) => {
        Point { x: $x, y: $y, z: $z }
    };
}

let p1 = create_point!(1, 2);       // z defaults to 0
let p2 = create_point!(1, 2, 3);    // All three provided
```

### Nested Repetitions

```rust
macro_rules! matrix {
    ( $( [ $( $val:expr ),* ] ),* ) => {
        vec![ $( vec![ $( $val ),* ] ),* ]
    };
}

let m = matrix![
    [1, 2, 3],
    [4, 5, 6]
];
// Creates: vec![vec![1, 2, 3], vec![4, 5, 6]]
```

### Matching Different Fragment Types

```rust
macro_rules! typed_value {
    ($val:literal) => {
        concat!("literal: ", stringify!($val))
    };
    ($val:expr) => {
        concat!("expression: ", stringify!($val))
    };
    ($val:ident) => {
        concat!("identifier: ", stringify!($val))
    };
}

typed_value!(42);        // "literal: 42"
typed_value!(1 + 2);     // "expression: 1 + 2"
typed_value!(foo);       // "identifier: foo"
```

## Comparison to Python Metaprogramming

| Feature | Python | Rust macros |
|---------|--------|-------------|
| Execution time | Runtime | Compile-time |
| Input | Strings or AST | Token trees |
| Type safety | Dynamic | Static (after expansion) |
| Performance | Runtime overhead | Zero overhead |
| Debugging | Standard debugger | cargo expand, trace_macros |
| Hygiene | Manual | Automatic |

**Python decorator (runtime):**
```python
@json_serializable
class Config:
    name: str = "MyApp"
    version: float = 1.0

# Decorator modifies class at runtime
```

**Rust macro (compile-time):**
```rust
let config = json!({
    "name": "MyApp",
    "version": 1.0
});

// Expanded to code before compilation
```

**Python metaprogramming:**
```python
# Manipulating AST at import time
import ast

class JSONTransformer(ast.NodeTransformer):
    def visit_Dict(self, node):
        # Transform dictionary literals
        return ast.Call(func=ast.Name(id='Json.Object'), ...)

# Must be in separate file, runs at import
```

**Rust equivalent:**
```rust
// Macro generates code at compile time
// No separate compilation step needed
// Fully type-checked after expansion
```

## Common Macro Patterns

### 1. Counting Arguments

```rust
macro_rules! count {
    () => { 0 };
    ($head:tt $($tail:tt)*) => { 1 + count!($($tail)*) };
}

let n = count!(a b c);  // 3
```

### 2. Creating Multiple Items

```rust
macro_rules! create_functions {
    ($($name:ident),*) => {
        $(
            fn $name() {
                println!("Called {}", stringify!($name));
            }
        )*
    };
}

create_functions!(foo, bar, baz);
// Expands to three functions: foo(), bar(), baz()
```

### 3. DSL (Domain Specific Language)

```rust
macro_rules! sql {
    (SELECT $($field:ident),* FROM $table:ident WHERE $cond:expr) => {
        format!("SELECT {} FROM {} WHERE {}",
                stringify!($($field),*),
                stringify!($table),
                stringify!($cond))
    };
}

let query = sql!(SELECT name, age FROM users WHERE age > 18);
```

## When NOT to Use Macros

Prefer these alternatives when possible:

1. **Functions** - For simple code reuse
2. **Generics** - For type-generic operations
3. **Traits** - For polymorphism
4. **Const functions** - For compile-time computation

Use macros only when you need:
- Custom syntax
- Compile-time code generation based on patterns
- Operations that can't be expressed with types

## Testing Macros

Always test your macros thoroughly:

```rust
#[cfg(test)]
mod tests {
    use crate::Json;

    #[test]
    fn json_null() {
        assert_eq!(json!(null), Json::Null);
    }

    #[test]
    fn json_array() {
        let value = json!([1, 2, 3]);
        match value {
            Json::Array(arr) => assert_eq!(arr.len(), 3),
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn json_nested() {
        let value = json!({
            "user": {
                "name": "Alice",
                "age": 30
            }
        });
        // Verify structure
        assert!(matches!(value, Json::Object(_)));
    }

    #[test]
    fn json_with_expressions() {
        const HELLO: &str = "hello";
        let value = json!({
            "math": (4 - 2 == 2),
            "greeting": HELLO
        });
        // Macros can use Rust expressions
    }
}
```

## Next Steps

- Learn about [procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html) for more power
- Study [real-world macro examples](https://github.com/dtolnay/quote) like `serde_json::json!`
- Read [The Little Book of Rust Macros](https://danielkeep.github.io/tlborm/book/)

## Related Examples

- `/home/user/rust-programming-examples/json-macro/src/macros.rs` - Full JSON macro implementation
- `/home/user/rust-programming-examples/json-macro/src/lib.rs` - Supporting types and tests
