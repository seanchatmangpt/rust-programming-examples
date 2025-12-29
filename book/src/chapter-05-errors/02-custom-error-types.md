# Custom Error Types

While `Result<T, E>` provides the foundation for error handling, the error type `E` is where architectural decisions about failure representation materialize. Custom error types transform raw failure signals into rich domain information that can drive recovery logic, user feedback, and debugging workflows.

## Errors as Domain State Machines

An error type is fundamentally a **state machine over failure modes**. Each variant represents a distinct failure state, and the transitions between states are encoded in error conversion logic.

Consider a configuration loading system:

```rust
#[derive(Debug)]
pub enum ConfigError {
    FileNotFound(PathBuf),
    PermissionDenied(PathBuf),
    InvalidFormat { line: usize, column: usize, expected: String },
    MissingRequiredField(String),
    ValidationFailed(Vec<String>),
}
```

This isn't just an error type; it's a **taxonomy of failure** for configuration loading. Each variant encodes:
1. **What went wrong** (the variant itself)
2. **Why it went wrong** (the associated data)
3. **How to respond** (implicit in the structure)

The type system ensures exhaustive handling:

```rust
fn handle_config_error(err: ConfigError) -> String {
    match err {
        ConfigError::FileNotFound(path) =>
            format!("Config file not found: {}", path.display()),
        ConfigError::PermissionDenied(path) =>
            format!("Permission denied reading: {}", path.display()),
        ConfigError::InvalidFormat { line, column, expected } =>
            format!("Parse error at {}:{}, expected {}", line, column, expected),
        ConfigError::MissingRequiredField(field) =>
            format!("Missing required field: {}", field),
        ConfigError::ValidationFailed(errors) =>
            format!("Validation failed:\n{}", errors.join("\n")),
    }
}
```

If you add a new variant, the compiler forces you to handle it everywhere this error type is matched. This is **exhaustiveness checking as architectural safety**.

## Display vs Debug: Dual Representations

Rust's error types typically implement two distinct representations:

```rust
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::FileNotFound(path) =>
                write!(f, "configuration file not found: {}", path.display()),
            ConfigError::InvalidFormat { line, column, .. } =>
                write!(f, "invalid configuration format at line {}, column {}", line, column),
            // User-facing messages
            _ => write!(f, "configuration error"),
        }
    }
}

impl fmt::Debug for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::FileNotFound(path) =>
                f.debug_struct("FileNotFound")
                    .field("path", path)
                    .finish(),
            ConfigError::InvalidFormat { line, column, expected } =>
                f.debug_struct("InvalidFormat")
                    .field("line", line)
                    .field("column", column)
                    .field("expected", expected)
                    .finish(),
            // Developer-facing diagnostic information
            _ => f.debug_tuple("ConfigError").field(self).finish(),
        }
    }
}
```

**Display**: User-facing error messages for production.
**Debug**: Developer-facing diagnostic information for debugging.

This dual representation enables:
- **Production**: Show users `Display` output (clean, helpful messages)
- **Development**: Show developers `Debug` output (full diagnostic info)
- **Logging**: Use `Debug` in logs, `Display` in user notifications

## Error Conversions: Architectural Boundaries

The `From` trait defines **conversion boundaries** between error domains:

```rust
impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> ConfigError {
        match err.kind() {
            io::ErrorKind::NotFound =>
                ConfigError::FileNotFound(PathBuf::from("unknown")),
            io::ErrorKind::PermissionDenied =>
                ConfigError::PermissionDenied(PathBuf::from("unknown")),
            _ => panic!("Unexpected I/O error: {}", err),
        }
    }
}
```

This conversion defines how **lower-level I/O errors are interpreted** in the configuration domain. The `?` operator uses this conversion automatically:

```rust
fn load_config(path: &Path) -> Result<Config, ConfigError> {
    let contents = std::fs::read_to_string(path)?;
    //             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Returns io::Error
    //                                            ? Converts to ConfigError via From

    parse_config(&contents)
}
```

### Contextual Conversion

Sometimes you need to preserve context during conversion:

```rust
impl ConfigError {
    pub fn with_path(self, path: PathBuf) -> Self {
        match self {
            ConfigError::FileNotFound(_) => ConfigError::FileNotFound(path),
            ConfigError::PermissionDenied(_) => ConfigError::PermissionDenied(path),
            other => other,
        }
    }
}

fn load_config(path: &Path) -> Result<Config, ConfigError> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| ConfigError::from(e).with_path(path.to_owned()))?;
    parse_config(&contents)
}
```

This pattern **enriches errors with context** as they propagate through layers.

## Error Hierarchies Without Inheritance

Unlike object-oriented languages, Rust doesn't use inheritance for error hierarchies. Instead, it uses **enum composition**:

```rust
#[derive(Debug)]
pub enum ApplicationError {
    Config(ConfigError),
    Database(DatabaseError),
    Network(NetworkError),
    Authentication(AuthError),
}

impl From<ConfigError> for ApplicationError {
    fn from(err: ConfigError) -> Self {
        ApplicationError::Config(err)
    }
}

impl From<DatabaseError> for ApplicationError {
    fn from(err: DatabaseError) -> Self {
        ApplicationError::Database(err)
    }
}
```

This creates a **compositional error hierarchy** where higher-level errors aggregate lower-level errors. The type system ensures you handle each category:

```rust
fn handle_app_error(err: ApplicationError) {
    match err {
        ApplicationError::Config(config_err) => {
            // Handle configuration errors
            log::error!("Configuration error: {:?}", config_err);
        }
        ApplicationError::Database(db_err) => {
            // Handle database errors differently
            log::error!("Database error: {:?}", db_err);
            alert_ops_team(db_err);
        }
        ApplicationError::Network(net_err) => {
            // Maybe retry network errors
            retry_with_backoff(net_err);
        }
        ApplicationError::Authentication(auth_err) => {
            // Security-sensitive logging
            security_log::error("Auth failure: {:?}", auth_err);
        }
    }
}
```

