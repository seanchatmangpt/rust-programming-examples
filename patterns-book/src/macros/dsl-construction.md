# Pattern: Domain-Specific Language Construction with Macros

## Context

You are building a Rust library and want to provide users with an intuitive, domain-specific syntax that feels natural for your problem domain. You need syntax that doesn't look like normal Rust code—perhaps JSON notation, HTML tags, SQL queries, or configuration syntax. The syntax should be checked at compile time, integrate seamlessly with Rust code, and provide good error messages.

You have mastered **MACRO_RULES**, **FRAGMENT SPECIFIERS**, **RECURSIVE MACRO**, and **MACRO HYGIENE** patterns. Now you want to combine these techniques to build a complete domain-specific language (DSL) embedded in Rust.

## Problem

**How do you create a domain-specific language that provides intuitive syntax for your domain while maintaining Rust's compile-time safety and zero-cost abstractions?**

Users of your library need to express domain concepts clearly:

```rust
// What users want to write (JSON DSL):
let config = json!({
    "server": {
        "host": "localhost",
        "port": 8080,
        "features": ["logging", "metrics"]
    },
    "timeout": 30
});

// Instead of verbose Rust code:
let config = {
    let mut server = HashMap::new();
    server.insert("host".to_string(), Json::String("localhost".to_string()));
    server.insert("port".to_string(), Json::Number(8080.0));
    let mut features = Vec::new();
    features.push(Json::String("logging".to_string()));
    features.push(Json::String("metrics".to_string()));
    server.insert("features".to_string(), Json::Array(features));
    let mut root = HashMap::new();
    root.insert("server".to_string(), Json::Object(Box::new(server)));
    root.insert("timeout".to_string(), Json::Number(30.0));
    Json::Object(Box::new(root))
};
```

The DSL version is:
- More readable (matches the domain)
- Less error-prone (less boilerplate)
- Easier to maintain (structure is clear)
- Still type-safe (compile-time checking)

But implementing it requires:
- Custom syntax that doesn't follow normal Rust rules
- Nested structures with arbitrary depth
- Integration with Rust expressions and variables
- Clear error messages when syntax is wrong

## Forces

- **Domain Alignment**: Syntax should match how domain experts think about the problem
- **Type Safety**: The DSL should be checked at compile time, not runtime
- **Zero Cost**: Generated code should be as efficient as hand-written Rust
- **Composability**: DSL elements should nest and combine naturally
- **Error Messages**: Wrong syntax should produce clear, helpful errors
- **Integration**: DSL should work seamlessly with normal Rust code
- **Hygiene**: DSL internal implementation shouldn't conflict with user code
- **Extensibility**: Adding new DSL features shouldn't break existing code
- **Learning Curve**: DSL should be intuitive even for users new to Rust
- **Debugging**: Users need to understand what code is generated

## Solution

**Design a macro-based DSL by combining recursive pattern matching, fragment specifiers, hygiene, and absolute paths to create compile-time syntax transformation that generates efficient runtime code.**

### DSL Design Principles

1. **Match Domain Semantics**: Syntax should mirror the domain's natural expression
2. **Use Recursive Patterns**: Handle nested structures with recursive macro expansion
3. **Choose Appropriate Fragment Specifiers**: Use `tt` for flexibility, `expr` for values
4. **Ensure Hygiene**: Use unique variable names and `$crate::` paths
5. **Provide Multiple Entry Points**: Support both simple and complex use cases
6. **Generate Efficient Code**: Expand to zero-cost abstractions

### Complete Example: JSON DSL

From `/home/user/rust-programming-examples/json-macro/`:

#### 1. Define the Domain Type

```rust
// lib.rs
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

This represents JSON values in Rust.

#### 2. Implement Conversions

```rust
// From trait for automatic conversion
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

