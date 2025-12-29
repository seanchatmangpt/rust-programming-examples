# Architectural Implications

Error handling is not an implementation detail—it's a **fundamental architectural concern** that shapes system boundaries, defines API contracts, and determines system reliability. This section examines how error handling decisions cascade through software architecture.

## Error Propagation Boundaries in Layered Systems

Every layer in a system defines an **error boundary**—a point where errors are caught, transformed, or propagated. These boundaries are architectural seams that determine how failures flow through your application.

### The Three-Layer Pattern

Consider a typical web application with presentation, business logic, and data layers:

```rust
// Data Layer: Database-specific errors
mod data {
    use sqlx;

    #[derive(Debug)]
    pub enum DataError {
        ConnectionFailed(sqlx::Error),
        QueryFailed(sqlx::Error),
        NotFound,
        ConstraintViolation { field: String, constraint: String },
    }

    pub fn find_user(id: UserId) -> Result<User, DataError> {
        // Database operations that can fail
        todo!()
    }
}

// Business Layer: Domain-specific errors
mod business {
    use super::data::{DataError, find_user};

    #[derive(Debug)]
    pub enum BusinessError {
        UserNotFound(UserId),
        InvalidOperation(String),
        PermissionDenied { user: UserId, action: String },
        DataAccess(DataError),
    }

    impl From<DataError> for BusinessError {
        fn from(err: DataError) -> Self {
            match err {
                DataError::NotFound => BusinessError::InvalidOperation(
                    "Referenced entity not found".to_string()
                ),
                other => BusinessError::DataAccess(other),
            }
        }
    }

    pub fn get_user_profile(id: UserId) -> Result<UserProfile, BusinessError> {
        let user = find_user(id)?;  // DataError -> BusinessError
        Ok(UserProfile::from(user))
    }
}

// Presentation Layer: HTTP-specific errors
mod presentation {
    use super::business::{BusinessError, get_user_profile};
    use actix_web::{HttpResponse, ResponseError};

    #[derive(Debug)]
    pub enum ApiError {
        NotFound(String),
        BadRequest(String),
        Forbidden(String),
        InternalServerError,
    }

    impl From<BusinessError> for ApiError {
        fn from(err: BusinessError) -> Self {
            match err {
                BusinessError::UserNotFound(id) =>
                    ApiError::NotFound(format!("User {} not found", id)),
                BusinessError::PermissionDenied { action, .. } =>
                    ApiError::Forbidden(format!("Cannot perform: {}", action)),
                BusinessError::InvalidOperation(msg) =>
                    ApiError::BadRequest(msg),
                BusinessError::DataAccess(_) => {
                    log::error!("Data access error: {:?}", err);
                    ApiError::InternalServerError
                }
            }
        }
    }

    impl ResponseError for ApiError {
        fn error_response(&self) -> HttpResponse {
            match self {
                ApiError::NotFound(msg) =>
                    HttpResponse::NotFound().json(json!({ "error": msg })),
                ApiError::BadRequest(msg) =>
                    HttpResponse::BadRequest().json(json!({ "error": msg })),
                ApiError::Forbidden(msg) =>
                    HttpResponse::Forbidden().json(json!({ "error": msg })),
                ApiError::InternalServerError =>
                    HttpResponse::InternalServerError().json(
                        json!({ "error": "Internal server error" })
                    ),
            }
        }
    }
}
```

**Architectural principles:**
1. **Each layer defines its error domain**: Data layer knows about databases, business layer knows about domain rules, presentation layer knows about HTTP
2. **Errors transform at boundaries**: `DataError → BusinessError → ApiError`
3. **Information is preserved or hidden**: Internal errors logged, sanitized errors shown to users
4. **Recovery decisions occur at appropriate layers**: Business logic retries queries, presentation layer sets HTTP status codes

### The Actix-GCD Example

The `actix-gcd` project demonstrates **error handling at the presentation boundary**:

```rust
async fn post_gcd(form: web::Form<GcdParameters>) -> HttpResponse {
    if form.n == 0 || form.m == 0 {
        return HttpResponse::BadRequest()
            .content_type("text/html")
            .body("Computing the GCD with zero is boring.");
    }

    let response = format!(
        "The greatest common divisor of the numbers {} and {} is <b>{}</b>\n",
        form.n, form.m, gcd(form.n, form.m)
    );

    HttpResponse::Ok()
        .content_type("text/html")
        .body(response)
}
```

**Key architectural decisions:**
1. **Validation errors return immediately** with user-friendly messages
2. **No error type defined** because errors are simple (just invalid input)
3. **Guard clauses** prevent invalid states from reaching business logic
4. **HTTP layer owns presentation** (HTML formatting of errors)

This is **minimalist error architecture** appropriate for a simple application.

## Fallible Initialization Patterns

Initialization is a critical architectural phase where **many things can go wrong**. Rust provides several patterns for handling fallible initialization.

### Pattern 1: Constructor Returns Result

