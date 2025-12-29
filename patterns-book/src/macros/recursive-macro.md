# Pattern: Recursive Macro Expansion

## Context

You are building a **DECLARATIVE MACRO** using macro_rules! and need to handle nested or recursive data structures. Your input might contain arbitrarily deep nesting—arrays within objects, objects within arrays, or trees of varying depth. You cannot know at macro definition time how many levels of nesting the user will provide.

Examples include:
- JSON structures with nested objects and arrays
- HTML/XML trees with nested elements
- Expression trees with arbitrary depth
- Configuration formats with nested sections

You have already learned **MACRO_RULES** basics but need to handle inputs that contain your own macro invocations within themselves.

## Problem

**How do you handle arbitrarily nested structures in a macro when you cannot know the depth of nesting in advance?**

Macros process token trees at compile time, expanding patterns into code. However, nested structures require processing at multiple levels:

```rust
// User wants to write this:
json!({
    "name": "Project",
    "config": {
        "version": 1,
        "enabled": true
    },
    "items": [1, 2, 3]
})
```

A naive macro would need separate rules for:
- Objects containing objects (2 levels)
- Objects containing objects containing objects (3 levels)
- Arrays containing objects containing arrays (mixed nesting)
- ... infinitely many combinations

Writing a rule for each possible nesting pattern is impossible. You need the macro to handle its own output as input.

## Forces

- **Arbitrary Depth**: Users may nest structures to any depth; you cannot limit this artificially
- **Uniform Processing**: Each level should be processed identically regardless of depth
- **Token Tree Matching**: Macros work on token trees (tt), not parsed structures
- **Self-Reference**: The macro must be able to invoke itself during expansion
- **Type Safety**: The compiler must verify the final expanded code, not intermediate expansions
- **Termination**: Recursion must have a base case to prevent infinite expansion
- **Pattern Ambiguity**: Multiple macro rules may match the same input; the first match wins
- **Performance**: Deep recursion increases compile time

## Solution

**Define multiple macro rules where some rules invoke the macro itself (recursively) on sub-parts of the input, and ensure at least one base case rule terminates the recursion.**

### Structure

```rust
macro_rules! recursive_macro {
    // Base case(s): terminate recursion
    (base_pattern) => {
        base_expansion
    };

    // Recursive case(s): invoke the macro on sub-parts
    (recursive_pattern) => {
        code_with_recursive_macro!(subpart)
    };
}
```

### Real Example: The json! Macro

From `/home/user/rust-programming-examples/json-macro/src/macros.rs`:

```rust
#[macro_export]
macro_rules! json {
    // Base case 1: null literal
    (null) => {
        $crate::Json::Null
    };

    // Recursive case 1: array with elements
    ([ $( $element:tt ),* ]) => {
        $crate::Json::Array(vec![ $( json!($element) ),* ])
        //                        ^^^^^ Recursive invocation
    };

    // Recursive case 2: object with key-value pairs
    ({ $( $key:tt : $value:tt ),* }) => {
        {
            let mut fields = $crate::macros::Box::new(
                $crate::macros::HashMap::new());
            $(
                fields.insert($crate::macros::ToString::to_string($key),
                              json!($value));
                //            ^^^^^ Recursive invocation
            )*
            $crate::Json::Object(fields)
        }
    };

    // Base case 2: any other token (fallback)
    ($other:tt) => {
        $crate::Json::from($other)
    };
}
```

### How Recursion Works

Consider this invocation:

```rust
json!([
    {
        "nested": true
    }
])
```

**Expansion Steps**:

1. **First expansion** - Array rule matches:
   ```rust
   // Matches: [ $( $element:tt ),* ]
   // $element captures: { "nested": true }

   Json::Array(vec![ json!({ "nested": true }) ])
   //                ^^^^^ Recursive call
   ```

