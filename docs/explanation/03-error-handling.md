# Error Handling in Rust

## A Fundamentally Different Approach

Error handling is one of the most striking differences between Python and Rust. Python uses exceptions that can be thrown from anywhere and caught anywhere (or not caught at all, crashing your program). Rust takes a radically different approach: errors are values that must be explicitly handled, and the compiler enforces that you deal with them.

## No Exceptions in Rust

Let's start with what Rust doesn't have: exceptions. There's no `try/catch`, no `raise`, no stack unwinding through multiple function calls. Instead, errors are part of the type system.

### The Python Way

```python
def read_config(path):
    # Might raise FileNotFoundError
    # Might raise PermissionError
    # Might raise ValueError during parsing
    # You don't know without reading docs or the source
    with open(path) as f:
        return json.load(f)

# You might catch errors...
try:
    config = read_config("config.json")
except (FileNotFoundError, PermissionError, ValueError) as e:
    print(f"Error: {e}")
    config = default_config()

# ...or you might not, and your program crashes
```

Problems with this approach:
- **Invisible errors** - Function signatures don't tell you what can go wrong
- **Easy to forget** - Nothing forces you to handle errors
- **Performance cost** - Exception handling has runtime overhead
- **Control flow** - Exceptions make code harder to reason about

### The Rust Way

```rust
fn read_config(path: &str) -> Result<Config, std::io::Error> {
    // The return type tells you this can fail
    let contents = std::fs::read_to_string(path)?;
    let config = parse_config(&contents)?;
    Ok(config)
}

// The compiler forces you to handle the error
match read_config("config.json") {
    Ok(config) => println!("Loaded config"),
    Err(e) => {
        eprintln!("Error: {}", e);
        // Handle the error
    }
}
```

The `Result` type makes errors visible in the function signature. You can't ignore them; the compiler won't let you.

## The Result Type

`Result<T, E>` is an enum with two variants:

```rust
enum Result<T, E> {
    Ok(T),    // Success: contains a value of type T
    Err(E),   // Failure: contains an error of type E
}
```

Every function that can fail returns a `Result`. The type signature tells you:
- What you get on success (`T`)
- What error you get on failure (`E`)

From `/home/user/rust-programming-examples/http-get/src/main.rs`:

```rust
use std::error::Error;
use std::io;

fn http_get_main(url: &str) -> Result<(), Box<dyn Error>> {
    // Send the HTTP request and get a response.
    let mut response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        Err(format!("{}", response.status()))?;
    }

    // Read the response body and write it to stdout.
    let stdout = io::stdout();
    io::copy(&mut response, &mut stdout.lock())?;

    Ok(())
}
```