// ... implementations for all numeric types
```

This allows `Json::from(42)`, `Json::from(true)`, etc.

#### 3. Build the DSL Macro

```rust
// macros.rs
#[macro_export]
macro_rules! json {
    // Rule 1: null literal (domain keyword)
    (null) => {
        $crate::Json::Null
    };

    // Rule 2: arrays (recursive structure)
    ([ $( $element:tt ),* ]) => {
        $crate::Json::Array(vec![ $( json!($element) ),* ])
        //                        ^^^^^ Recursive: process each element
    };

    // Rule 3: objects (recursive key-value pairs)
    ({ $( $key:tt : $value:tt ),* }) => {
        {
            // Hygienic variable - won't conflict with user code
            let mut fields = $crate::macros::Box::new(
                $crate::macros::HashMap::new());
            $(
                fields.insert(
                    $crate::macros::ToString::to_string($key),
                    json!($value)  // Recursive: process each value
                );
            )*
            $crate::Json::Object(fields)
        }
    };

    // Rule 4: fallback for literals and expressions
    ($other:tt) => {
        $crate::Json::from($other)
        // Uses From trait implementations
    };
}
```

#### 4. DSL Usage Examples

**Simple Values**:
```rust
json!(null)         // Json::Null
json!(true)         // Json::Boolean(true)
json!(42)           // Json::Number(42.0)
json!("hello")      // Json::String("hello".to_string())
```

**Arrays**:
```rust
json!([1, 2, 3])    // Json::Array(vec![...])
json!([])           // Empty array
json!([null, true, 42])  // Mixed types
```

**Objects**:
```rust
json!({
    "name": "Alice",
    "age": 30
})
```

**Nested Structures**:
```rust
json!({
    "user": {
        "name": "Bob",
        "roles": ["admin", "user"]
    },
    "active": true
})
```

**Integration with Rust Code** (from test at lines 35-49):
```rust
const HELLO: &'static str = "hello";
let value = json!({
    "math_works": (4 - 2 == 2),  // Rust expression
    "en": HELLO,                  // Rust constant
    HELLO: "bonjour!"             // Constant as key
});
```

### How the DSL Works

**Expansion Process** for:
```rust
json!({
    "enabled": true,
    "count": 5
})
```

**Step 1**: Match object rule
```rust
({ $( $key:tt : $value:tt ),* })
// Captures:
// $key[0] = "enabled", $value[0] = true
// $key[1] = "count",   $value[1] = 5
```

**Step 2**: Expand to code
```rust
{
    let mut fields = Box::new(HashMap::new());
    fields.insert("enabled".to_string(), json!(true));
    fields.insert("count".to_string(), json!(5));
    Json::Object(fields)
}
```

**Step 3**: Recursively expand `json!(true)` and `json!(5)`
```rust
json!(true)  → Json::from(true)  → Json::Boolean(true)
json!(5)     → Json::from(5)     → Json::Number(5.0)
```

**Step 4**: Final expanded code
```rust
{
    let mut fields#123 = Box::new(HashMap::new());  // Hygienic name
    fields#123.insert("enabled".to_string(), Json::Boolean(true));
    fields#123.insert("count".to_string(), Json::Number(5.0));
    Json::Object(fields#123)
}
```

## Resulting Context

### Benefits Demonstrated

**Domain-Aligned Syntax** (from test at lines 72-88):

Users write JSON that looks like JSON:
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

Compare to equivalent hand-coded version (lines 44-70): 35+ lines of HashMap and Vec construction.

**Compile-Time Checking**:
```rust
json!({
    "key":  // Missing value
});
// Error: expected one of `,`, `}`, found `)`
// Caught at compile time, before the program runs
```

**Zero Runtime Cost**:

The macro generates the same code you would write by hand. No parsing, no runtime overhead. The `assert_eq!` tests (lines 49, 88, 107, 124) verify that macro-generated and hand-coded values are identical.

**Seamless Integration** (from test at lines 111-125):

```rust
let width = 4.0;
let desc = json!({
    "width": width,                  // Rust variable
    "height": (width * 9.0 / 4.0)    // Rust expression
});
```

Rust expressions work inside the DSL seamlessly.

**Hygiene Protection** (from test at lines 127-144):

```rust
let fields = "user data";  // User's variable
let obj = json!({
    "actor": fields  // Uses user's variable
});
// Macro's internal 'fields' variable doesn't conflict
```

### Real-World Comparison

**With DSL**:
```rust
let config = json!({
    "database": {
        "host": "localhost",
        "port": 5432
    }
});
```
3 lines, clear structure, easy to maintain.

**Without DSL**:
```rust
let mut db = HashMap::new();
db.insert("host".to_string(), Json::String("localhost".to_string()));
db.insert("port".to_string(), Json::Number(5432.0));
let mut config = HashMap::new();
config.insert("database".to_string(), Json::Object(Box::new(db)));
let config = Json::Object(Box::new(config));
```
6 lines, repeated boilerplate, error-prone.

### New Capabilities

With this DSL pattern, you can now:
- Build other DSLs (HTML, CSS, SQL, configuration formats)
- Combine DSLs (embed JSON in HTML, SQL in configuration)
- Extend the DSL with new syntax forms
- Maintain type safety across all transformations
- Generate optimal code without runtime parsing

### Challenges

**Complex Error Messages**:

When users make syntax errors, the error points to the expansion site:
```rust
json!({
    "key" => "value"  // Wrong: should be ':' not '=>'
});
// Error: no rules expected the token `=>`
```

The error is clear, but macro errors can sometimes be cryptic.

**Debugging Generated Code**:

Users see the DSL, not the generated code. Use `cargo expand` to debug:
```bash
cargo expand json_tests::hygiene
```

**Compilation Time**:

Complex macros with deep nesting increase compile time. The json! macro is lightweight, but more complex DSLs can slow compilation.

## Related Patterns

- **MACRO_RULES**: Foundation for DSL construction
- **RECURSIVE MACRO**: Essential for nested DSL structures
- **FRAGMENT SPECIFIERS**: Define DSL syntax constraints
- **MACRO HYGIENE**: Protect DSL implementation from user code

## Known Uses

### serde_json::json!

The original inspiration for this example:

```rust
use serde_json::json;

let value = json!({
    "name": "serde",
    "features": ["derive", "std"]
});
```

Production-quality JSON DSL with comprehensive support.

### html! (yew)

HTML DSL for web development:

```rust
html! {
    <div class="container">
        <h1>{ "Hello, World!" }</h1>
        <button onclick={callback}>{ "Click me" }</button>
    </div>
}
```

### diesel::table!

Database schema DSL:

```rust
table! {
    users (id) {
        id -> Integer,
        name -> Text,
        email -> Text,
    }
}
```

### quote! (proc_macro2)

DSL for generating Rust code in procedural macros:

```rust
quote! {
    impl MyTrait for #type_name {
        fn method(&self) -> i32 {
            #value
        }
    }
}
```

### tokio::select!

Async operation selection DSL:

```rust
select! {
    result = async_op_1() => {
        println!("Op 1 completed: {:?}", result);
    }
    result = async_op_2() => {
        println!("Op 2 completed: {:?}", result);
    }
}
```

### clap::command!

Command-line argument parsing DSL:

```rust
command!("myapp")
    .arg(arg!(-c --config <FILE> "Config file"))
    .arg(arg!(-v --verbose "Verbose output"))
