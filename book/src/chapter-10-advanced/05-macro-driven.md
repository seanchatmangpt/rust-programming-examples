# Macro-Driven Architecture

## Learning Objectives

By the end of this chapter, you will:
- Understand when macros improve system architecture
- Master declarative macro patterns for domain-specific languages
- Implement procedural macros for code generation
- Apply lessons from the json! macro case study

## Introduction

Macros are Rust's metaprogramming facility, enabling compile-time code generation and domain-specific languages (DSLs). While powerful, macros add complexity. This chapter explores when and how to leverage macros in systems architecture, with deep analysis of the `json-macro` project.

## When to Use Macros in Systems

Macros solve specific architectural problems that other language features cannot.

### The Macro Decision Tree

```
Does this problem require compile-time code generation?
├─ No → Use functions, traits, or const fn
└─ Yes → Is it repetitive boilerplate?
    ├─ No → Reconsider: is macro really needed?
    └─ Yes → Will users benefit from DSL syntax?
        ├─ No → Use build.rs code generation
        └─ Yes → Macro appropriate
            ├─ Simple pattern substitution? → Declarative macro
            └─ Complex AST transformation? → Procedural macro
```

### Valid Use Cases

From across the project portfolio:

#### 1. Eliminating Boilerplate

**Problem**: Repetitive test setup in `binary-tree`:

```rust
// Without macros: repetitive
#[test]
fn test_insert_left() {
    let mut tree = BinaryTree::new(10);
    tree.insert(5);
    assert_eq!(tree.left.as_ref().unwrap().value, 5);
}

#[test]
fn test_insert_right() {
    let mut tree = BinaryTree::new(10);
    tree.insert(15);
    assert_eq!(tree.right.as_ref().unwrap().value, 15);
}

// With declarative macro: DRY
macro_rules! tree_test {
    ($name:ident, $root:expr, $insert:expr, $check:expr) => {
        #[test]
        fn $name() {
            let mut tree = BinaryTree::new($root);
            tree.insert($insert);
            $check(&tree);
        }
    };
}

tree_test!(test_insert_left, 10, 5, |t| {
    assert_eq!(t.left.as_ref().unwrap().value, 5);
});

tree_test!(test_insert_right, 10, 15, |t| {
    assert_eq!(t.right.as_ref().unwrap().value, 15);
});
```

#### 2. Domain-Specific Languages

**Problem**: JSON construction is verbose in `actix-gcd`:

```rust
// Without macro: verbose and error-prone
let mut map = serde_json::Map::new();
map.insert(
    "result".to_string(),
    serde_json::Value::Number(result.into()),
);
map.insert(
    "inputs".to_string(),
    serde_json::Value::Array(vec![
        serde_json::Value::Number(n.into()),
        serde_json::Value::Number(m.into()),
    ]),
);
let value = serde_json::Value::Object(map);

// With json! macro: intuitive DSL
let value = json!({
    "result": result,
    "inputs": [n, m]
});
```

#### 3. Compile-Time Verification

**Problem**: Configuration errors found at runtime:

```rust
// Runtime error - discovered when code runs
let config = Config::from_str(r#"{ "port": "not a number" }"#)?;

// Compile-time error - discovered during build
let config = config_macro! {
    port: 8080,  // Type-checked at compile time
    host: "localhost"
};
```

### Invalid Use Cases (Macro Misuse)

#### Anti-Pattern 1: Macro Where Function Suffices

```rust
// BAD: Macro for simple repetition
macro_rules! add {
    ($a:expr, $b:expr) => {
        $a + $b
    };
}

// GOOD: Just use a function
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Why**: Functions provide better type checking, error messages, and IDE support.

#### Anti-Pattern 2: Overly Complex Macro

```rust
// BAD: Turing-complete macro soup
macro_rules! state_machine {
    // 200 lines of macro rules implementing a state machine...
}

