# Pattern: Macro Hygiene and Scope Safety

## Context

You are writing **DECLARATIVE MACROS** that introduce variables, use external items (types, functions, traits), or define temporary bindings. Your macro might be invoked in code where variable names coincidentally match names you use internally in the macro expansion. You want to ensure that your macro doesn't accidentally capture user variables, and that user code doesn't accidentally interfere with your macro's internal operations.

You have learned **MACRO_RULES**, **FRAGMENT SPECIFIERS**, and possibly **RECURSIVE MACRO** patterns, but need to understand how Rust prevents name collisions between macro-generated code and user code.

## Problem

**How do you ensure that variables and items used within a macro don't conflict with names in the code where the macro is invoked?**

Consider a macro that creates a temporary variable:

```rust
// POTENTIALLY PROBLEMATIC: What if user has 'temp'?
macro_rules! double {
    ($value:expr) => {{
        let temp = $value;
        temp + temp
    }};
}

// User code
let temp = "I'm the user's variable";
let result = double!(5);
println!("{}", temp);  // Which 'temp' is this?
```

Without hygiene, macros would be fragile:
- Macros couldn't use common variable names like `temp`, `result`, or `value`
- Users would need to know macro implementation details to avoid conflicts
- Refactoring macro internals could break existing code
- Macros from different crates could conflict with each other

Similarly, macros need to reference items (types, functions) that might not be in scope at the invocation site.

## Forces

- **Name Collision**: Macro-introduced names might conflict with user code
- **Scope Isolation**: Macro internals should be invisible to surrounding code
- **Intentional Capture**: Sometimes macros *should* use user-provided variables
- **Item Visibility**: Macros need to reference items from their defining crate
- **Predictability**: Macro behavior should be consistent regardless of invocation context
- **Debugging**: Developers need to understand which names refer to what
- **Backward Compatibility**: Hygiene rules affect macro design across Rust editions
- **Performance**: Hygiene must not introduce runtime overhead

## Solution

**Rely on Rust's automatic macro hygiene for local variables, and use the `$crate::` prefix for item paths to ensure macros reference the correct items regardless of where they're invoked.**

Rust provides two key hygiene mechanisms:

### 1. Automatic Variable Hygiene

**Variables introduced by a macro are automatically renamed to avoid conflicts with user code.**

```rust
macro_rules! double {
    ($value:expr) => {{
        let temp = $value;  // 'temp' is automatically unique
        temp + temp
    }};
}
```

When expanded, Rust internally renames `temp` to something like `temp#123` (conceptually—the actual mechanism is different). User code cannot access this variable, and the macro cannot accidentally access a user variable named `temp`.

### 2. Absolute Paths with $crate

**Use `$crate::` to reference items from your crate, regardless of the invocation context.**

```rust
// In your crate
pub struct MyType;

#[macro_export]
macro_rules! use_my_type {
    () => {
        $crate::MyType::new()
        // ^^^^^^ Absolute path from macro's defining crate
    };
}
```

### Real Example: json! Macro Hygiene

From `/home/user/rust-programming-examples/json-macro/src/macros.rs`:

```rust
({ $( $key:tt : $value:tt ),* }) => {
    {
        let mut fields = $crate::macros::Box::new(
            $crate::macros::HashMap::new());
        //      ^^^^^^ Absolute path ensures we get the right HashMap
        $(
            fields.insert($crate::macros::ToString::to_string($key),
                          json!($value));
        )*
        $crate::Json::Object(fields)
        //      ^^^^ Absolute path to Json type
    }
};
```

**Why these paths?**

- `$crate::macros::Box`: Ensures we use `Box` from std, not a user-defined `Box`
- `$crate::macros::HashMap`: Ensures we use the standard `HashMap`
- `$crate::Json::Object`: Creates the correct `Json` enum variant

The `macros` module re-exports these items:

```rust
// macros.rs
pub use std::collections::HashMap;
pub use std::boxed::Box;
pub use std::string::ToString;
```

This allows the macro to use absolute paths while keeping the expansion readable.

### Hygiene Test Case

From lines 127-144 in macros.rs:

```rust
#[test]
fn hygiene() {
    // The surprise is that *the macro works as-is*.
    // Rust renames the variable for you!

    let fields = "Fields, W.C.";
    //  ^^^^^^ User variable named 'fields'

    let role = json!({
        "name": "Larson E. Whipsnade",
        "actor": fields  // Uses user's 'fields'
    });
    // Inside the macro, there's also a variable named 'fields':
    // let mut fields = Box::new(HashMap::new());
    //
    // But Rust renames the macro's 'fields' internally,
    // so no conflict occurs!

    assert_eq!(role, /* ... */);
}
```

