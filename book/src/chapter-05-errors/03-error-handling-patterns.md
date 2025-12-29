# Error Handling Patterns

Error handling is not just about propagating failures—it's about making **architectural decisions** about when to recover, when to propagate, and how to transform errors as they move through your system. This section explores battle-tested patterns for structuring error handling in real applications.

## Pattern 1: The Builder Pattern with Result

The builder pattern traditionally accumulates configuration, but when combined with `Result`, it can **accumulate validation failures**:

```rust
#[derive(Debug)]
pub struct ServerConfig {
    host: String,
    port: u16,
    max_connections: usize,
}

pub struct ServerConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    max_connections: Option<usize>,
}

impl ServerConfigBuilder {
    pub fn new() -> Self {
        Self {
            host: None,
            port: None,
            max_connections: None,
        }
    }

    pub fn host(mut self, host: String) -> Self {
        self.host = Some(host);
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn max_connections(mut self, max: usize) -> Self {
        self.max_connections = Some(max);
        self
    }

    pub fn build(self) -> Result<ServerConfig, String> {
        Ok(ServerConfig {
            host: self.host.ok_or("Host is required")?,
            port: self.port.ok_or("Port is required")?,
            max_connections: self.max_connections.unwrap_or(100),
        })
    }
}
```

Usage:

```rust
let config = ServerConfigBuilder::new()
    .host("localhost".to_string())
    .port(8080)
    .build()?;  // Fails if required fields missing
```

This pattern **defers validation** until `build()`, allowing partial configuration while ensuring complete configuration before use.

### Enhanced: Accumulating Multiple Errors

For better UX, collect **all validation errors** rather than failing on the first:

```rust
pub fn build(self) -> Result<ServerConfig, Vec<String>> {
    let mut errors = Vec::new();

    let host = match self.host {
        Some(h) => h,
        None => {
            errors.push("Host is required".to_string());
            String::new()  // Placeholder
        }
    };

    let port = match self.port {
        Some(p) => p,
        None => {
            errors.push("Port is required".to_string());
            0  // Placeholder
        }
    };

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(ServerConfig {
        host,
        port,
        max_connections: self.max_connections.unwrap_or(100),
    })
}
```

Now users see **all validation failures** at once instead of fixing them one at a time.

## Pattern 2: Recover vs Propagate Decision Trees

Not every error should be propagated. The decision to handle or propagate depends on architectural context:

```rust
fn load_configuration() -> Result<Config, ConfigError> {
    // Try custom config first
    match load_config_from_file("custom.toml") {
        Ok(config) => return Ok(config),

        // File not found is expected - try default
        Err(ConfigError::FileNotFound(_)) => {
            match load_config_from_file("default.toml") {
                Ok(config) => return Ok(config),

                // Still not found - use hardcoded defaults
                Err(ConfigError::FileNotFound(_)) => {
                    return Ok(Config::default());
                }

                // Other errors are unexpected - propagate
                Err(e) => return Err(e),
            }
        }

        // Other errors during custom load - propagate
        Err(e) => return Err(e),
    }
}
```

This creates a **recovery cascade**:
1. Try primary source
2. Fall back to secondary source
3. Fall back to hardcoded defaults
4. Propagate only truly unexpected errors

### Decision Framework

**Recover when:**
- You have a reasonable fallback value
- The error represents expected absence (not failure)
- Continuing with a default is safe
- The error is transient and you can retry

**Propagate when:**
- No reasonable fallback exists
- The error indicates data corruption or invariant violation
- Callers need to make recovery decisions
- The error requires user intervention

## Pattern 3: Exhaustiveness Checking with Match

Rust's exhaustiveness checking ensures you handle **all error variants**:

```rust
fn handle_database_error(err: DatabaseError) -> RecoveryStrategy {
    match err {
        DatabaseError::ConnectionLost => RecoveryStrategy::Reconnect,
        DatabaseError::QueryTimeout => RecoveryStrategy::RetryWithBackoff,
        DatabaseError::ConstraintViolation => RecoveryStrategy::Abort,
        DatabaseError::Deadlock => RecoveryStrategy::RetryTransaction,
        // If you add a new variant, this won't compile until you handle it
    }
}
```

This is **architectural safety**: the type system ensures all failure modes are considered.

### Using Match for Context Preservation

Extract information while handling errors:

```rust
fn process_request() -> Result<Response, String> {
    match authenticate_user() {
        Ok(user) => {
            match authorize_action(&user) {
                Ok(()) => perform_action(&user),
                Err(AuthError::InsufficientPermissions { required, actual }) => {
                    Err(format!(
                        "User lacks permissions. Required: {:?}, Has: {:?}",
                        required, actual
                    ))
                }
                Err(AuthError::Expired) => {
                    Err("Session expired. Please log in again.".to_string())
                }
            }
        }
        Err(AuthError::InvalidCredentials) => {
            Err("Invalid username or password".to_string())
        }
        Err(e) => Err(format!("Authentication error: {}", e)),
    }
}
```

Each match arm **adds context specific to that error variant**.

## Pattern 4: The map_err Combinator for Context

When propagating errors, add context with `map_err`:

```rust
fn load_user_profile(user_id: UserId) -> Result<Profile, AppError> {
    let data = fetch_from_database(user_id)
        .map_err(|e| AppError::Database {
            operation: "fetch_user_profile".to_string(),
            user_id,
            source: e,
        })?;

    let profile = parse_profile_data(&data)
        .map_err(|e| AppError::ParseError {
            data_type: "UserProfile".to_string(),
            user_id,
            source: e,
        })?;

    Ok(profile)
}
```

Each `map_err` **enriches the error** with operation-specific context before propagation.

## Pattern 5: Early Return with Guard Clauses

Use guard clauses to validate preconditions and **fail fast**:

```rust
fn process_payment(payment: Payment) -> Result<Receipt, PaymentError> {
    // Guard: Validate amount
    if payment.amount <= 0 {
        return Err(PaymentError::InvalidAmount(payment.amount));
    }

    // Guard: Check account status
    if !payment.account.is_active() {
        return Err(PaymentError::InactiveAccount(payment.account.id));
    }

    // Guard: Verify balance
    if payment.account.balance < payment.amount {
        return Err(PaymentError::InsufficientFunds {
            required: payment.amount,
            available: payment.account.balance,
        });
    }

    // All guards passed - proceed with payment
    execute_payment(payment)
}
```

This **front-loads validation**, making the happy path code cleaner and easier to reason about.

## Pattern 6: Context Preservation in Iterators

When processing collections, preserve context about **which item failed**:

```rust
fn validate_all_users(users: Vec<User>) -> Result<(), ValidationError> {
    for (index, user) in users.iter().enumerate() {
        validate_user(user).map_err(|e| ValidationError::UserValidationFailed {
            user_index: index,
            user_id: user.id,
            source: e,
        })?;
    }
    Ok(())
}
```

The error now includes **which user** failed validation, not just that validation failed.

### Collecting All Failures

Sometimes you want to **continue processing** and collect all errors:

```rust
fn validate_all_users_collect(users: Vec<User>) -> Result<(), Vec<ValidationError>> {
    let errors: Vec<ValidationError> = users
        .iter()
        .enumerate()
        .filter_map(|(index, user)| {
            validate_user(user).err().map(|e| ValidationError::UserValidationFailed {
                user_index: index,
                user_id: user.id,
                source: e,
            })
        })
        .collect();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

This is more user-friendly: **all errors are reported at once**.

## Pattern 7: The or_else Combinator for Fallback Chains

Create fallback chains with `or_else`:

```rust
fn get_configuration() -> Result<Config, ConfigError> {
    load_from_environment()
        .or_else(|_| load_from_file("config.toml"))
        .or_else(|_| load_from_file("/etc/myapp/config.toml"))
        .or_else(|_| Ok(Config::default()))
}
```

Each fallback is tried in sequence until one succeeds or all fail.

## Pattern 8: Transactional Error Handling

For operations that must **atomically succeed or fail**:

```rust
fn transfer_funds(from: AccountId, to: AccountId, amount: Decimal) -> Result<(), TransferError> {
    let mut transaction = database.begin_transaction()?;

    // All operations within transaction
    let result = (|| -> Result<(), TransferError> {
        transaction.debit_account(from, amount)?;
        transaction.credit_account(to, amount)?;
        transaction.record_transfer(from, to, amount)?;
        Ok(())
    })();

    match result {
        Ok(()) => {
            transaction.commit()?;
            Ok(())
        }
        Err(e) => {
            transaction.rollback()?;
            Err(e)
        }
    }
}
```

The closure captures all fallible operations. If any fails, the transaction rolls back.

## Pattern 9: Retry with Exponential Backoff

For transient errors, implement **retry logic**:

```rust
fn fetch_with_retry(url: &str, max_retries: u32) -> Result<Response, NetworkError> {
    let mut attempt = 0;
    let mut delay = Duration::from_millis(100);

    loop {
        match fetch_data(url) {
            Ok(response) => return Ok(response),
            Err(e) if is_transient(&e) && attempt < max_retries => {
                attempt += 1;
                eprintln!("Attempt {} failed, retrying after {:?}", attempt, delay);
                std::thread::sleep(delay);
                delay *= 2;  // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
}

fn is_transient(err: &NetworkError) -> bool {
    matches!(err,
        NetworkError::Timeout |
        NetworkError::ConnectionReset |
        NetworkError::ServiceUnavailable
    )
}
```

This pattern **distinguishes transient from permanent errors** and retries appropriately.

## Pattern 10: Error Wrapping for API Boundaries

At API boundaries, wrap internal errors to **hide implementation details**:

```rust
// Internal error type (private)
#[derive(Debug)]
enum InternalError {
    Database(sqlx::Error),
    Cache(redis::Error),
    Serialization(serde_json::Error),
}

// Public API error type
#[derive(Debug)]
pub enum ApiError {
    NotFound,
    InvalidInput(String),
    ServerError,
}

impl From<InternalError> for ApiError {
    fn from(err: InternalError) -> Self {
        match err {
            InternalError::Database(e) => {
                log::error!("Database error: {:?}", e);
                ApiError::ServerError
            }
            InternalError::Cache(_) => ApiError::ServerError,
            InternalError::Serialization(_) => ApiError::InvalidInput(
                "Invalid data format".to_string()
            ),
        }
    }
}
```

External callers see **clean API errors** while internal systems log detailed diagnostics.

## Real-World Example: grep Error Handling

Let's analyze error handling in the `grep` project:

```rust
fn grep<R>(target: &str, reader: R) -> io::Result<()>
    where R: BufRead
{
    for line_result in reader.lines() {
        let line = line_result?;  // Propagate I/O errors
        if line.contains(target) {
            println!("{}", line);
        }
    }
    Ok(())
}

fn grep_main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let target = match args.next() {
        Some(s) => s,
        None => Err("usage: grep PATTERN FILE...")?  // Convert to error
    };
    let files: Vec<PathBuf> = args.map(PathBuf::from).collect();

    if files.is_empty() {
        let stdin = io::stdin();
        grep(&target, stdin.lock())?;  // Propagate I/O errors
    } else {
        for file in files {
            let f = File::open(file)?;  // Propagate file open errors
            grep(&target, BufReader::new(f))?;
        }
    }

    Ok(())
}

fn main() {
    let result = grep_main();
    if let Err(err) = result {
        eprintln!("{}", err);  // User-facing error message
        std::process::exit(1);
    }
}
```

**Architectural decisions:**
1. `grep()` uses specific `io::Result<()>` since it only performs I/O
2. `grep_main()` uses `Box<dyn Error>` to unify different error types
3. All errors propagate to `main()` for user-facing presentation
4. No error recovery—all failures are terminal for this CLI tool
5. Guard clause for missing arguments converts `None` to error

## Decision Framework Summary

Here's a decision tree for error handling:

```
┌─────────────────────┐
│ Can you recover?    │
└──────┬──────────────┘
       │
   ┌───┴───┐
   │  Yes  │ → Use match/or_else with fallback
   └───┬───┘
       │  No
       ▼
┌──────────────────────┐
│ Need to add context? │
└──────┬───────────────┘
       │
   ┌───┴───┐
   │  Yes  │ → Use map_err to enrich error
   └───┬───┘
       │  No
       ▼
┌──────────────────────────┐
│ Converting between types?│
└──────┬───────────────────┘
       │
   ┌───┴───┐
   │  Yes  │ → Implement From<E> for your error type
   └───┬───┘
       │  No
       ▼
┌────────────────┐
│ Propagate with?│
└────────────────┘
```

## Conclusion

Error handling patterns are **architectural tools**. They determine:
- **Where** errors are handled (boundaries between layers)
- **How** errors are transformed (conversions and context addition)
- **When** to recover vs propagate (based on domain knowledge)
- **What** information to preserve (for debugging and recovery)

The next section explores how these patterns manifest in **architectural implications** across system layers.