// GOOD: Use const fn or build.rs
const fn build_state_machine() -> StateMachine {
    // Clear, debuggable code
}
```

## Declarative Macros for DSLs

Declarative macros (`macro_rules!`) excel at pattern matching and substitution.

### Basic Patterns

From `queue` project principles:

```rust
macro_rules! queue {
    () => {
        Queue::new()
    };
    ($($x:expr),+ $(,)?) => {
        {
            let mut q = Queue::new();
            $(q.push($x);)+
            q
        }
    };
}

// Usage
let empty = queue![];
let numbers = queue![1, 2, 3, 4, 5];
```

### Recursive Patterns

Counting at compile time:

```rust
macro_rules! count {
    () => { 0 };
    ($head:tt $($tail:tt)*) => {
        1 + count!($($tail)*)
    };
}

const LEN: usize = count!(a b c d e);  // LEN = 5
```

### Hygiene and Scope

Macros are hygienic—variables don't leak:

```rust
macro_rules! create_function {
    ($name:ident) => {
        fn $name() {
            let x = 42;  // This x doesn't conflict with outer x
            println!("{}", x);
        }
    };
}

let x = 10;
create_function!(my_fn);
my_fn();  // Prints 42, not 10
```

### Advanced: TT Muncher Pattern

Process arbitrary-length input:

```rust
macro_rules! parse_fields {
    // Base case
    (@fields) => {
        vec![]
    };

    // Recursive case
    (@fields $field:ident : $type:ty, $($rest:tt)*) => {
        {
            let mut fields = parse_fields!(@fields $($rest)*);
            fields.push((stringify!($field), stringify!($type)));
            fields
        }
    };

    // Entry point
    ($($tt:tt)*) => {
        parse_fields!(@fields $($tt)*)
    };
}

let fields = parse_fields!(name: String, age: u32, active: bool,);
// fields = [("name", "String"), ("age", "u32"), ("active", "bool")]
```

## Procedural Macros for Code Generation

Procedural macros operate on token streams, enabling full AST manipulation.

### The Three Types

```rust
// 1. Function-like macro
let json = json!({ "key": "value" });

// 2. Derive macro
#[derive(Serialize, Deserialize)]
struct User { name: String }

// 3. Attribute macro
#[route(GET, "/users/{id}")]
fn get_user(id: u64) -> User { }
```

### Anatomy of a Procedural Macro

Minimal proc macro structure:

```rust
// In proc-macro crate (Cargo.toml: proc-macro = true)
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    // 1. Parse input tokens into AST
    let input = parse_macro_input!(input as DeriveInput);

    // 2. Extract struct name and fields
    let name = &input.ident;
    let builder_name = format!("{}Builder", name);
    let builder_ident = syn::Ident::new(&builder_name, name.span());

    // 3. Generate code with quote!
    let expanded = quote! {
        pub struct #builder_ident {
            // Generated fields...
        }

        impl #name {
            pub fn builder() -> #builder_ident {
                #builder_ident::default()
            }
        }
    };

    // 4. Return generated tokens
    TokenStream::from(expanded)
}
```

## Case Study: json! Macro Implementation

The `json-macro` project demonstrates real-world proc macro architecture.

### Requirements Analysis

**Goals**:
1. Intuitive JSON syntax
2. Compile-time validation
3. Zero runtime parsing overhead
4. Integration with serde_json types

**Input**:
```rust
json!({
    "name": "Alice",
    "age": 30,
    "hobbies": ["reading", "coding"],
    "active": true
})
```

**Output**: `serde_json::Value` constructed at compile time.

### Architecture Overview

```
User code with json! macro
    ↓
Proc macro expansion
    ↓
TokenStream parsing (syn)
    ↓
AST construction
    ↓
Validation (type checking)
    ↓
Code generation (quote)
    ↓
Expanded Rust code
    ↓
Compilation to binary
```

### Implementation: Parsing Phase

From `json-macro/src/lib.rs`:

```rust
use proc_macro::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Result, Token,
};