2. **Second expansion** - Object rule matches:
   ```rust
   // Matches: { $( $key:tt : $value:tt ),* }
   // $key captures: "nested"
   // $value captures: true

   {
       let mut fields = Box::new(HashMap::new());
       fields.insert("nested".to_string(), json!(true));
       //                                  ^^^^^ Recursive call
       Json::Object(fields)
   }
   ```

3. **Third expansion** - Fallback rule matches:
   ```rust
   // Matches: $other:tt
   // $other captures: true

   Json::from(true)  // Base case - no more recursion
   ```

4. **Final expanded code**:
   ```rust
   Json::Array(vec![{
       let mut fields = Box::new(HashMap::new());
       fields.insert("nested".to_string(), Json::from(true));
       Json::Object(fields)
   }])
   ```

### Key Principles

**1. Use tt (Token Tree) for Maximum Flexibility**

```rust
// ✅ GOOD: Accepts any valid syntax
([ $( $element:tt ),* ]) => { ... };

// ❌ BAD: Too restrictive - can't handle nested structures
([ $( $element:expr ),* ]) => { ... };
```

Why? Because `{ "key": "value" }` is not an expression—it's a braced token tree. Using `expr` would reject nested objects.

**2. Order Rules from Specific to General**

```rust
macro_rules! json {
    // SPECIFIC: Match special literals first
    (null) => { Json::Null };

    // GENERAL: Catch-all comes last
    ($other:tt) => { Json::from($other) };
}
```

The first matching rule is used. If the general rule came first, it would match everything, and the specific rule would never execute.

**3. Ensure Base Cases Terminate**

```rust
// Base case: no recursive call
($other:tt) => {
    $crate::Json::from($other)  // No json!() invocation
};
```

Without base cases, the macro would recurse infinitely and fail to compile.

## Resulting Context

### Benefits

**Handles Arbitrary Depth**: The json! macro processes structures of any nesting level:

```rust
// 5 levels deep - no problem!
let deep = json!({
    "level1": {
        "level2": {
            "level3": {
                "level4": {
                    "level5": "value"
                }
            }
        }
    }
});
```

**Uniform Processing**: Each level of nesting is handled identically. The same rules apply whether processing the outermost structure or the deepest nested value.

**Composability**: Users can mix arrays and objects freely:

```rust
json!([
    { "type": "object" },
    [1, 2, 3],
    "string",
    null
])
```

### Real-World Example from Tests

From `/home/user/rust-programming-examples/json-macro/src/macros.rs` (lines 72-88):

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

This involves:
- Array containing 2 objects (array rule invokes json! twice)
- Each object contains 3 key-value pairs (object rule invokes json! 6 times)
- Each value is a base case (strings/numbers, 6 base case invocations)

Total: **15 macro invocations** from a single top-level call.

### New Challenges

**Debugging Recursive Expansions**: Errors in deeply nested structures can produce long error messages. Use `cargo expand` to see the full expansion:

```bash
cargo expand
```

**Compile Time Overhead**: Each recursive invocation adds to compile time. For extremely deep structures, this can become noticeable.

**Pattern Overlap**: Multiple rules might match the same input. The first matching rule wins, so rule ordering is critical.

### When You Have This Pattern

You can now:
- Build domain-specific languages with nested syntax (**DSL CONSTRUCTION**)
- Process tree structures at compile time
- Create macros that handle arbitrarily complex configurations
- Combine recursion with **FRAGMENT SPECIFIERS** for type-safe nested structures

## Related Patterns

- **MACRO_RULES**: Foundation for declarative macros; recursive macros build on this
- **FRAGMENT SPECIFIERS**: Use tt for recursion, but expr/ty/ident for base cases where appropriate
- **DSL CONSTRUCTION**: Recursive macros enable complex domain-specific syntax
- **MACRO HYGIENE**: Especially important when recursive macros create multiple scopes

## Known Uses

### Standard Library: vec!

The vec! macro handles nested vectors recursively:

```rust
let nested = vec![
    vec![1, 2],
    vec![3, 4],
];
```

Each inner `vec![...]` is a separate macro invocation.

### serde_json::json!

The inspiration for this example:

```rust
let complex = serde_json::json!({
    "users": [
        {
            "name": "Alice",
            "roles": ["admin", "user"]
        }
    ]
});
```

Handles arbitrary JSON nesting through recursive expansion.

### html! (yew framework)

Creates HTML structures with recursive nesting:

```rust
html! {
    <div>
        <h1>{"Title"}</h1>
        <ul>
            <li>{"Item 1"}</li>
            <li>{"Item 2"}</li>
        </ul>
    </div>
}
```

### diesel::query_builder

Builds SQL queries with nested conditions:

```rust
users
    .filter(
        active.eq(true)
            .and(role.eq("admin")
                .or(role.eq("moderator")))
    )
```

### json-macro Project

Complete implementation demonstrating recursive array and object handling for JSON DSL.

## Examples

### Example 1: Recursive Tree Structure

```rust
#[derive(Debug)]
enum Tree {
    Leaf(i32),
    Branch(Vec<Tree>),
}

macro_rules! tree {
    // Base case: leaf node
    ($value:expr) => {
        Tree::Leaf($value)
    };

    // Recursive case: branch with children
    ([ $( $child:tt ),* ]) => {
        Tree::Branch(vec![ $( tree!($child) ),* ])
        //                 ^^^^^ Recursive invocation
    };
}

// Usage:
let t = tree!([
    1,
    [2, 3],
    [[4, 5], 6]
]);

// Expands to:
// Tree::Branch(vec![
//     Tree::Leaf(1),
//     Tree::Branch(vec![Tree::Leaf(2), Tree::Leaf(3)]),
//     Tree::Branch(vec![
//         Tree::Branch(vec![Tree::Leaf(4), Tree::Leaf(5)]),
//         Tree::Leaf(6)
//     ])
// ])
```

### Example 2: Nested Configuration

```rust
macro_rules! config {
    // Base case: key-value pair
    ($key:ident = $value:expr) => {
        ($key, ConfigValue::Literal($value))
    };

    // Recursive case: nested section
    ($key:ident { $( $inner:tt )* }) => {
        ($key, ConfigValue::Section(vec![ $( config!($inner) ),* ]))
        //                                ^^^^^^ Recursive call
    };
}

let cfg = config!(
    database {
        host = "localhost"
        port = 5432
        connection {
            timeout = 30
            pool_size = 10
        }
    }
);
```

### Example 3: Expression Tree Evaluator

```rust
macro_rules! calc {
    // Base case: number
    ($num:literal) => {
        $num
    };

    // Recursive case: addition
    (( $left:tt + $right:tt )) => {
        calc!($left) + calc!($right)
        //   ^^^^^^       ^^^^^^ Recursive calls
    };

    // Recursive case: multiplication
    (( $left:tt * $right:tt )) => {
        calc!($left) * calc!($right)
    };
}

let result = calc!((2 + (3 * 4)));
// Expands to: 2 + (3 * 4)
```

## Common Pitfalls and Solutions

### Pitfall 1: Infinite Recursion

```rust
// ❌ BAD: No base case
macro_rules! infinite {
    ($x:tt) => {
        infinite!($x)  // Always recurses
    };
}

// ✅ GOOD: Base case terminates
macro_rules! finite {
    (stop) => { 0 };  // Base case
    ($x:tt) => {
        finite!(stop)   // Eventually reaches base case
    };
}
```

### Pitfall 2: Wrong Rule Order

```rust
// ❌ BAD: General rule first, specific never matches
macro_rules! bad_order {
    ($x:tt) => { Generic };     // Matches everything
    (null) => { Specific };     // Never reached
}

// ✅ GOOD: Specific first, general last
macro_rules! good_order {
    (null) => { Specific };     // Matches null
    ($x:tt) => { Generic };     // Matches everything else
}
```

### Pitfall 3: Using expr Instead of tt