## Error Context Chains

The `std::error::Error` trait supports **error chains** for preserving causality:

```rust
impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::InvalidFormat { .. } => None,
            _ => None,
        }
    }
}
```

For errors with underlying causes, you can chain them:

```rust
#[derive(Debug)]
pub struct ParseError {
    message: String,
    source: Option<Box<dyn std::error::Error + 'static>>,
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref())
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parse error: {}", self.message)?;
        if let Some(source) = &self.source {
            write!(f, "\nCaused by: {}", source)?;
        }
        Ok(())
    }
}
```

This enables **root cause analysis** by following the error chain to the original failure.

## Type Erasure with Box&lt;dyn Error&gt;

When exact error types aren't architecturally significant, **type erasure** simplifies APIs:

```rust
fn http_get_main(url: &str) -> Result<(), Box<dyn Error>> {
    let mut response = reqwest::blocking::get(url)?;
    //                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ reqwest::Error
    //                                             ? converts to Box<dyn Error>

    if !response.status().is_success() {
        Err(format!("{}", response.status()))?;
        //  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ String -> Box<dyn Error>
    }

    io::copy(&mut response, &mut io::stdout().lock())?;
    //                                                  ^ io::Error -> Box<dyn Error>

    Ok(())
}
```

`Box<dyn Error>` accepts **any error type** that implements `std::error::Error`. This is useful for:
- **Main functions** where you just want to report errors
- **Prototyping** before defining custom error types
- **Utility functions** where callers don't need to discriminate error types

### When Not to Use Type Erasure

Avoid `Box<dyn Error>` in:
- **Library APIs** where callers need to handle specific errors
- **Recovery logic** where you need to match on error variants
- **Performance-critical paths** (boxing has allocation overhead)

## Practical Example: Building a Custom Error Type

Let's build a comprehensive error type for a hypothetical API client:

```rust
use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum ApiError {
    /// Network connectivity error
    Network {
        url: String,
        source: reqwest::Error
    },

    /// HTTP error status
    HttpStatus {
        code: u16,
        message: String
    },

    /// Response body couldn't be parsed
    ParseError {
        body: String,
        source: serde_json::Error
    },

    /// API returned error response
    ApiFailure {
        code: String,
        message: String
    },

    /// Rate limit exceeded
    RateLimited {
        retry_after: Option<u64>
    },
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::Network { url, .. } =>
                write!(f, "Network error accessing {}", url),
            ApiError::HttpStatus { code, message } =>
                write!(f, "HTTP {}: {}", code, message),
            ApiError::ParseError { .. } =>
                write!(f, "Failed to parse API response"),
            ApiError::ApiFailure { code, message } =>
                write!(f, "API error {}: {}", code, message),
            ApiError::RateLimited { retry_after } => {
                if let Some(seconds) = retry_after {
                    write!(f, "Rate limited, retry after {} seconds", seconds)
                } else {
                    write!(f, "Rate limited")
                }
            }
        }
    }
}

impl Error for ApiError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ApiError::Network { source, .. } => Some(source),
            ApiError::ParseError { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::Network {
            url: err.url().map(|u| u.to_string()).unwrap_or_default(),
            source: err
        }
    }
}
```

This error type:
1. **Enumerates failure modes** comprehensively
2. **Preserves context** (URLs, status codes, response bodies)
3. **Chains underlying errors** via `source()`
4. **Provides conversions** from lower-level errors
5. **Implements Display** for user-facing messages
6. **Enables recovery** (e.g., extracting `retry_after` for rate limiting)

## Architectural Principles for Custom Errors

### Principle 1: Encode Domain Semantics

Your error type should reflect **domain failure modes**, not implementation details:

```rust
// ❌ Bad: Leaks implementation
pub enum BadError {
    VecIndexOutOfBounds,
    HashMapKeyMissing,
}

// ✅ Good: Domain semantics
pub enum GoodError {
    UserNotFound(UserId),
    InvalidPermissions { user: UserId, resource: ResourceId },
}
```

### Principle 2: Preserve Information for Recovery

Include data needed for **error recovery**:

```rust
pub enum PaymentError {
    InsufficientFunds {
        required: Decimal,
        available: Decimal
    },
    CardDeclined {
        reason: String,
        retry_allowed: bool
    },
}

fn handle_payment_error(err: PaymentError) -> RetryStrategy {
    match err {
        PaymentError::CardDeclined { retry_allowed: true, .. } =>
            RetryStrategy::RetryWithDifferentCard,
        PaymentError::InsufficientFunds { .. } =>
            RetryStrategy::RequestFundsFromUser,
        _ =>
            RetryStrategy::Abort,
    }
}
```

### Principle 3: Make Errors Actionable

Each variant should suggest a **recovery path** or action:

```rust
pub enum DatabaseError {
    ConnectionLost,        // → Reconnect
    QueryTimeout,          // → Retry with timeout
    ConstraintViolation,   // → Validate input
    Deadlock,              // → Retry transaction
}
```

## Conclusion

Custom error types are architectural tools for encoding failure semantics. By carefully designing error enums, implementing thoughtful Display/Debug representations, and providing conversions at layer boundaries, you create error handling architectures that are:

- **Type-safe**: Compiler-verified exhaustiveness
- **Informative**: Rich diagnostic information
- **Composable**: Errors convert across layers
- **Recoverable**: Context preserved for recovery logic

In the next section, we'll explore **error handling patterns** that leverage these custom types to build robust systems.