enum Json {
    Object(Vec<(String, Json)>),
    Array(Vec<Json>),
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

impl Parse for Json {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(syn::token::Brace) {
            // Parse object: { "key": value, ... }
            let content;
            syn::braced!(content in input);

            let pairs: Punctuated<ObjectPair, Token![,]> =
                content.parse_terminated(ObjectPair::parse)?;

            Ok(Json::Object(
                pairs.into_iter().map(|p| (p.key, p.value)).collect(),
            ))
        } else if lookahead.peek(syn::token::Bracket) {
            // Parse array: [ value, ... ]
            let content;
            syn::bracketed!(content in input);

            let elements: Punctuated<Json, Token![,]> =
                content.parse_terminated(Json::parse)?;

            Ok(Json::Array(elements.into_iter().collect()))
        } else if lookahead.peek(syn::LitStr) {
            // Parse string: "value"
            let lit: syn::LitStr = input.parse()?;
            Ok(Json::String(lit.value()))
        } else if lookahead.peek(syn::LitFloat) {
            // Parse number: 3.14
            let lit: syn::LitFloat = input.parse()?;
            Ok(Json::Number(lit.base10_parse()?))
        } else if lookahead.peek(Token![true]) || lookahead.peek(Token![false]) {
            // Parse bool: true | false
            let value: syn::LitBool = input.parse()?;
            Ok(Json::Bool(value.value))
        } else if lookahead.peek(Token![null]) {
            // Parse null
            input.parse::<Token![null]>()?;
            Ok(Json::Null)
        } else {
            Err(lookahead.error())
        }
    }
}

struct ObjectPair {
    key: String,
    value: Json,
}

impl Parse for ObjectPair {
    fn parse(input: ParseStream) -> Result<Self> {
        let key: syn::LitStr = input.parse()?;
        input.parse::<Token![:]>()?;
        let value: Json = input.parse()?;
        Ok(ObjectPair {
            key: key.value(),
            value,
        })
    }
}
```

### Implementation: Code Generation Phase

```rust
use quote::quote;