```

## DSL Design Patterns

### Pattern 1: Literal Keywords for Domain Concepts

```rust
// JSON: null, true, false
(null) => { Json::Null };
(true) => { Json::Boolean(true) };

// SQL: SELECT, WHERE, FROM
(SELECT $($col:ident),* FROM $table:ident) => { ... };
```

### Pattern 2: Nested Structures with Recursion

```rust
// Arrays
([ $( $element:tt ),* ]) => {
    vec![ $( recurse!($element) ),* ]
};

// Objects
({ $( $key:tt : $value:tt ),* }) => {
    create_map![ $( $key => recurse!($value) ),* ]
};
```

### Pattern 3: Fallback to Rust Expressions

```rust
// Catch-all for any other token
($other:tt) => {
    convert_from($other)
};
```

This allows seamless Rust integration.

### Pattern 4: Multiple Syntax Forms

```rust
macro_rules! config {
    // Short form
    ($key:ident) => { ... };

    // Long form with value
    ($key:ident = $value:expr) => { ... };

    // Nested form
    ($key:ident { $($inner:tt)* }) => { ... };
}
```

## Building Your Own DSL

### Step 1: Define Domain Types

```rust
// Example: CSS DSL
pub enum CssValue {
    Pixels(i32),
    Percent(f32),
    Color(String),
}

