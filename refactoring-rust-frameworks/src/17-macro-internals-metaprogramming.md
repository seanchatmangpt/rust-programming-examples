# Macro Internals & Metaprogramming Deep Dive

**Target Audience**: AI agents analyzing, writing, or reviewing Rust macro code
**Prerequisites**: Understanding of Rust syntax, procedural code generation concepts
**Scope**: Declarative macros, procedural macros, token streams, and metaprogramming patterns

---

## 1. Declarative Macros (macro_rules!)

Declarative macros are pattern-matching systems that transform input token trees into output code. They operate at the syntactic level, replacing one set of tokens with another based on pattern rules.

### Token Matching and Pattern Language

Declarative macros use a domain-specific pattern language to match input tokens:

```rust
macro_rules! simple_example {
    // Pattern => expansion
    (add $a:expr, $b:expr) => {
        $a + $b
    };
    (multiply $a:expr, $b:expr) => {
        $a * $b
    };
}

// Usage
let sum = simple_example!(add 5, 3);        // Expands to: 5 + 3
let product = simple_example!(multiply 2, 4); // Expands to: 2 * 4
```

### Fragment Specifiers

Fragment specifiers define what kind of token tree a metavariable can capture:

**Key fragment specifiers:**
- `expr` - expressions (most common)
- `stmt` - statements
- `ident` - identifiers
- `ty` - types
- `pat` - patterns
- `item` - items (functions, structs, impls)
- `block` - block expressions
- `meta` - attribute metadata
- `tt` - single token tree (most flexible)

### Repetition Patterns

Repetition is core to macro power, enabling variable-length inputs:

```rust
macro_rules! vec_of_strings {
    // Match zero or more comma-separated expressions
    ($($element:expr),*) => {
        {
            let mut v = Vec::new();
            $(
                v.push($element.to_string());
            )*
            v
        }
    };
}

let strings = vec_of_strings!("hello", "world", "rust");
```

**Repetition operators:**
- `*` - zero or more repetitions
- `+` - one or more repetitions
- `?` - zero or one repetition

### Recursion in Declarative Macros

Macros can invoke themselves for complex transformations:

```rust
macro_rules! count {
    () => { 0 };
    ($head:expr) => { 1 };
    ($head:expr, $($tail:expr),+) => {
        1 + count!($($tail),+)
    };
}

const COUNT: usize = count!(1, 2, 3, 4, 5); // Evaluates to 5
```

### Macro Hygiene

Rust macros are hygienic - identifiers defined inside macros don't collide with identifiers in the calling scope:

```rust
macro_rules! hygienic_example {
    () => {
        let x = 42; // This 'x' won't collide with outer scope
        x
    };
}

fn test_hygiene() {
    let x = 10;
    let result = hygienic_example!(); // result = 42, not 10
    println!("x: {}, result: {}", x, result); // x: 10, result: 42
}
```

## 2. Procedural Macros Fundamentals

Procedural macros are Rust functions that take token streams as input and produce token streams as output. They provide more flexibility than declarative macros at the cost of complexity.

### Three Types of Procedural Macros

**Function-like macros:**
```rust
#[proc_macro]
pub fn my_macro(input: TokenStream) -> TokenStream {
    // Transform input to output
    input
}
```

**Attribute macros:**
```rust
#[proc_macro_attribute]
pub fn my_attribute(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Transform the item based on attribute
    item
}
```

**Derive macros:**
```rust
#[proc_macro_derive(MyTrait)]
pub fn derive_my_trait(input: TokenStream) -> TokenStream {
    // Generate impl for MyTrait
    TokenStream::new()
}
```

### Using syn and quote for Safe Parsing

The `syn` and `quote` crates provide structured parsing and code generation:

```rust
use syn::{parse_macro_input, DeriveInput, Data, Fields};
use quote::quote;

#[proc_macro_derive(MyDerive)]
pub fn my_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let expanded = quote! {
        impl MyTrait for #name {
            fn method(&self) -> String {
                String::from("generated")
            }
        }
    };

    TokenStream::from(expanded)
}
```

## 3. Error Handling in Macros

### Compile-Time Errors

```rust
use syn::Error;

fn validate_fields(fields: &Fields) -> syn::Result<()> {
    for field in fields.iter() {
        if field.ident.is_none() {
            return Err(Error::new_spanned(
                field,
                "unnamed fields not supported"
            ));
        }
    }
    Ok(())
}
```

## 4. Advanced Macro Patterns

### DSL Construction

Macros can create domain-specific languages:

```rust
macro_rules! html {
    ($tag:ident { $($content:tt)* }) => {
        format!("<{0}>{1}</{0}>",
                stringify!($tag),
                html!($($content)*))
    };
    ($text:expr) => {
        $text.to_string()
    };
}
```

## 5. Performance Considerations

### Compilation Time Impact

Macros expand during compilation, adding to build time. Mitigation strategies:
- Limit macro expansion scope
- Use procedural macros for complex logic (better caching)
- Profile with `cargo build --timings`

### Code Size Impact

Macros can cause code bloat through monomorphization. Each call with different types generates new code.

## 6. Common Macro Pitfalls

### Operator Precedence

```rust
// WRONG - precedence issues
macro_rules! bad_multiply {
    ($a:expr, $b:expr) => {
        $a * $b
    };
}

let result = bad_multiply!(2 + 3, 4); // Expands to: 2 + 3 * 4 = 14, not 20

// CORRECT - wrap in parentheses
macro_rules! good_multiply {
    ($a:expr, $b:expr) => {
        ($a) * ($b)
    };
}

let result = good_multiply!(2 + 3, 4); // Expands to: (2 + 3) * (4) = 20
```

## 7. AI Agent Macro Analysis Guidelines

When analyzing Rust code with macros, AI agents should:

### Understanding Expansion

1. **Identify macro invocations** - Look for `!` suffix
2. **Determine macro type** - Declarative vs procedural
3. **Trace expansion** - Use `cargo expand` mentally
4. **Understand generated code** - What does the expansion produce?

### Verifying Correctness

```rust
// When seeing this:
#[derive(Debug, Clone)]
struct User {
    name: String,
    age: u32,
}

// Recognize it generates:
impl Debug for User { /* ... */ }
impl Clone for User { /* ... */ }
```

### Detecting Macro-Related Bugs

Common issues to watch for:
- **Missing parentheses** in expressions
- **Incorrect fragment specifiers**
- **Hygiene violations**
- **Type mismatches** in generated code
- **Missing trait bounds** in generic macros

## Conclusion

Rust's macro system bridges compile-time metaprogramming with runtime type safety. For AI agents analyzing Rust code:

1. Recognize macro patterns and mentally expand invocations
2. Understand generated code structure
3. Watch for common pitfalls (precedence, bounds, hygiene)
4. Use `cargo expand` to verify expansions
5. Balance compile-time costs with runtime benefits

**Key Takeaways:**
- Macros operate on token trees, not text
- Hygiene prevents accidental variable capture
- Use `syn` and `quote` for safe procedural macros
- Always parenthesize macro expressions
- Test macro output with `cargo expand`