**What happens:**
1. User code has: `let fields = "Fields, W.C.";`
2. Macro creates: `let mut fields = Box::new(HashMap::new());`
3. Rust renames macro's `fields` to something like `fields#macro_123`
4. User's `fields` remains `fields`
5. Inside macro, `fields.insert(...)` refers to `fields#macro_123`
6. The `$value` containing `fields` refers to user's `fields`
7. No conflict!

## How Hygiene Works

### Syntax Contexts

Each token carries a "syntax context" that tracks where it came from:
- Tokens from macro definitions have the macro's context
- Tokens from macro invocations have the invocation site's context
- Tokens from macro arguments preserve their original context

```rust
macro_rules! demo {
    ($user_var:ident) => {{
        let macro_var = 10;
        //  ^^^^^^^^^ Has macro context
        $user_var + macro_var
        // ^^^^^^^^^ Has invocation site context
    }};
}

let x = 5;
//  ^ Has main function context
demo!(x);
// Expands to (conceptually):
// {
//     let macro_var#demo_123 = 10;
//     x + macro_var#demo_123
// }
```

The compiler uses syntax contexts to:
1. Resolve which `macro_var` is which (if user also has one)
2. Prevent macro from accessing user's variables it shouldn't see
3. Allow macro to use its own local variables safely

### What Is Hygienic

**Variables introduced by the macro:**
```rust
macro_rules! hygienic_var {
    () => {{
        let temp = 42;  // Hygienic - won't conflict
        temp
    }};
}
```

**Labels:**
```rust
macro_rules! hygienic_label {
    ($value:expr) => {
        'macro_loop: loop {  // Hygienic label
            if $value { break 'macro_loop; }
        }
    };
}
```

### What Is NOT Hygienic

**Item names:**
```rust
macro_rules! not_hygienic {
    () => {
        struct MyStruct;  // NOT hygienic - visible to caller
        fn my_function() {}  // NOT hygienic
    };
}
```

These are intentionally visible because macros often need to generate public items.

**Variables from arguments:**
```rust
macro_rules! uses_arg {
    ($var:ident) => {
        $var = 10;  // NOT hygienic - intentionally uses user's variable
    };
}

let mut x = 5;
uses_arg!(x);  // Sets user's x to 10
```

## Resulting Context

### Benefits

**Robust Macros**: The json! macro uses `fields` as a variable name without worrying about conflicts:

```rust
let fields = "some data";  // User's variable
let obj = json!({
    "data": fields  // Works! Uses user's 'fields'
});
// Macro's internal 'fields' variable doesn't interfere
```

**Predictable Behavior**: Macros work the same way regardless of surrounding code:

```rust
// Both work identically
{
    let result = double!(5);
}

{
    let temp = "unrelated";
    let result = double!(5);
    // 'temp' doesn't interfere with macro's internal 'temp'
}
```

**Crate Independence**: The $crate prefix ensures macros work across crate boundaries:

```rust
// In json_macro crate
#[macro_export]
macro_rules! json {
    (null) => { $crate::Json::Null };
}

// In user's crate
use json_macro::json;
let value = json!(null);  // Works! Uses json_macro::Json
```

### Real-World Impact

From the hygiene test (lines 127-144), the test explicitly verifies this behavior:

```rust
let fields = "Fields, W.C.";
let role = json!({
    "name": "Larson E. Whipsnade",
    "actor": fields  // User's 'fields'
});
// Macro creates its own 'fields' internally
// Both coexist without conflict
```

The comment at line 129 highlights the "surprise":
```rust
// The surprise is that *the macro works as-is*.
// Rust renames the variable for you!
```

This would cause a name collision in many other languages' macro systems (like C preprocessor), but Rust's hygiene makes it just work.

### New Challenges

**Intentional Capture**: When you *want* to use a user variable, you must pass it explicitly:

```rust
// ❌ BAD: Can't access user's variable by name
macro_rules! broken {
    () => {
        user_var + 1  // Error: can't find 'user_var'
    };
}

// ✅ GOOD: Accept it as a parameter
macro_rules! working {
    ($var:ident) => {
        $var + 1
    };
}

let user_var = 5;
working!(user_var);
```

**Module Paths**: You need `$crate::` for all item references:

```rust
// ❌ BAD: Assumes Json is in scope
macro_rules! bad {
    (null) => { Json::Null };
}

// ✅ GOOD: Uses absolute path
#[macro_export]
macro_rules! good {
    (null) => { $crate::Json::Null };
}
```

**Debugging**: Hygiene can make debugging harder because variable names in errors might not match source:

```bash
error: cannot find value `temp#12345` in this scope
```

Use `cargo expand` to see the actual expansion.

### When You Have This Pattern

You can now:
- Write macros that safely use common variable names
- Export macros that work across crate boundaries
- Combine multiple macros without name conflicts
- Build **DSL CONSTRUCTION** with internal state safely
- Refactor macro internals without breaking users

## Related Patterns

- **MACRO_RULES**: Hygiene applies to all macro_rules! macros
- **FRAGMENT SPECIFIERS**: Captured fragments preserve their original context
- **RECURSIVE MACRO**: Each recursive expansion maintains hygiene
- **DSL CONSTRUCTION**: Hygiene enables complex DSLs with internal state

## Known Uses

### Standard Library: vec!

```rust
vec![1, 2, 3]

// Expands to something like:
{
    let mut temp_vec = Vec::new();
    // 'temp_vec' is hygienic - won't conflict with user code
    temp_vec.push(1);
    temp_vec.push(2);
    temp_vec.push(3);
    temp_vec
}
```

Users can have their own `temp_vec` variable without conflict.

### Standard Library: println!

```rust
println!("Value: {}", x);

// Uses hygienic variables internally and $crate:: for paths
// Something like:
$crate::io::_print($crate::format_args!("Value: {}", x));
```

### lazy_static!

```rust
lazy_static! {
    static ref CONFIG: Config = load_config();
}

// Uses hygienic variables for initialization guard
// Users can have variables with common names
```

### serde_json::json!

Like our example, uses hygiene for internal `map` or `vec` variables:

```rust
json!({
    "key": "value"
})

// Internally creates variables like 'map' or 'object'
// These don't conflict with user code
```

### diesel query builder

```rust
users.filter(active.eq(true))

// Macro generates code with temporary variables
// All hygienic, won't conflict with user's variables
```

## Examples

### Example 1: Safe Temporary Variables

```rust
macro_rules! swap {
    ($a:expr, $b:expr) => {{
        let temp = $a;
        // 'temp' is hygienic - safe to use
        $a = $b;
        $b = temp;
    }};
}

let temp = "user's temp";
let mut x = 1;
let mut y = 2;
swap!(x, y);
// Works! Macro's 'temp' doesn't interfere
println!("{}", temp);  // Still "user's temp"
```

### Example 2: $crate for Item Access

```rust
// my_crate/src/lib.rs
pub struct Counter(pub i32);

impl Counter {
    pub fn new() -> Self {
        Counter(0)
    }
}

#[macro_export]
macro_rules! create_counter {
    () => {
        $crate::Counter::new()
        // ^^^^^^ Ensures we get the right Counter
    };
}

// other_crate/src/main.rs
use my_crate::create_counter;

struct Counter;  // User's own Counter type

let c = create_counter!();  // Gets my_crate::Counter, not user's
```

### Example 3: Re-exporting Items for Hygiene

```rust
// lib.rs
pub struct MyType;

#[macro_use]
mod macros_impl;

// macros_impl.rs
pub use std::vec::Vec;  // Re-export for macro use

#[macro_export]
macro_rules! create_vec {
    () => {
        $crate::macros_impl::Vec::new()
        // Uses re-exported Vec, not potentially shadowed one
    };
}
```

This is exactly what json-macro does with `Box`, `HashMap`, and `ToString`.

### Example 4: Intentional Capture via Parameters

```rust
macro_rules! increment {
    ($var:ident) => {
        $var += 1;
        // Intentionally modifies user's variable
    };
}

let mut count = 0;
increment!(count);
println!("{}", count);  // 1
```

Here, the macro *wants* to access the user's variable, so it's passed explicitly.

## Hygiene Patterns for Macro Authors

### Pattern 1: Use Descriptive Internal Names

Even though hygiene protects you, use clear names for maintainability:

```rust
// ✅ GOOD: Clear intent
macro_rules! create_collection {
    () => {{
        let mut temp_collection = Vec::new();
        temp_collection.push(1);
        temp_collection
    }};
}

// ✗ WORKS BUT UNCLEAR
macro_rules! create_collection {
    () => {{
        let mut x = Vec::new();
        x.push(1);
        x
    }};
}
```

### Pattern 2: Always Use $crate for Exported Macros

```rust
// ✅ GOOD: Works across crates
#[macro_export]
macro_rules! public_macro {
    () => {
        $crate::MyType::new()
    };
}

// ✗ BAD: Breaks when used in other crates
#[macro_export]
macro_rules! public_macro {
    () => {
        MyType::new()  // Which MyType?
    };
}
```

### Pattern 3: Re-export Dependencies

If your macro uses std types, re-export them:

```rust
// macros_support.rs
pub use std::collections::HashMap;
pub use std::boxed::Box;