pub struct CssRule {
    selector: String,
    properties: HashMap<String, CssValue>,
}
```

### Step 2: Design DSL Syntax

Sketch what users should write:
```rust
css! {
    .container {
        width: 100%;
        padding: 10px;
    }
}
```

### Step 3: Implement Macro Rules

```rust
macro_rules! css {
    // Rule pattern
    ( $selector:tt { $( $prop:ident : $value:tt );* } ) => {
        CssRule {
            selector: stringify!($selector).to_string(),
            properties: {
                let mut props = HashMap::new();
                $(
                    props.insert(
                        stringify!($prop).to_string(),
                        parse_css_value!($value)
                    );
                )*
                props
            }
        }
    };
}
```

### Step 4: Add Conversion Helpers

```rust
macro_rules! parse_css_value {
    ($num:literal px) => { CssValue::Pixels($num) };
    ($num:literal %) => { CssValue::Percent($num) };
    ($color:literal) => { CssValue::Color($color.to_string()) };
}
```

### Step 5: Test Extensively

```rust
#[test]
fn test_css_dsl() {
    let rule = css! {
        .container {
            width: 100%;
            padding: 10px;
        }
    };
    assert_eq!(rule.selector, ".container");
    // ... more assertions
}
```

## Advanced DSL Techniques

### Technique 1: Embedded Rust Expressions

Allow Rust code inside the DSL:

```rust
let port = 8080;
let config = json!({
    "server": {
        "port": port,  // Rust variable
        "workers": (num_cpus::get() * 2)  // Rust expression
    }
});
```

Implementation:
```rust
($other:tt) => {
    $crate::Json::from($other)  // Accepts any expression
};
```

### Technique 2: Validation at Compile Time

```rust
macro_rules! rgb {
    ($r:literal, $g:literal, $b:literal) => {{
        // Compile-time range check (if possible with const functions)
        const _: () = assert!($r <= 255);
        const _: () = assert!($g <= 255);
        const _: () = assert!($b <= 255);
        Color { r: $r, g: $g, b: $b }
    }};
}

// rgb!(300, 0, 0); // Compile error: assertion failed
```

### Technique 3: Multiple DSL Layers

Build DSLs that compose:

```rust
html! {
    <div>
        { json!({ "data": "value" }) }  // JSON inside HTML
    </div>
}
```

### Technique 4: Stringify for Meta-Programming

```rust
macro_rules! field_names {
    ($($field:ident),*) => {
        vec![ $( stringify!($field) ),* ]
    };
}

let names = field_names!(name, age, email);
// vec!["name", "age", "email"]
```

## Testing DSLs

### Test 1: Basic Syntax

```rust
#[test]
fn test_simple_values() {
    assert_eq!(json!(null), Json::Null);
    assert_eq!(json!(true), Json::Boolean(true));
    assert_eq!(json!(42), Json::Number(42.0));
}
```

### Test 2: Nested Structures

```rust
#[test]
fn test_nested() {
    let value = json!({
        "array": [1, 2, 3],
        "object": {
            "key": "value"
        }
    });
    // Assert structure
}
```

### Test 3: Rust Integration

```rust
#[test]
fn test_expressions() {
    let x = 10;
    let value = json!({
        "computed": (x * 2)
    });
    // Verify expression evaluated
}
```

### Test 4: Hygiene

```rust
#[test]
fn test_hygiene() {
    let fields = "user data";
    let value = json!({ "data": fields });
    assert_eq!(fields, "user data");  // Not modified
}
```

### Test 5: Equivalence with Hand-Coded

```rust
#[test]
fn test_equivalence() {
    let dsl_value = json!({ "key": "value" });
    let hand_coded = {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Json::String("value".to_string()));
        Json::Object(Box::new(map))
    };
    assert_eq!(dsl_value, hand_coded);
}
```

## Common Pitfalls

### Pitfall 1: Too Complex Syntax

```rust
// ❌ BAD: Overly complex syntax
macro_rules! complex_dsl {
    ($($a:tt => [$($b:tt)|+] @ $c:tt { $($d:tt)* })*) => { ... };
}
// Users won't understand this
```

**Solution**: Keep syntax simple and domain-aligned.

### Pitfall 2: Poor Error Messages

```rust
// ❌ BAD: Generic error
macro_rules! bad_dsl {
    ($($tt:tt)*) => { compile_error!("Invalid syntax") };
}
```

**Solution**: Provide specific error messages:

```rust
macro_rules! good_dsl {
    (null) => { Null };
    (true) => { Boolean(true) };
    ($invalid:tt) => {
        compile_error!(
            concat!("Expected 'null' or 'true', found: ",
                    stringify!($invalid))
        )
    };
}
```

### Pitfall 3: Forgetting Hygiene

```rust
// ❌ BAD: Not using $crate
#[macro_export]
macro_rules! bad_dsl {
    () => { MyType::new() };  // Which MyType?
}