This function returns `Result<(), Box<dyn Error>>`:
- Success: `Ok(())` (unit type, like Python's `None`)
- Failure: `Err(Box<dyn Error>)` (any error type)

## The ? Operator: Error Propagation

The `?` operator is Rust's secret weapon for clean error handling. It means:
- If the result is `Ok(value)`, extract `value` and continue
- If the result is `Err(e)`, return early with `Err(e)`

Compare these equivalent code snippets:

```rust
// Without ?
fn read_and_parse(path: &str) -> Result<Config, Error> {
    let contents = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return Err(e),
    };

    let config = match parse_config(&contents) {
        Ok(c) => c,
        Err(e) => return Err(e),
    };

    Ok(config)
}

// With ?
fn read_and_parse(path: &str) -> Result<Config, Error> {
    let contents = std::fs::read_to_string(path)?;
    let config = parse_config(&contents)?;
    Ok(config)
}
```

The `?` operator eliminates boilerplate while keeping errors explicit. It's like Python's exceptions, but:
- Only works in functions that return `Result` or `Option`
- Type-checked at compile time
- Visible in the code (explicit `?`)
- Zero runtime overhead

### ? in Action

From the HTTP GET example:

```rust
fn http_get_main(url: &str) -> Result<(), Box<dyn Error>> {
    let mut response = reqwest::blocking::get(url)?;
    // If get() fails, ? returns Err immediately

    if !response.status().is_success() {
        Err(format!("{}", response.status()))?;
        // Manually create an error and propagate it
    }

    io::copy(&mut response, &mut stdout.lock())?;
    // If copy fails, ? returns Err immediately

    Ok(())
}
```

Each `?` is a potential early return. The function short-circuits on the first error.

## The Option Type

`Option<T>` handles the absence of a value:

```rust
enum Option<T> {
    Some(T),  // Value is present
    None,     // Value is absent
}
```

This replaces Python's `None` but with type safety:

```python
# Python
def find_user(id):
    user = database.query(id)
    return user  # Might be None, but type system doesn't know

# Later...
user = find_user(42)
print(user.name)  # Might crash with AttributeError!
```

```rust
// Rust
fn find_user(id: u32) -> Option<User> {
    database.query(id)
    // Returns Some(user) or None
}

// Later...
match find_user(42) {
    Some(user) => println!("{}", user.name),
    None => println!("User not found"),
}
```

The type system forces you to handle the `None` case.

### Option in the Queue

From `/home/user/rust-programming-examples/generic-queue/src/lib.rs`:

```rust
pub fn pop(&mut self) -> Option<T> {
    if self.older.is_empty() {
        use std::mem::swap;

        if self.younger.is_empty() {
            return None;  // Queue is empty
        }

        swap(&mut self.older, &mut self.younger);
        self.older.reverse();
    }

    self.older.pop()  // Vec::pop also returns Option<T>
}
```

The return type `Option<T>` makes it clear that popping can fail (empty queue). Users must handle both cases:

```rust
let mut q = Queue::new();
q.push(42);

match q.pop() {
    Some(value) => println!("Got: {}", value),
    None => println!("Queue was empty"),
}

// Or use unwrap() if you're sure it won't be None
let value = q.pop().unwrap();  // Panics if None!

// Or provide a default
let value = q.pop().unwrap_or(0);
```

### ? Works with Option Too

The `?` operator works with `Option` in functions that return `Option`:

```rust
fn get_first_element<T>(queue: &mut Queue<T>) -> Option<T> {
    let value = queue.pop()?;  // Returns None if queue is empty
    Some(value)
}
```

From the binary tree iterator (`/home/user/rust-programming-examples/binary-tree/src/lib.rs`):

```rust
impl<'a, T> Iterator for TreeIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        let node = self.unvisited.pop()?;  // Return None if empty
        self.push_left_edge(&node.right);
        Some(&node.element)
    }
}
```

The `?` after `pop()` means: if the stack is empty (returns `None`), return `None` from `next()`. Otherwise, continue with the node.

## Error Propagation Patterns

### Pattern 1: Propagate Everything

Use `?` to bubble errors up:

```rust
fn process_data() -> Result<Data, Error> {
    let raw = read_file("data.txt")?;
    let parsed = parse_data(&raw)?;
    let validated = validate_data(parsed)?;
    Ok(validated)
}
```

Python equivalent:

```python
def process_data():
    # Exceptions propagate automatically (invisible)
    raw = read_file("data.txt")
    parsed = parse_data(raw)
    validated = validate_data(parsed)
    return validated
```

Rust makes propagation explicit with `?`.

### Pattern 2: Handle Specific Errors

Use `match` for fine-grained control:

```rust
fn read_config() -> Result<Config, Error> {
    match std::fs::read_to_string("config.json") {
        Ok(contents) => parse_config(&contents),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // File not found - use defaults
            Ok(Config::default())
        }
        Err(e) => {
            // Other errors - propagate
            Err(e.into())
        }
    }
}
```

Python equivalent:

```python
def read_config():
    try:
        with open("config.json") as f:
            return parse_config(f.read())
    except FileNotFoundError:
        return Config.default()
    except Exception as e:
        raise
```

### Pattern 3: Convert Errors

Sometimes you need to convert between error types:

```rust
fn http_get_main(url: &str) -> Result<(), Box<dyn Error>> {
    let mut response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        // Convert a String into an error
        Err(format!("{}", response.status()))?;
    }
    Ok(())
}
```

The `?` operator automatically converts errors if they implement `From`:

```rust
// This works because std::io::Error implements From<reqwest::Error>
fn get_data() -> Result<String, std::io::Error> {
    let response = reqwest::blocking::get("...")?.text()?;
    Ok(response)
}
```

### Pattern 4: Collect Results

Process multiple items, stopping at the first error:

```rust
fn process_all(items: Vec<Item>) -> Result<Vec<Processed>, Error> {
    items.into_iter()
        .map(|item| process_item(item))  // Returns Result<Processed, Error>
        .collect()  // Collect into Result<Vec<Processed>, Error>
}
```

If any `process_item()` fails, the entire operation fails. Python would need a loop with try/catch.

## Custom Error Types

For libraries, you usually define custom error types:

```rust
use std::fmt;

#[derive(Debug)]
enum ConfigError {
    FileNotFound(String),
    ParseError(String),
    ValidationError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::FileNotFound(path) =>
                write!(f, "Config file not found: {}", path),
            ConfigError::ParseError(msg) =>
                write!(f, "Parse error: {}", msg),
            ConfigError::ValidationError(msg) =>
                write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

fn read_config(path: &str) -> Result<Config, ConfigError> {
    // Now errors are specific and typed
    // ...
}
```

Python equivalent:

```python
class ConfigError(Exception):
    pass

class FileNotFoundError(ConfigError):
    pass

class ParseError(ConfigError):
    pass

class ValidationError(ConfigError):
    pass
```

Rust's approach:
- Errors are enums (exhaustive matching)
- Type system tracks which errors are possible
- No need for inheritance hierarchy

## Comparison to Python's try/except

| Aspect | Python try/except | Rust Result/Option |
|--------|-------------------|---------------------|
| Visibility | Hidden in function | Visible in type signature |
| Enforcement | Optional | Compiler-enforced |
| Performance | Runtime overhead | Zero overhead |
| Propagation | Automatic (invisible) | Explicit (with `?`) |
| Type safety | Runtime checks | Compile-time checks |
| Partial failure | Hard to handle | Easy with combinators |

### When Rust Panics

Rust does have a panic mechanism, but it's for **unrecoverable errors**:

```rust
let v = vec![1, 2, 3];
let item = v[10];  // Panics: index out of bounds
```

Panics are like Python's exceptions, but:
- Reserved for programming errors (bugs)
- Should be rare in production code
- Can't be caught in normal code (by design)
- Usually terminate the program

Use `Result` for expected errors (network failures, invalid input). Use panics for impossible states (bugs).

## Real-World Example: HTTP GET

The HTTP GET example shows error handling in action:

```rust
fn http_get_main(url: &str) -> Result<(), Box<dyn Error>> {
    // Multiple operations that can fail
    let mut response = reqwest::blocking::get(url)?;

    // Check for HTTP errors (not automatic!)
    if !response.status().is_success() {
        Err(format!("{}", response.status()))?;
    }

    // Copy response to stdout (can fail)
    let stdout = io::stdout();
    io::copy(&mut response, &mut stdout.lock())?;

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: http-get URL");
        return;
    }

    // Handle the Result at the top level
    if let Err(err) = http_get_main(&args[1]) {
        eprintln!("error: {}", err);
    }
}
```

Notice:
- `http_get_main` uses `?` to propagate errors
- `main` handles the final `Result`
- Errors are typed: `Result<(), Box<dyn Error>>`
- Each `?` is an explicit "this can fail" marker

## Combinators: Functional Error Handling

Rust provides combinators for working with `Result` and `Option`:

```rust
// map: transform the success value
let result: Result<i32, Error> = Ok(5);
let doubled = result.map(|x| x * 2);  // Ok(10)

// map_err: transform the error value
let result: Result<i32, String> = Err("failed".to_string());
let converted = result.map_err(|e| format!("Error: {}", e));

// and_then: chain operations that can fail
let result = read_file("data.txt")
    .and_then(|contents| parse_data(&contents))
    .and_then(|data| validate_data(data));

// or_else: provide fallback on error
let config = read_config("custom.json")
    .or_else(|_| read_config("default.json"));

// unwrap_or: provide default value
let value = queue.pop().unwrap_or(0);

// unwrap_or_else: compute default value lazily
let value = queue.pop().unwrap_or_else(|| expensive_default());
```

Python doesn't have built-in equivalents; you'd use try/except or the `Optional` type from typing.

## Key Takeaways

1. **Errors are values** - Represented by `Result<T, E>`, not exceptions
2. **Compiler-enforced handling** - You can't ignore a `Result`
3. **Visible in signatures** - Function types tell you what can fail
4. **The ? operator** - Concise error propagation with zero overhead
5. **Option for absence** - Type-safe null handling
6. **No hidden control flow** - Errors don't jump through the stack invisibly
7. **Zero-cost abstraction** - Error handling compiles to efficient code
8. **Panic for bugs** - Unrecoverable errors for programmer mistakes

Rust's error handling takes more upfront thought than Python's try/except, but it prevents entire classes of bugs. The type system ensures you handle errors, and the `?` operator keeps the code clean. Once you adjust to this model, you'll find it produces more robust code than exception-based error handling.