// macro definition
#[macro_export]
macro_rules! my_macro {
    () => {
        $crate::macros_support::HashMap::new()
    };
}
```

This ensures:
1. Absolute path to the type
2. Protection from shadowing
3. Clear dependency on std types

## Common Pitfalls and Solutions

### Pitfall 1: Assuming Items Are Hygienic

```rust
// ❌ BAD: Assumes struct is only in macro
macro_rules! create_struct {
    () => {
        struct Data { value: i32 }
        // NOT hygienic - visible to caller
        Data { value: 42 }
    };
}

// Conflicts if user has their own Data
```

**Solution**: Use unique names or generate via identifiers:

```rust
macro_rules! create_struct {
    ($name:ident) => {
        struct $name { value: i32 }
        $name { value: 42 }
    };
}

create_struct!(MyData);  // User controls name
```

### Pitfall 2: Forgetting $crate in Exported Macros

```rust
// ❌ BAD: Won't work in other crates
#[macro_export]
macro_rules! bad_macro {
    () => {
        Json::from(42)  // Which Json?
    };
}

// ✅ GOOD: Absolute path
#[macro_export]
macro_rules! good_macro {
    () => {
        $crate::Json::from(42)
    };
}
```

### Pitfall 3: Over-Relying on Hygiene

```rust
// ❌ QUESTIONABLE: Works but unclear
macro_rules! confusing {
    () => {{
        let result = {
            let result = 42;
            result
        };
        result
    }};
}
```

Even though hygiene makes this work, it's confusing. Use distinct names:

```rust
// ✅ GOOD: Clear intent
macro_rules! clear {
    () => {{
        let intermediate = {
            let value = 42;
            value
        };
        intermediate
    }};
}
```

### Pitfall 4: Shadowing std Types

```rust
// User code
struct Vec;  // Shadows std::vec::Vec

// ❌ BAD: Uses shadowed Vec
macro_rules! bad {
    () => {
        Vec::new()  // Gets user's Vec!
    };
}

// ✅ GOOD: Uses absolute path
macro_rules! good {
    () => {
        $crate::macros_support::Vec::new()
    };
}
```

## Testing Hygiene

### Test 1: Variable Name Conflicts

```rust
#[test]
fn test_hygiene_variables() {
    let temp = "user's temp";
    let result = double!(5);
    assert_eq!(result, 10);
    assert_eq!(temp, "user's temp");  // User's var unchanged
}
```

### Test 2: Item Shadowing

```rust
#[test]
fn test_hygiene_items() {
    struct Json;  // Shadow the macro's Json type

    let value = json!(null);  // Should still work
    // Gets $crate::Json, not local Json
}
```

### Test 3: Cross-Crate Usage

```rust
// In a separate integration test crate
use json_macro::json;

#[test]
fn test_cross_crate() {
    let value = json!({"key": "value"});
    // Macro must use $crate:: for this to work
}
```

## Debugging Hygiene Issues

### Use cargo expand

```bash
cargo install cargo-expand
cargo expand

# Or for a specific item:
cargo expand json_tests::hygiene
```

Shows the actual expanded code with hygiene markers.

### Check Error Messages

If you see errors like:
```
error: cannot find value `fields#1234` in this scope
```

This indicates a hygiene issue. The `#1234` suffix shows Rust's internal renaming.

### Verify $crate Usage

Search your macro for absolute paths:
```bash
grep -n '\$crate::' src/macros.rs
```

Every item reference should use `$crate::`.

## Implementation Checklist

- [ ] Use hygienic variable names for internal temporary variables
- [ ] Add `$crate::` prefix to all item paths in exported macros
- [ ] Re-export std types used by macros
- [ ] Test macros with variable names that match internal names
- [ ] Test macros in separate crates (integration tests)
- [ ] Document which variables are intentionally captured
- [ ] Use `cargo expand` to verify expansions
- [ ] Check for item shadowing in tests
- [ ] Ensure labels are hygienic if used
- [ ] Verify hygiene across different Rust editions

## Further Reading

- **Rust Reference - Macro Hygiene**: https://doc.rust-lang.org/reference/macros-by-example.html#hygiene
- **The Little Book of Rust Macros - Hygiene**: https://veykril.github.io/tlborm/decl-macros/minutiae/hygiene.html
- **RFC 1584 - Macros**: Discusses hygiene design: https://rust-lang.github.io/rfcs/1584-macros.html
- **cargo-expand**: Essential tool for debugging hygiene: https://github.com/dtolnay/cargo-expand