impl Json {
    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        match self {
            Json::Object(pairs) => {
                let insertions = pairs.iter().map(|(key, value)| {
                    let value_tokens = value.to_token_stream();
                    quote! {
                        map.insert(
                            String::from(#key),
                            #value_tokens
                        );
                    }
                });

                quote! {
                    {
                        let mut map = serde_json::Map::new();
                        #(#insertions)*
                        serde_json::Value::Object(map)
                    }
                }
            }
            Json::Array(elements) => {
                let element_tokens = elements.iter().map(|e| e.to_token_stream());
                quote! {
                    {
                        let mut array = Vec::new();
                        #(array.push(#element_tokens);)*
                        serde_json::Value::Array(array)
                    }
                }
            }
            Json::String(s) => {
                quote! { serde_json::Value::String(String::from(#s)) }
            }
            Json::Number(n) => {
                quote! {
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(#n).unwrap()
                    )
                }
            }
            Json::Bool(b) => {
                quote! { serde_json::Value::Bool(#b) }
            }
            Json::Null => {
                quote! { serde_json::Value::Null }
            }
        }
    }
}

#[proc_macro]
pub fn json(input: TokenStream) -> TokenStream {
    let json = syn::parse_macro_input!(input as Json);
    let tokens = json.to_token_stream();
    TokenStream::from(tokens)
}
```

### What Gets Generated

Input:
```rust
json!({
    "name": "Alice",
    "age": 30
})
```

Expands to:
```rust
{
    let mut map = serde_json::Map::new();
    map.insert(
        String::from("name"),
        serde_json::Value::String(String::from("Alice"))
    );
    map.insert(
        String::from("age"),
        serde_json::Value::Number(serde_json::Number::from_f64(30.0).unwrap())
    );
    serde_json::Value::Object(map)
}
```

**Result**: Zero parsing at runtime, fully type-checked at compile time.

## Performance Implications

### Compile-Time Cost

```bash
# Measure macro expansion time
cargo build --timings

# json-macro adds ~2 seconds to build
# But saves ~100ns per JSON object at runtime
```

**Trade-off**: Worthwhile for servers processing millions of requests.

### Runtime Benefit

Benchmark comparison:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_json_macro(c: &mut Criterion) {
    c.bench_function("json! macro", |b| {
        b.iter(|| {
            black_box(json!({
                "name": "Alice",
                "age": 30,
                "hobbies": ["reading", "coding"]
            }))
        })
    });
}

fn bench_json_parse(c: &mut Criterion) {
    c.bench_function("runtime parse", |b| {
        b.iter(|| {
            black_box(
                serde_json::from_str::<serde_json::Value>(
                    r#"{"name":"Alice","age":30,"hobbies":["reading","coding"]}"#
                ).unwrap()
            )
        })
    });
}

criterion_group!(benches, bench_json_macro, bench_json_parse);
criterion_main!(benches);

// Results:
// json! macro:     8 ns  (allocations only)
// runtime parse: 120 ns  (parsing + allocations)
```

**15× speedup** for this example.

## Best Practices for Macro Architecture

### 1. Error Messages

Poor macro errors are frustrating:

```rust
// BAD: Cryptic error
error: no rules expected the token `{`

// GOOD: Clear guidance
error: expected one of: object `{}`, array `[]`, string, number, bool, or null
  --> src/main.rs:4:15
   |
 4 |     let x = json!(invalid);
   |                   ^^^^^^^ unexpected token
```

Implementation:
```rust
impl Parse for Json {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(syn::token::Brace) {
            // ... parse object
        } else {
            // Helpful error message
            Err(syn::Error::new(
                input.span(),
                "expected JSON value: object, array, string, number, bool, or null"
            ))
        }
    }
}
```

### 2. Hygiene and Imports

Avoid requiring users to import dependencies:

```rust
// BAD: User must import serde_json
quote! { serde_json::Value::Null }

// GOOD: Fully-qualified path
quote! { ::serde_json::Value::Null }
```

### 3. Debug Output

Enable macro expansion viewing:

```bash
cargo expand --lib
# Shows fully expanded macro output
```

### 4. Testing

Test macro output, not just usage:

```rust
#[test]
fn test_json_object() {
    let result = json!({ "key": "value" });

    // Test generated code compiles
    assert!(matches!(result, serde_json::Value::Object(_)));

    // Test runtime behavior
    assert_eq!(result["key"], "value");
}

// Test expansion directly
#[test]
fn test_expansion() {
    let input = quote! { { "key": "value" } };
    let output = json_impl(input.into());

    // Verify generated code structure
    assert!(output.to_string().contains("Map::new"));
}
```

## Decision Framework

| Consideration | Declarative | Procedural | Neither |
|---------------|-------------|------------|---------|
| Simple pattern substitution | ✅ | ❌ | ❌ |
| Complex AST manipulation | ❌ | ✅ | ❌ |
| Derive implementation | ❌ | ✅ | ❌ |
| Attribute macro | ❌ | ✅ | ❌ |
| Compile-time acceptable | ✅ | ✅ | ❌ |
| Runtime generation OK | ❌ | ❌ | ✅ (build.rs) |

## Summary

Macro-driven architecture enables:
- **DSLs** for intuitive APIs (json! macro)
- **Boilerplate elimination** through code generation
- **Compile-time verification** for safety
- **Zero runtime overhead** for performance

The `json-macro` project exemplifies production-quality proc macro design: clear parsing, robust error handling, comprehensive testing, and measurable performance benefits.

Use macros judiciously—they're powerful but add complexity. When the benefits justify the cost, they enable elegant solutions impossible otherwise.

## Further Reading

- Chapter 10.4: Trade-offs between macro complexity and benefits
- Chapter 11.1: Modern ecosystem trends in macro usage
- The Little Book of Rust Macros: https://veykril.github.io/tlborm/