```rust
// ❌ BAD: Can't handle nested objects
macro_rules! json_bad {
    ([ $( $element:expr ),* ]) => {
        vec![ $( $element ),* ]
    };
}

// This fails:
// json_bad!([{ "key": "value" }])
//            ^^^^^^^^^^^^^^^^^ Not an expr

// ✅ GOOD: Use tt for nested structures
macro_rules! json_good {
    ([ $( $element:tt ),* ]) => {
        vec![ $( json_good!($element) ),* ]
    };
    ($other:tt) => {
        $other
    };
}
```

### Pitfall 4: Forgetting $crate:: Prefix

```rust
// ❌ BAD: Assumes Json is in scope
macro_rules! json_bad {
    (null) => { Json::Null };
}

// ✅ GOOD: Use absolute path
#[macro_export]
macro_rules! json_good {
    (null) => { $crate::Json::Null };
}
```

The `$crate::` prefix ensures the macro works regardless of where it's invoked, even in other crates.

## Implementation Strategy

### Step 1: Identify Recursive Structure

List all possible input patterns:
- Base cases (terminals that don't contain nested structure)
- Recursive cases (containers that may hold nested structures)

### Step 2: Define Base Cases First

Write rules for terminals:

```rust
macro_rules! my_macro {
    (null) => { /* base case 1 */ };
    ($literal:literal) => { /* base case 2 */ };
}
```

### Step 3: Add Recursive Cases

Write rules that invoke the macro on sub-parts:

```rust
macro_rules! my_macro {
    // ... base cases ...

    ([ $( $item:tt ),* ]) => {
        vec![ $( my_macro!($item) ),* ]
    };
}
```

### Step 4: Order Rules Correctly

Put specific rules before general ones:

```rust
macro_rules! my_macro {
    // 1. Most specific
    (null) => { ... };

    // 2. Recursive cases
    ([ $( $item:tt ),* ]) => { ... };

    // 3. General fallback (last)
    ($other:tt) => { ... };
}
```

### Step 5: Test with Nested Examples

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_nested() {
        let result = my_macro!([
            [1, 2],
            [3, [4, 5]]
        ]);
        // Verify expansion
    }
}
```

## Debugging Recursive Macros

### Use cargo expand

```bash
cargo install cargo-expand
cargo expand --test test_name
```

Shows the fully expanded code, revealing all recursive invocations.

### Add Temporary println! in Expansions

```rust
macro_rules! debug_json {
    ($x:tt) => {{
        println!("Expanding: {}", stringify!($x));
        json!($x)
    }};
}
```

### Trace Expansion Depth

For complex debugging, add depth tracking:

```rust
macro_rules! json_debug {
    (@depth $depth:expr; null) => {{
        println!("{}Base case: null", "  ".repeat($depth));
        Json::Null
    }};

    (@depth $depth:expr; [ $( $element:tt ),* ]) => {{
        println!("{}Array [", "  ".repeat($depth));
        let result = Json::Array(vec![
            $( json_debug!(@depth $depth + 1; $element) ),*
        ]);
        println!("{}]", "  ".repeat($depth));
        result
    }};

    ($x:tt) => {
        json_debug!(@depth 0; $x)
    };
}
```

## Checklist for Recursive Macros

- [ ] Identify all base cases (terminals that don't recurse)
- [ ] Define recursive cases that call the macro on sub-parts
- [ ] Use tt fragment specifier for nested structures
- [ ] Order rules from specific to general
- [ ] Ensure every recursion path eventually reaches a base case
- [ ] Use $crate:: prefix for absolute paths
- [ ] Test with varying nesting depths
- [ ] Use cargo expand to verify expansions
- [ ] Document the recursion structure
- [ ] Consider compile-time performance for deep nesting

## Further Reading

- **The Little Book of Rust Macros - Incremental TT Munchers**: https://veykril.github.io/tlborm/decl-macros/patterns/tt-muncher.html
- **Rust Reference - Macro Invocation**: https://doc.rust-lang.org/reference/macros-by-example.html#macro-invocation
- **json! macro in serde_json**: https://docs.rs/serde_json/latest/serde_json/macro.json.html