```rust
pub struct DatabaseConnection {
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl DatabaseConnection {
    pub fn new(connection_string: &str) -> Result<Self, sqlx::Error> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect_lazy(connection_string)?;

        Ok(Self { pool })
    }
}
```

**Advantages:**
- Clear that construction can fail
- Caller must handle the error
- No invalid states possible

**When to use:** When initialization has dependencies (network, filesystem, etc.)

### Pattern 2: Builder with Validation

```rust
pub struct ServerConfig {
    host: String,
    port: u16,
}

impl ServerConfig {
    pub fn builder() -> ServerConfigBuilder {
        ServerConfigBuilder::default()
    }
}

pub struct ServerConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
}

impl ServerConfigBuilder {
    pub fn build(self) -> Result<ServerConfig, ConfigError> {
        Ok(ServerConfig {
            host: self.host.ok_or(ConfigError::MissingHost)?,
            port: self.port.ok_or(ConfigError::MissingPort)?,
        })
    }
}
```

**Advantages:**
- Validation deferred to `build()`
- Partial configuration allowed
- Multiple configuration sources can be combined

**When to use:** Complex initialization with many optional parameters

### Pattern 3: Lazy Initialization with Once

```rust
use std::sync::Once;
use std::sync::Mutex;

static INIT: Once = Once::new();
static mut INSTANCE: Option<Mutex<Database>> = None;

pub fn get_database() -> Result<&'static Mutex<Database>, InitError> {
    unsafe {
        INIT.call_once(|| {
            match Database::connect() {
                Ok(db) => INSTANCE = Some(Mutex::new(db)),
                Err(_) => (),
            }
        });

        INSTANCE.as_ref().ok_or(InitError::ConnectionFailed)
    }
}
```

**Advantages:**
- Initialization happens only once
- Thread-safe
- Deferred until first use

**When to use:** Expensive resources that may never be needed

## Error Recovery Strategies at Different Layers

Different architectural layers have different **recovery capabilities** and responsibilities.

### Infrastructure Layer: Retry and Reconnect

At the infrastructure layer, errors are often **transient**:

```rust
mod infrastructure {
    use std::time::Duration;

    pub fn send_request(url: &str) -> Result<Response, NetworkError> {
        let max_retries = 3;
        let mut attempt = 0;

        loop {
            match try_send(url) {
                Ok(response) => return Ok(response),
                Err(e) if is_retryable(&e) && attempt < max_retries => {
                    attempt += 1;
                    let delay = Duration::from_millis(100 * 2u64.pow(attempt));
                    std::thread::sleep(delay);  // Exponential backoff
                }
                Err(e) => return Err(e),
            }
        }
    }

    fn is_retryable(err: &NetworkError) -> bool {
        matches!(err,
            NetworkError::Timeout |
            NetworkError::ConnectionReset |
            NetworkError::ServiceUnavailable
        )
    }
}
```

**Recovery strategy:** Retry with exponential backoff for transient failures.

### Domain Layer: Business Rule Enforcement

At the domain layer, errors enforce **business invariants**:

```rust
mod domain {
    pub struct BankAccount {
        balance: Decimal,
    }

    #[derive(Debug)]
    pub enum TransactionError {
        InsufficientFunds { required: Decimal, available: Decimal },
        AccountClosed,
        DailyLimitExceeded { limit: Decimal },
    }

    impl BankAccount {
        pub fn withdraw(&mut self, amount: Decimal) -> Result<(), TransactionError> {
            if self.balance < amount {
                return Err(TransactionError::InsufficientFunds {
                    required: amount,
                    available: self.balance,
                });
            }

            self.balance -= amount;
            Ok(())
        }
    }
}
```

**Recovery strategy:** Propagate to caller—only the caller knows how to respond to business rule violations (e.g., request more funds from user).

### Application Layer: Orchestration

At the application layer, errors are **orchestrated** across multiple domain operations:

```rust
mod application {
    use super::domain::{BankAccount, TransactionError};

    pub fn transfer_funds(
        from: &mut BankAccount,
        to: &mut BankAccount,
        amount: Decimal
    ) -> Result<(), TransactionError> {
        from.withdraw(amount)?;
        to.deposit(amount);
        Ok(())
    }

    pub fn transfer_with_rollback(
        from: &mut BankAccount,
        to: &mut BankAccount,
        amount: Decimal
    ) -> Result<(), TransferError> {
        let original_from = from.balance;
        let original_to = to.balance;

        match transfer_funds(from, to, amount) {
            Ok(()) => Ok(()),
            Err(e) => {
                // Rollback on error
                from.balance = original_from;
                to.balance = original_to;
                Err(TransferError::Failed(e))
            }
        }
    }
}
```

**Recovery strategy:** Rollback and restore previous state.

### Presentation Layer: User Communication

At the presentation layer, errors are **translated for users**:

```rust
mod presentation {
    use super::application::TransferError;
    use actix_web::{HttpResponse, ResponseError};

    impl ResponseError for TransferError {
        fn error_response(&self) -> HttpResponse {
            match self {
                TransferError::Failed(TransactionError::InsufficientFunds {
                    required, available
                }) => HttpResponse::BadRequest().json(json!({
                    "error": "Insufficient funds",
                    "required": required,
                    "available": available,
                    "suggestion": "Please deposit more funds"
                })),

                TransferError::Failed(TransactionError::AccountClosed) =>
                    HttpResponse::Forbidden().json(json!({
                        "error": "Account is closed",
                        "suggestion": "Contact customer support"
                    })),

                _ => HttpResponse::InternalServerError().json(json!({
                    "error": "Transfer failed"
                })),
            }
        }
    }
}
```

**Recovery strategy:** None—inform user and let them decide next action.

## Error Types as API Contracts

The error type you choose defines a **contract** with your API consumers about what can go wrong and how to handle it.

### Public API Error Contract

```rust
// Public library API
pub fn parse_configuration(source: &str) -> Result<Config, ParseError> {
    // Implementation details hidden
    todo!()
}

#[derive(Debug)]
pub enum ParseError {
    InvalidSyntax { line: usize, column: usize },
    MissingRequiredField(String),
    UnknownDirective(String),
}
```

**Contract guarantees:**
1. **Exhaustive**: Callers know all possible failure modes
2. **Stable**: Adding a variant is a breaking change
3. **Documented**: Each variant explains what went wrong
4. **Actionable**: Variants provide enough info for recovery

### Internal API Error Contract

```rust
// Internal module boundary
fn load_from_disk(path: &Path) -> Result<String, io::Error> {
    std::fs::read_to_string(path)
}

fn parse_toml(source: &str) -> Result<Config, toml::de::Error> {
    toml::from_str(source)
}

// Unified at public boundary
pub fn load_config(path: &Path) -> Result<Config, ConfigError> {
    let source = load_from_disk(path)
        .map_err(|e| ConfigError::IoError { path: path.to_owned(), source: e })?;

    parse_toml(&source)
        .map_err(|e| ConfigError::ParseError { source: e })?
}
```

**Architectural insight:** Internal modules use **concrete error types** from dependencies. Public API uses **unified custom type** that hides implementation details.

## Case Study: grep Error Architecture

Let's analyze the architectural implications of error handling in `grep`:

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

**Architectural decision:** Use `io::Result` because:
1. **All errors are I/O errors** (no parsing, no domain logic)
2. **Callers don't need to discriminate** between error types
3. **Standard library type** is well-understood
4. **No error recovery** at this layer—propagate to caller

```rust
fn grep_main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let target = match args.next() {
        Some(s) => s,
        None => Err("usage: grep PATTERN FILE...")?
    };

    // ... rest of implementation
}
```

**Architectural decision:** Use `Box<dyn Error>` because:
1. **Multiple error sources** (argument parsing, file opening, I/O)
2. **No error recovery** needed—all errors are terminal
3. **Simple error handling** at main boundary (just print and exit)
4. **Type erasure acceptable** since we don't need to match on error variants

```rust
fn main() {
    if let Err(err) = grep_main() {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
```

**Architectural decision:** Terminal error handling:
1. **Print to stderr** for error messages
2. **Exit with status 1** to signal failure to shell
3. **No recovery** because CLI tool—just fail fast
4. **Simple presentation** using `Display` trait

## Architectural Principles

### Principle 1: Errors Flow Upward, Recovery Flows Downward

**Errors propagate up** the call stack:
```
Data Layer → Business Layer → Application Layer → Presentation Layer
```

**Recovery decisions flow down**:
```
Presentation Layer decides → Application Layer orchestrates → Domain Layer enforces
```

### Principle 2: Each Layer Defines Its Error Domain

Don't leak implementation details:
- ❌ Bad: Presentation layer returns `sqlx::Error`
- ✅ Good: Presentation layer returns `ApiError::InternalServerError`

### Principle 3: Error Types Mirror Architectural Boundaries

Your error types should reflect your **module boundaries**:
```
src/
  data/mod.rs → DataError
  business/mod.rs → BusinessError
  api/mod.rs → ApiError
```

### Principle 4: Fail Fast at Boundaries

Validate at architectural boundaries:
```rust
// At API boundary
pub fn create_user(request: CreateUserRequest) -> Result<User, ApiError> {
    // Validate immediately
    if request.email.is_empty() {
        return Err(ApiError::InvalidInput("Email required".to_string()));
    }

    // Only valid data proceeds to business layer
    business::create_user(request)
}
```

## Conclusion

Error handling shapes your architecture through:
1. **Layer boundaries** defined by error type conversions
2. **API contracts** specified by error enumerations
3. **Recovery strategies** determined by architectural responsibilities
4. **Information flow** from detailed (infrastructure) to sanitized (presentation)

Well-designed error handling makes your architecture **visible in the type system**, **enforced by the compiler**, and **understandable to maintainers**. The next section examines these principles in action through a comprehensive case study.
