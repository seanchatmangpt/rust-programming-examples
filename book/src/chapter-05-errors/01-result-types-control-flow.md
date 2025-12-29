# Result Types as Control Flow

Error handling in Rust represents a fundamental shift from exception-based systems. Rather than treating errors as exceptional control flow that unwinds the stack, Rust treats errors as **first-class values** that participate in the type system. This architectural decision has profound implications for how we structure programs, define APIs, and reason about failure modes.

## Errors as Domain Concepts

In exception-based languages, errors are often an afterthought—invisible in function signatures, optional to handle, and easy to ignore until production. Rust inverts this relationship by making errors explicit domain concepts that must be designed alongside success cases.

Consider the `grep` tool from our codebase:

```rust
fn grep<R>(target: &str, reader: R) -> io::Result<()>
    where R: BufRead
{
    for line_result in reader.lines() {
        let line = line_result?;
        if line.contains(target) {
            println!("{}", line);
        }
    }
    Ok(())
}
```

The return type `io::Result<()>` makes an architectural statement: **this operation can fail due to I/O errors**. The type signature documents the failure domain—not in comments or external documentation, but in the type system itself. Callers cannot use this function without acknowledging the possibility of I/O failure.

## Result vs Option: Choosing Failure Semantics

Rust provides two primary types for representing potential failure: `Result<T, E>` and `Option<T>`. The choice between them encodes semantic information about *why* an operation might not produce a value.

### Option: Absence is Normal

`Option<T>` represents operations where absence is a normal, expected part of the domain model:

```rust
// Looking up a value that might not exist is normal
fn find_user(id: UserId) -> Option<User>

// A queue might be empty
fn pop(&mut self) -> Option<T>

// Parsing might find no match
fn extract_number(s: &str) -> Option<i32>
```

The absence of a value is not an error—it's a valid state in the problem domain. An empty queue isn't broken; it's just empty.

### Result: Failure Requires Explanation

`Result<T, E>` represents operations where failure needs an explanation—a reason why the operation couldn't complete:

```rust
// File I/O can fail for many reasons (permissions, disk full, etc.)
fn read_config(path: &Path) -> Result<Config, io::Error>

// Network operations have multiple failure modes
fn http_get(url: &str) -> Result<Response, reqwest::Error>

// Parsing can fail with specific error information
fn parse_json(s: &str) -> Result<Value, serde_json::Error>
```

The error type `E` carries information about *what went wrong*. This is architectural metadata that enables error recovery, logging, and user-facing error messages.

### The Architectural Decision

Choosing between `Result` and `Option` is an architectural decision that affects your entire API:

```rust
// Option: "Not found" is a normal outcome
fn get_cache(key: &str) -> Option<String>

// Result: "Not found" might be an error condition worth logging
fn get_required_config(key: &str) -> Result<String, ConfigError>
```

In the first case, cache misses are expected and require no explanation. In the second, a missing required configuration might indicate a deployment error that should be investigated.

## The ? Operator: Propagation as Architecture

The `?` operator is often presented as syntactic sugar for error handling, but it's more accurately understood as an **architectural primitive for composing fallible operations**.

### Propagation Semantics

Consider the `http-get` example:

```rust
fn http_get_main(url: &str) -> Result<(), Box<dyn Error>> {
    let mut response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        Err(format!("{}", response.status()))?;
    }

    let stdout = io::stdout();
    io::copy(&mut response, &mut stdout.lock())?;

    Ok(())
}
```

Each `?` represents a **propagation boundary**—a point where control flow might exit early. This is not hidden exception handling; it's explicit, visible failure propagation. The function's architecture is a pipeline of fallible operations, each of which can short-circuit the entire sequence.

### Type-Directed Propagation

The `?` operator doesn't just propagate errors; it **transforms** them according to type constraints:

```rust
fn process_data() -> Result<Data, CustomError> {
    let text = std::fs::read_to_string("data.txt")?;
    //         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ returns io::Error
    //                                         ? converts to CustomError

    parse_data(&text)?
}
```

This works because `CustomError` implements `From<io::Error>`. The `?` operator automatically invokes the conversion, enabling **error unification across abstraction boundaries**. Lower-level errors (I/O) are converted to higher-level domain errors (CustomError) at the point of propagation.