// ✅ GOOD: Absolute path
#[macro_export]
macro_rules! good_dsl {
    () => { $crate::MyType::new() };
}
```

### Pitfall 4: Insufficient Testing

Test with:
- Simple cases
- Nested structures
- Rust expressions
- Variable conflicts (hygiene)
- Cross-crate usage
- Error cases

## Performance Considerations

### Zero-Cost Abstraction

DSLs should generate the same code as hand-written:

```rust
// Both generate identical assembly
let v1 = json!([1, 2, 3]);
let v2 = Json::Array(vec![
    Json::Number(1.0),
    Json::Number(2.0),
    Json::Number(3.0),
]);
```

Verify with:
```bash
cargo rustc --release -- --emit asm
```

### Compile-Time Impact

Measure compilation time:
```bash
cargo clean
cargo build --timings
```

Check the timings report for macro expansion overhead.

### Runtime Impact

DSLs should have zero runtime cost:
- No parsing at runtime
- No dynamic dispatch (unless intended)
- Same performance as hand-coded

## Documentation for DSL Users

### Provide Examples

```rust
/// Create JSON values with intuitive syntax.
///
/// # Examples
///
/// ```
/// use json_macro::json;
///
/// let value = json!({
///     "name": "Alice",
///     "age": 30
/// });
/// ```
#[macro_export]
macro_rules! json { ... }
```

### Document Syntax Rules

```markdown
## Syntax

- `null` - null value
- `true`, `false` - booleans
- `42`, `3.14` - numbers
- `"string"` - strings
- `[elem1, elem2]` - arrays
- `{"key": value}` - objects
- Rust expressions are supported as values
```

### Show Integration Examples

```rust
/// # Integration with Rust
///
/// ```
/// let count = 5;
/// let value = json!({
///     "count": count,
///     "doubled": (count * 2)
/// });
/// ```
```

## Implementation Checklist

- [ ] Define domain types (enums, structs)
- [ ] Implement From/Into conversions
- [ ] Design DSL syntax (sketch examples)
- [ ] Write macro rules from specific to general
- [ ] Use `tt` for nested structures
- [ ] Add `$crate::` to all item paths
- [ ] Re-export std types used by macro
- [ ] Test with simple cases
- [ ] Test with nested structures
- [ ] Test Rust expression integration
- [ ] Test hygiene (variable conflicts)
- [ ] Test cross-crate usage
- [ ] Document syntax and examples
- [ ] Verify zero-cost abstraction (assembly)
- [ ] Measure compilation time impact
- [ ] Provide helpful error messages

## Further Reading

- **The Little Book of Rust Macros**: https://veykril.github.io/tlborm/
- **serde_json::json! Implementation**: https://docs.rs/serde_json/latest/src/serde_json/macros.rs.html
- **Yew html! Macro**: https://yew.rs/docs/concepts/html
- **Diesel Query DSL**: https://diesel.rs/guides/all-about-query-dsl
- **Building a DSL in Rust (Blog Series)**: Various community articles
- **cargo-expand**: Essential for debugging DSL macros: https://github.com/dtolnay/cargo-expand
