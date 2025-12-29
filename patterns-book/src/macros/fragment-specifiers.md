# Pattern: Fragment Specifiers for Type-Safe Macros

## Context

You are writing a **DECLARATIVE MACRO** using macro_rules! and need to capture different kinds of Rust syntax elements as macro parameters. Your macro might accept type names, expressions, identifiers, literal values, or complex token sequences. You want the compiler to validate that users pass the correct syntax elements, catching errors at the macro invocation site rather than in the expanded code.

You've learned **MACRO_RULES** basics and possibly **RECURSIVE MACRO** patterns, but need to understand how to constrain what tokens your macro accepts and how those constraints affect what you can do with the captured values.

## Problem

**How do you specify what kinds of syntax elements a macro accepts, ensuring type safety while maintaining flexibility for your macro's purpose?**

Macros operate on token trees before type checking. Without constraints, you might accept invalid syntax and produce confusing error messages:

```rust
// Too permissive - accepts anything
macro_rules! bad_add {
    ($a:tt, $b:tt) => {
        $a + $b  // What if $a is a type name? Or a struct definition?
    };
}

bad_add!(struct Foo {}, 42);  // Compiles the macro, fails later
// Error: expected expression, found keyword `struct`
```

You need to specify what kinds of syntax elements each parameter accepts:
- Expressions (2 + 2, function_call(), variable_name)
- Types (i32, Vec<String>, MyStruct)
- Identifiers (variable or function names)
- Literals (42, "string", true)
- Or flexible token trees for complex syntax

The wrong choice leads to either rejecting valid inputs or accepting invalid ones.

## Forces

- **Type Safety**: More specific fragment specifiers catch errors earlier, at the macro invocation site
- **Flexibility vs Safety Trade-off**: `tt` (token tree) accepts anything; `expr` only accepts expressions
- **Composition Constraints**: Some fragment specifiers can follow others in patterns; some cannot
- **Expansion Context**: What you can do with a captured fragment depends on its specifier
- **User Experience**: Clear specifiers produce better error messages for users
- **Recursive Macros**: `tt` is often required for recursion; other specifiers are too restrictive
- **Pattern Matching**: The compiler matches against fragment specifier syntax rules
- **Future Compatibility**: Fragment specifier rules may change between Rust editions

## Solution

**Choose the most specific fragment specifier that captures the syntax you need, balancing between type safety and flexibility.**

### Available Fragment Specifiers

| Specifier | What It Matches | Use When | Example |
|-----------|----------------|----------|---------|
| `expr` | Expression | Need a computable value | `2 + 2`, `foo()`, `x` |
| `ty` | Type | Need a type name | `i32`, `Vec<String>` |
| `ident` | Identifier | Need a name (variable, function, type) | `my_var`, `MyStruct` |
| `path` | Path | Need a module path or type path | `std::io::Error`, `crate::MyType` |
| `tt` | Single token tree | Maximum flexibility, recursive macros | Anything valid Rust |
| `item` | Item | Need a complete item definition | `fn foo() {}`, `struct Bar {}` |
| `block` | Block expression | Need a braced block | `{ let x = 5; }` |
| `stmt` | Statement | Need a statement | `let x = 5;`, `return x;` |
| `pat` | Pattern | Need a pattern for matching | `Some(x)`, `(a, b)` |
| `literal` | Literal value | Need a literal constant | `42`, `"string"`, `true` |
| `lifetime` | Lifetime | Need lifetime parameter | `'a`, `'static` |
| `meta` | Attribute contents | Need attribute syntax | `derive(Debug)` |
| `vis` | Visibility | Need pub/pub(crate)/etc | `pub`, `pub(super)` |

### Real Example: json! Macro Fragment Choices

From `/home/user/rust-programming-examples/json-macro/src/macros.rs`:

```rust
#[macro_export]
macro_rules! json {
    // Uses literal keyword
    (null) => {
        $crate::Json::Null
    };

    // Uses tt for elements (not expr!)
    ([ $( $element:tt ),* ]) => {
        //     ^^^^^^^^^ tt allows nested objects: { "key": "value" }
        $crate::Json::Array(vec![ $( json!($element) ),* ])
    };

    // Uses tt for keys and values (not ident and expr!)
    ({ $( $key:tt : $value:tt ),* }) => {
        //    ^^^^^^    ^^^^^^^^ tt allows both "string" and CONST keys
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

    // Fallback: accepts any single token tree
    ($other:tt) => {
        $crate::Json::from($other)
    };
}
```

### Why tt Instead of expr?

**Problem**: If we used `expr`:

```rust
// WRONG: Too restrictive
([ $( $element:expr ),* ]) => { ... };
```

This would fail for nested objects:

```rust
json!([
    { "key": "value" }
    //^^^^^^^^^^^^^^^^ NOT an expression - it's a braced token tree
]);
// Error: expected expression, found `{`
```

**Solution**: Use `tt` for nested structures:

```rust
// CORRECT: Flexible enough
([ $( $element:tt ),* ]) => {
    // Each $element can be any valid token tree,
    // including nested objects, arrays, etc.
};
```

### Why tt for Keys Instead of ident?

From the test at line 35-49 in macros.rs:

```rust
#[test]
fn json_with_rust_expressions() {
    const HELLO: &'static str = "hello";
    let macro_generated_value =
        json!({
            "math_works": (4 - 2 == 2),  // String literal as key
            "en": HELLO,                  // String literal as key
            HELLO: "bonjour!"             // Constant as key
        });
    // ...
}
```

**Problem**: If we used `ident`:

```rust
// WRONG: Too restrictive
({ $( $key:ident : $value:tt ),* }) => { ... };
```

This would reject string literal keys:

```rust
json!({
    "key": "value"
    //^^^^ NOT an identifier - it's a string literal
});
// Error: expected identifier, found `"key"`
```

**Solution**: Use `tt` to accept both identifiers and literals:

```rust
// CORRECT: Accepts both
({ $( $key:tt : $value:tt ),* }) => {
    // $key can be "string" or IDENT
    // to_string($key) handles both
};
```

### When to Use Each Specifier

#### Use expr When You Need Expressions

```rust
macro_rules! double {
    ($value:expr) => {
        $value * 2
    };
}

double!(5);              // ✓ Literal expression
double!(x + 3);          // ✓ Binary expression
double!(foo());          // ✓ Call expression
// double!({ x: 10 });   // ✗ Struct literal is NOT an expr context here
```

#### Use ident When You Need Names

```rust
macro_rules! create_setter {
    ($field:ident) => {
        fn $field(&mut self, value: i32) {
            self.$field = value;
        }
    };
}

create_setter!(age);      // ✓ Identifier
// create_setter!(42);    // ✗ Not an identifier
// create_setter!("age"); // ✗ Not an identifier
```

#### Use ty When You Need Types

```rust
macro_rules! create_vec {
    ($type:ty) => {
        Vec::<$type>::new()
    };
}

let v = create_vec!(i32);           // ✓ Simple type
let v = create_vec!(Vec<String>);   // ✓ Generic type
// let v = create_vec!(42);         // ✗ Not a type
```

#### Use tt When You Need Flexibility

```rust
macro_rules! flexible {
    ($($anything:tt)*) => {
        // Can match any valid Rust token sequence
    };
}

flexible!(fn foo() {});              // ✓ Function definition
flexible!({ x: 10, y: 20 });         // ✓ Struct literal
flexible!(if x { y } else { z });    // ✓ If expression
flexible!(literally anything);        // ✓ Identifiers
```

## Resulting Context

### Benefits

**Early Error Detection**: With specific specifiers, the compiler catches errors at the macro invocation:

```rust
macro_rules! add {
    ($a:expr, $b:expr) => { $a + $b };
}

// Error caught HERE, at the invocation
add!(struct Foo {}, 42);
// Error: expected expression, found keyword `struct`
```

Not later in the expansion:

```rust
macro_rules! add_loose {
    ($a:tt, $b:tt) => { $a + $b };
}

// Macro accepts it...
add_loose!(struct Foo {}, 42);
// Error shows up in expanded code (confusing!)
```

**Better Documentation**: Fragment specifiers document what the macro expects:

```rust
macro_rules! documented {
    ($name:ident, $value:expr, $type:ty) => {
        // Clear from signature:
        // - name is an identifier
        // - value is an expression
        // - type is a type
    };
}
```

**Type-Safe Expansion**: Captured fragments maintain type information:

```rust
macro_rules! assert_type {
    ($value:expr, $type:ty) => {
        let _: $type = $value;  // Compiler checks types
    };
}

assert_type!(42, i32);        // ✓ OK
// assert_type!(42, String);  // ✗ Compile error
```

### Real-World Usage from json-macro

From the test at line 111-125:

```rust
#[test]
fn json_monolith() {
    let width = 4.0;
    let desc =
        json!({
            "width": width,          // expr captured and evaluated
            "height": (width * 9.0 / 4.0)  // complex expr
        });
    // ...
}
```

The `tt` specifier allows both:
- Simple identifiers: `width`
- Complex expressions: `(width * 9.0 / 4.0)`

Because the macro converts to `Json::from($value)`, and `From` trait handles various types.

### New Challenges

**Fragment Following Restrictions**: Some specifiers cannot follow others in patterns:

```rust
// ✗ INVALID: expr cannot follow tt without separator
macro_rules! bad {
    ($first:tt $second:expr) => { };
}

// ✓ VALID: Separator between them
macro_rules! good {
    ($first:tt, $second:expr) => { };
}

// ✓ VALID: tt can follow anything
macro_rules! also_good {
    ($first:expr $second:tt) => { };
}
```

**Over-Restriction**: Too specific specifiers reject valid inputs:

```rust
macro_rules! too_specific {
    ($value:literal) => { $value };
}

too_specific!(42);        // ✓ OK
// too_specific!(x);      // ✗ Rejects variables
// too_specific!(2 + 2);  // ✗ Rejects expressions
```

**Under-Restriction**: Too general specifiers allow invalid inputs:

```rust
macro_rules! too_general {
    ($value:tt) => {
        let x: i32 = $value;  // Assumes $value is i32
    };
}

too_general!(42);              // ✓ OK
too_general!("string");        // ✗ Compiles macro, fails later
```

### When You Have This Pattern

You can now:
- Design type-safe macro APIs with appropriate constraints
- Balance flexibility and safety in **RECURSIVE MACRO** patterns
- Provide better error messages for macro users
- Build **DSL CONSTRUCTION** with clear syntax rules

## Related Patterns

- **MACRO_RULES**: Foundation; fragment specifiers are part of macro rules
- **RECURSIVE MACRO**: Often requires `tt` for maximum flexibility
- **DSL CONSTRUCTION**: Fragment specifiers define the DSL's syntax
- **MACRO HYGIENE**: Applies to all captured fragments regardless of specifier

## Known Uses

### Standard Library: vec!

```rust
// Accepts expressions for each element
macro_rules! vec {
    ($($element:expr),*) => {
        // ^^^^^^^^^^^^ expr allows any expression
        {
            let mut temp_vec = Vec::new();
            $(temp_vec.push($element);)*
            temp_vec
        }
    };
}

vec![1, 2, 3];           // Literals
vec![x, y, z];           // Variables
vec![foo(), bar()];      // Calls
```

### println!

```rust
// Simplified version
macro_rules! println {
    ($fmt:expr) => { ... };
    //  ^^^^^^^^^ Format string must be expression
    ($fmt:expr, $($arg:expr),*) => { ... };
    //              ^^^^^^^^^ Arguments are expressions
}

println!("Value: {}", x + 5);
//                    ^^^^^ Expression
```

### serde_json::json!

Uses `tt` for maximum flexibility, like our example:

```rust
json!({
    "flexible": true,
    flexible_key: "also works",
    "value": complex_expression()
})
```

### diesel::table!

Uses specific specifiers for table definitions:

```rust
table! {
    users (id) {
        id -> Integer,
        //    ^^^^^^^ Type
        name -> Text,
    }
}
```

### lazy_static!

Uses `item` for static item definitions:

```rust
lazy_static! {
    static ref CONFIG: Config = load_config();
    //         ^^^^^^ ident
    //                ^^^^^^ ty
    //                         ^^^^^^^^^^^^^ expr
}
```

## Examples

### Example 1: Choosing Between expr and tt

```rust
// Version 1: expr (type-safe but restrictive)
macro_rules! double_expr {
    ($value:expr) => { $value * 2 };
}

double_expr!(5);              // ✓ Works
double_expr!(x + 3);          // ✓ Works
// double_expr!({ x: 10 });   // ✗ Struct literal not allowed

// Version 2: tt (flexible but less safe)
macro_rules! double_tt {
    ($value:tt) => { $value * 2 };
}

double_tt!(5);                 // ✓ Works
double_tt!(x + 3);             // ✓ Works
double_tt!({ x: 10 });         // ✓ Compiles, fails at runtime
//                                (struct literal can't be multiplied)
```

**Guideline**: Use `expr` unless you need to accept non-expression syntax.

### Example 2: Type-Safe Generic Function Generator

```rust
macro_rules! generate_getter {
    ($field:ident, $type:ty) => {
        pub fn $field(&self) -> $type {
            //  ^^^^^^ ident ensures valid function name
            //                  ^^^^^^ ty ensures valid return type
            self.$field
        }
    };
}

struct Person {
    name: String,
    age: u32,
}

impl Person {
    generate_getter!(name, String);
    generate_getter!(age, u32);
    // generate_getter!("name", String);  // ✗ Error: expected ident
    // generate_getter!(name, 42);        // ✗ Error: expected type
}
```

### Example 3: Flexible Configuration Macro

```rust
macro_rules! config {
    // Accepts both identifier and string literal keys
    ($($key:tt => $value:expr),*) => {{
        //  ^^^^ tt for flexibility
        //           ^^^^^^^^^ expr for values
        let mut map = HashMap::new();
        $(
            map.insert(stringify!($key).to_string(), $value);
        )*
        map
    }};
}

let conf = config!(
    database => "localhost",
    "port" => 5432,           // String literal key
    max_connections => 100    // Identifier key
);
```

### Example 4: Matching Multiple Fragment Types

```rust
macro_rules! describe {
    ($name:ident) => {
        concat!("Identifier: ", stringify!($name))
    };
    ($value:literal) => {
        concat!("Literal: ", stringify!($value))
    };
    ($expr:expr) => {
        concat!("Expression: ", stringify!($expr))
    };
}

describe!(foo);        // "Identifier: foo"
describe!(42);         // "Literal: 42"
describe!(1 + 1);      // "Expression: 1 + 1"
```

## Fragment Specifier Decision Tree

```
Do you need a complete item (fn, struct, etc.)?
├─ Yes → Use `item`
└─ No ↓

Do you need a type name?
├─ Yes → Use `ty`
└─ No ↓

Do you need just an identifier?
├─ Yes → Use `ident`
└─ No ↓

Do you need a value to compute?
├─ Yes → Use `expr`
└─ No ↓

Do you need maximum flexibility (nested structures, DSL)?
├─ Yes → Use `tt`
└─ No → Consider `block`, `stmt`, `pat`, `literal`, etc.
```

## Common Pitfalls and Solutions

### Pitfall 1: Using ident for Values

```rust
// ❌ BAD: Only accepts identifier
macro_rules! print_value {
    ($val:ident) => {
        println!("{}", $val);
    };
}

print_value!(x);      // ✓ OK
// print_value!(42);  // ✗ Error: expected ident

// ✅ GOOD: Accepts any expression
macro_rules! print_value {
    ($val:expr) => {
        println!("{}", $val);
    };
}

print_value!(x);      // ✓ OK
print_value!(42);     // ✓ OK
print_value!(x + 5);  // ✓ OK
```

### Pitfall 2: Using expr for Types

```rust
// ❌ BAD: expr doesn't match types
macro_rules! new_vec {
    ($type:expr) => {
        Vec::<$type>::new()
    };
}

// new_vec!(i32);  // ✗ Error: expected expression

// ✅ GOOD: Use ty for types
macro_rules! new_vec {
    ($type:ty) => {
        Vec::<$type>::new()
    };
}

new_vec!(i32);  // ✓ OK
```

### Pitfall 3: Forgetting Fragment Following Rules

```rust
// ❌ BAD: expr cannot follow tt without separator
macro_rules! bad_sequence {
    ($first:tt $second:expr) => { };
}

// ✅ GOOD: Add separator or use tt for both
macro_rules! good_sequence {
    ($first:tt, $second:expr) => { };
}
```

### Pitfall 4: Over-Using tt

```rust
// ❌ BAD: Loses type safety
macro_rules! add_loose {
    ($a:tt, $b:tt) => {
        $a + $b  // What if $a isn't addable?
    };
}

// ✅ GOOD: Use expr for type checking
macro_rules! add_safe {
    ($a:expr, $b:expr) => {
        $a + $b  // Compiler ensures valid
    };
}
```

## Implementation Checklist

- [ ] Identify what syntax elements the macro needs
- [ ] Choose the most specific appropriate specifier for each parameter
- [ ] Consider whether recursive macros need `tt` flexibility
- [ ] Test with various input types (literals, variables, expressions)
- [ ] Verify error messages are clear when wrong types are passed
- [ ] Check fragment following rules if parameters aren't separated
- [ ] Document expected parameter types in comments
- [ ] Test edge cases (nested structures, complex expressions)

## Comparison Table: When to Use Each Specifier

| Need | Use | Don't Use | Why |
|------|-----|-----------|-----|
| Variable name | `ident` | `expr`, `tt` | Too general, allows non-names |
| Computed value | `expr` | `tt` | Loses type checking |
| Type name | `ty` | `ident`, `expr` | Wrong syntax category |
| Nested structures | `tt` | `expr` | expr doesn't match braces |
| Function body | `block` | `expr`, `tt` | More specific than tt, more flexible than expr |
| Match pattern | `pat` | `expr`, `ident` | Patterns have special syntax |
| Visibility modifier | `vis` | `ident`, `tt` | Specific syntax rules |
| Maximum flexibility | `tt` | Anything else | When you need to accept anything |

## Further Reading

- **Rust Reference - Fragment Specifiers**: https://doc.rust-lang.org/reference/macros-by-example.html#metavariables
- **The Little Book of Rust Macros - Fragment Specifiers**: https://veykril.github.io/tlborm/decl-macros/minutiae/fragment-specifiers.html
- **Fragment Following Rules**: https://doc.rust-lang.org/reference/macros-by-example.html#follow-set-ambiguity-restrictions
- **RFC 1584 - Macro Fragment Specifiers**: https://rust-lang.github.io/rfcs/1584-macros.html