## Error Propagation as Architectural Design

The pattern of error propagation reveals architectural layers in your system:

```rust
// Low-level: Specific error types
fn read_raw_data(path: &Path) -> Result<Vec<u8>, io::Error> {
    std::fs::read(path)
}

// Mid-level: Domain-specific errors
fn load_config(path: &Path) -> Result<Config, ConfigError> {
    let data = read_raw_data(path)?;  // io::Error -> ConfigError
    parse_config(&data)                // ParseError -> ConfigError
}

// High-level: Unified error type
fn initialize_app() -> Result<App, Box<dyn Error>> {
    let config = load_config("app.toml")?;  // ConfigError -> Box<dyn Error>
    let database = connect_database(&config)?;  // DbError -> Box<dyn Error>
    Ok(App::new(config, database))
}
```

Each layer defines its error domain and provides conversions from lower layers. The `?` operator respects these boundaries, automatically performing conversions as errors propagate upward through the architecture.

## Control Flow Implications

Traditional exception handling creates invisible control flow:

```python
# Python: Hidden failure paths
def process_pipeline():
    data = read_file()      # Might throw
    validated = validate()   # Might throw
    result = transform()     # Might throw
    return result
```

You cannot determine from the code which operations might fail or what the failure modes are. The control flow graph is incomplete.

Rust makes failure explicit:

```rust
fn process_pipeline() -> Result<Data, Error> {
    let data = read_file()?;        // Visible failure point
    let validated = validate(data)?; // Visible failure point
    let result = transform(validated)?; // Visible failure point
    Ok(result)
}
```

Every `?` marks a potential early return. The control flow is **statically visible** in the source code. This enables both human reasoning and compiler optimizations based on the explicit failure paths.

## Result as a Monad: Compositional Architecture

From a functional programming perspective, `Result` is a monad that enables compositional error handling:

```rust
// Sequential composition
let result = fetch_data()
    .and_then(|data| validate(data))
    .and_then(|valid| transform(valid))
    .and_then(|transformed| save(transformed));

// Error recovery
let config = load_custom_config()
    .or_else(|_| load_default_config());

// Mapping over success
let doubled = compute().map(|x| x * 2);

// Mapping over errors
let contextualized = risky_operation()
    .map_err(|e| format!("Operation failed: {}", e));
```

These combinators enable a **declarative style** where error handling is woven into the data flow rather than represented as separate try/catch blocks. The architecture emerges from composition of fallible operations.

## Decision Framework: When to Propagate

Not every error should be propagated. The architectural decision of whether to handle or propagate depends on several factors:

**Propagate when:**
- The current layer cannot meaningfully recover
- The error represents a contract violation that callers must handle
- Adding context would not aid debugging (the error is already specific)
- The operation is a simple transformation in a larger pipeline

**Handle when:**
- You can provide a reasonable fallback or default
- The error is expected and doesn't indicate a problem (like cache miss)
- You can add meaningful context before re-propagating
- The error represents a transient condition you can retry

Example from `grep`:

```rust
fn grep_main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let target = match args.next() {
        Some(s) => s,
        None => Err("usage: grep PATTERN FILE...")?  // Propagate with context
    };

    if files.is_empty() {
        let stdin = io::stdin();
        grep(&target, stdin.lock())?;  // Propagate I/O errors
    } else {
        for file in files {
            let f = File::open(file)?;  // Propagate file errors
            grep(&target, BufReader::new(f))?;
        }
    }

    Ok(())
}
```

The function propagates all errors because `main` is the appropriate place to decide how to present errors to users.

## Conclusion: Errors as Architectural Contracts

By treating errors as values, Rust makes error handling an **architectural concern** rather than a control flow mechanism. The `Result` type creates explicit contracts between layers of your system, documenting failure modes in the type system and ensuring they are handled. The `?` operator provides concise propagation while maintaining type safety and visibility.

This approach forces upfront thinking about failure modes but results in more robust architectures where error handling is designed, not retrofitted. In the next section, we'll explore how custom error types extend this foundation to encode rich domain-specific failure information.
