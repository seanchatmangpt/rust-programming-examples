# Case Study: HTTP Client Error Architecture

The `http-get` project provides a compact but complete example of error handling in a networked application. Despite its simplicity—just 32 lines of code—it demonstrates sophisticated architectural decisions about error propagation, type unification, and boundary handling. Let's dissect this example to understand real-world error architecture.

## The Complete Implementation

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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: http-get URL");
        return;
    }

    if let Err(err) = http_get_main(&args[1]) {
        eprintln!("error: {}", err);
    }
}
```

This small program makes **seven distinct architectural decisions** about error handling. Let's examine each one.

## Decision 1: Type Erasure with Box&lt;dyn Error&gt;

```rust
fn http_get_main(url: &str) -> Result<(), Box<dyn Error>>
```

**Architectural choice:** Use type erasure instead of a concrete error type.

**Rationale:**
1. **Multiple error sources** exist in this function:
   - `reqwest::blocking::get()` returns `reqwest::Error`
   - `io::copy()` returns `io::Error`
   - Manual error creation from `format!()` produces `String`

2. **No error recovery** is performed—all errors are terminal
3. **Callers don't need to discriminate** between error types
4. **Simplicity over type safety** is appropriate for a CLI tool

**Trade-offs:**

| Advantage | Disadvantage |
|-----------|--------------|
| Unifies multiple error types | Loses type information |
| Simpler function signature | Can't match on specific errors |
| Automatic conversions via `?` | Runtime overhead (heap allocation) |
| Flexible—easy to add new errors | Less informative for library users |

**When this is appropriate:**
- ✅ Application code (not libraries)
- ✅ Simple error handling (just print and exit)
- ✅ Multiple heterogeneous error sources
- ✅ No error recovery logic needed

**When to use concrete types instead:**
- ❌ Library APIs where callers need to handle specific errors
- ❌ Error recovery based on error variants
- ❌ Performance-critical code (avoid heap allocation)

## Decision 2: Network Error Propagation

```rust
let mut response = reqwest::blocking::get(url)?;
```

**Architectural choice:** Propagate network errors immediately without recovery.

**Error categories in network operations:**

```rust
// reqwest::Error can represent:
pub enum NetworkErrorKind {
    // Connection errors
    ConnectionRefused,
    DnsResolutionFailed,
    NetworkUnreachable,

    // Protocol errors
    InvalidUrl,
    InvalidHeader,
    TlsHandshakeFailed,

    // Timeout errors
    ConnectionTimeout,
    RequestTimeout,

    // Server errors
    ServerResponseError,
}
```

**Architectural insight:** For a CLI tool, **all network errors are fatal**. There's no point distinguishing between "DNS failed" and "connection refused"—the user gets the same result either way (failure).

**Alternative approach for a library:**

```rust
#[derive(Debug)]
pub enum HttpGetError {
    Network { url: String, source: reqwest::Error },
    InvalidUrl(String),
    Timeout { url: String, duration: Duration },
    HttpError { status: u16, body: Option<String> },
    IoError(io::Error),
}

pub fn http_get_with_recovery(url: &str) -> Result<Vec<u8>, HttpGetError> {
    let response = reqwest::blocking::get(url)
        .map_err(|e| {
            if e.is_timeout() {
                HttpGetError::Timeout {
                    url: url.to_string(),
                    duration: Duration::from_secs(30),
                }
            } else if e.is_connect() {
                HttpGetError::Network {
                    url: url.to_string(),
                    source: e,
                }
            } else {
                HttpGetError::Network {
                    url: url.to_string(),
                    source: e,
                }
            }
        })?;

    // More error handling...
    Ok(vec![])
}
```

This library version **categorizes errors** to enable recovery strategies (like retrying on timeout).

## Decision 3: HTTP Status Code Validation

```rust
if !response.status().is_success() {
    Err(format!("{}", response.status()))?;
}
```

**Architectural choice:** Treat non-success HTTP status codes as errors.

**Why this matters:**
- `reqwest` considers HTTP 404 or 500 as **successful** requests (the HTTP protocol succeeded)
- Application semantics require treating 404/500 as **errors**

**HTTP Status Code Taxonomy:**

| Range | Category | Interpretation | Error? |
|-------|----------|----------------|--------|
| 200-299 | Success | Request succeeded | No |
| 300-399 | Redirect | Resource moved | Maybe |
| 400-499 | Client Error | Invalid request | Yes |
| 500-599 | Server Error | Server failed | Yes |

**Architectural implications:**

```rust
// For a more sophisticated client:
match response.status().as_u16() {
    200..=299 => Ok(response),  // Success

    300..=399 => {
        // Follow redirects automatically (reqwest can do this)
        Err(HttpError::Redirect {
            status: response.status(),
            location: response.headers().get("Location"),
        })
    }

    400..=499 => {
        // Client error - don't retry
        Err(HttpError::ClientError {
            status: response.status(),
            body: response.text().ok(),
        })
    }

    500..=599 => {
        // Server error - maybe retry
        Err(HttpError::ServerError {
            status: response.status(),
            retryable: true,
        })
    }

    _ => Err(HttpError::UnknownStatus(response.status())),
}
```

This categorization enables **appropriate recovery strategies** per error category.

## Decision 4: Error Conversion for String

```rust
Err(format!("{}", response.status()))?;
```

**Architectural observation:** This line creates a `String` and converts it to `Box<dyn Error>`.

**How this works:**

```rust
// String doesn't implement Error, but...
impl From<String> for Box<dyn Error> {
    fn from(s: String) -> Self {
        Box::new(StringError(s))
    }
}

// The ? operator calls this conversion automatically
```

**Better alternatives:**

```rust
// Option 1: Use a proper error type
#[derive(Debug)]
struct HttpStatusError {
    status: reqwest::StatusCode,
}

impl fmt::Display for HttpStatusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HTTP error: {}", self.status)
    }
}

impl Error for HttpStatusError {}

// Option 2: Use an existing error type
use std::io::{Error as IoError, ErrorKind};
Err(IoError::new(
    ErrorKind::Other,
    format!("HTTP {}", response.status())
))?;
```

## Decision 5: I/O Error Propagation

```rust
let stdout = io::stdout();
io::copy(&mut response, &mut stdout.lock())?;
```

**Architectural choice:** Propagate I/O errors when writing to stdout.

**Potential I/O errors:**

```rust
pub enum IoErrorKind {
    BrokenPipe,      // stdout was closed (piped to process that exited)
    PermissionDenied, // Can't write to stdout (rare)
    WriteZero,       // Couldn't write any bytes
    Interrupted,     // System call interrupted
    // ... many more
}
```

**Why propagate without handling?**

For a CLI tool, **stdout failures are terminal**. If stdout breaks:
- User's shell/terminal has issues
- Downstream pipe consumer crashed
- File redirection target is full/invalid

None of these are recoverable by the application.

**Alternative for a library:**

```rust
pub fn http_get_to_writer<W: Write>(
    url: &str,
    writer: &mut W
) -> Result<u64, HttpGetError> {
    let mut response = reqwest::blocking::get(url)?;

    io::copy(&mut response, writer)
        .map_err(|e| HttpGetError::WriteError {
            bytes_written: 0,  // Could track this
            source: e,
        })
}
```

This preserves **how many bytes were written** before failure, enabling partial success handling.

## Decision 6: Argument Validation

```rust
if args.len() != 2 {
    eprintln!("usage: http-get URL");
    return;
}
```

**Architectural choice:** Validate arguments in `main()` before calling `http_get_main()`.

**Why separate validation from main logic?**

1. **Single Responsibility**: `http_get_main()` assumes valid inputs
2. **Testability**: Can test `http_get_main()` without mocking `env::args()`
3. **Clear boundaries**: Argument parsing is presentation layer concern

**Alternative: Error-based validation**

```rust
fn parse_args() -> Result<String, ArgError> {
    let mut args = env::args().skip(1);
    args.next().ok_or(ArgError::MissingUrl)
}

fn main() {
    match parse_args() {
        Ok(url) => {
            if let Err(e) = http_get_main(&url) {
                eprintln!("error: {}", e);
            }
        }
        Err(ArgError::MissingUrl) => {
            eprintln!("usage: http-get URL");
            std::process::exit(1);
        }
    }
}
```

This makes argument errors **explicit in the type system**.

## Decision 7: Terminal Error Handling

```rust
if let Err(err) = http_get_main(&args[1]) {
    eprintln!("error: {}", err);
}
```

**Architectural choice:** Print error to stderr and exit silently.

**Error presentation strategy:**

```rust
// Current approach: Simple message
eprintln!("error: {}", err);

// Enhanced: Include error chain
eprintln!("error: {}", err);
let mut source = err.source();
while let Some(err) = source {
    eprintln!("  caused by: {}", err);
    source = err.source();
}

// Production: Structured logging
log::error!("HTTP request failed";
    "url" => %url,
    "error" => %err,
    "error_chain" => ?err.source()
);
```

**Exit code handling:**

```rust
// Current: Silent exit (code 0 even on error!)
if let Err(err) = http_get_main(&args[1]) {
    eprintln!("error: {}", err);
    // Missing: std::process::exit(1);
}

// Better: Explicit failure exit code
fn main() {
    let exit_code = match run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("error: {}", e);
            1
        }
    };
    std::process::exit(exit_code);
}
```

## Layered Error Propagation Analysis

The `http-get` program has **three conceptual layers**, even though it's tiny:

### Layer 1: Network I/O (reqwest)

```rust
reqwest::blocking::get(url)
    // Returns: Result<Response, reqwest::Error>
    // Error categories:
    //   - DNS resolution
    //   - TCP connection
    //   - TLS handshake
    //   - HTTP protocol
```

### Layer 2: Application Logic (http_get_main)

```rust
fn http_get_main(url: &str) -> Result<(), Box<dyn Error>> {
    let response = reqwest::blocking::get(url)?;
    //             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Layer 1 error
    //                                            converted to Box<dyn Error>

    if !response.status().is_success() {
        Err(...)?;  // Layer 2 error (application semantics)
    }

    io::copy(&mut response, &mut stdout)?;
    //                                  ^ Layer 1 error (different type)
    Ok(())
}
```

### Layer 3: User Interface (main)

```rust
fn main() {
    if let Err(err) = http_get_main(&args[1]) {
        eprintln!("error: {}", err);
        //        ^^^^^^^^^^^^^^^ Layer 2 error converted to string
    }
}
```

**Error transformations:**

```
reqwest::Error ──┐
                 ├──> Box<dyn Error> ──> String (via Display)
io::Error ───────┘
```

## Converting Between Error Types Across Layers

The `?` operator performs **automatic conversions** at layer boundaries:

```rust
// Explicit version (what ? does):
let response = match reqwest::blocking::get(url) {
    Ok(r) => r,
    Err(e) => {
        // Convert reqwest::Error to Box<dyn Error>
        let boxed: Box<dyn Error> = Box::new(e);
        return Err(boxed);
    }
};

// Concise version with ?:
let response = reqwest::blocking::get(url)?;
```

**How this works:**

```rust
// Box<dyn Error> implements From for any Error type
impl<E: Error + 'static> From<E> for Box<dyn Error> {
    fn from(err: E) -> Box<dyn Error> {
        Box::new(err)
    }
}

// So ? can convert:
reqwest::Error -> Box<dyn Error>  ✓
io::Error      -> Box<dyn Error>  ✓
String         -> Box<dyn Error>  ✓ (via intermediate type)
```

## Real-World Error Handling Patterns

Based on the `http-get` architecture, here are patterns for production HTTP clients:

### Pattern 1: Retry with Backoff

```rust
fn http_get_with_retry(url: &str, max_retries: u32) -> Result<Response, HttpError> {
    let mut attempt = 0;
    let mut delay = Duration::from_millis(100);

    loop {
        match reqwest::blocking::get(url) {
            Ok(response) => {
                if response.status().is_success() {
                    return Ok(response);
                } else if is_retryable(response.status()) && attempt < max_retries {
                    attempt += 1;
                    std::thread::sleep(delay);
                    delay *= 2;
                } else {
                    return Err(HttpError::HttpStatus(response.status()));
                }
            }
            Err(e) if is_transient(&e) && attempt < max_retries => {
                attempt += 1;
                std::thread::sleep(delay);
                delay *= 2;
            }
            Err(e) => return Err(HttpError::Network(e)),
        }
    }
}

fn is_retryable(status: StatusCode) -> bool {
    matches!(status.as_u16(),
        408 | // Request Timeout
        429 | // Too Many Requests
        500 | // Internal Server Error
        502 | // Bad Gateway
        503 | // Service Unavailable
        504   // Gateway Timeout
    )
}

fn is_transient(err: &reqwest::Error) -> bool {
    err.is_timeout() || err.is_connect()
}
```

### Pattern 2: Circuit Breaker

```rust
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct CircuitBreaker {
    failure_threshold: u32,
    timeout: Duration,
    failures: Arc<Mutex<u32>>,
    last_failure: Arc<Mutex<Option<Instant>>>,
}

impl CircuitBreaker {
    fn call<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let mut failures = self.failures.lock().unwrap();
        let mut last_failure = self.last_failure.lock().unwrap();

        // Check if circuit is open
        if *failures >= self.failure_threshold {
            if let Some(last) = *last_failure {
                if last.elapsed() < self.timeout {
                    return Err(CircuitBreakerError::Open);
                }
            }
            // Reset after timeout
            *failures = 0;
            *last_failure = None;
        }

        // Try operation
        match f() {
            Ok(result) => {
                *failures = 0;
                Ok(result)
            }
            Err(e) => {
                *failures += 1;
                *last_failure = Some(Instant::now());
                Err(CircuitBreakerError::Failed(e))
            }
        }
    }
}
```

### Pattern 3: Timeout Enforcement

```rust
use std::time::Duration;

fn http_get_with_timeout(
    url: &str,
    timeout: Duration
) -> Result<Response, HttpError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()?;

    client.get(url)
        .send()
        .map_err(|e| {
            if e.is_timeout() {
                HttpError::Timeout { url: url.to_string(), duration: timeout }
            } else {
                HttpError::Network(e)
            }
        })
}
```

## Lessons from http-get Architecture

### Lesson 1: Simple is Often Right

For a CLI tool, `Box<dyn Error>` is **architecturally appropriate**:
- No error recovery needed
- Multiple error sources
- Users just see a message and exit code

Don't over-engineer error types when simple will do.

### Lesson 2: Layer Boundaries Define Error Conversions

Errors cross boundaries:
```
reqwest → http_get_main → main → user
```

Each boundary is a conversion opportunity:
```
reqwest::Error → Box<dyn Error> → String
```

### Lesson 3: Validation at Entry Points

`main()` validates arguments **before** calling `http_get_main()`. This keeps the core logic clean and testable.

### Lesson 4: Status Codes Are Application Semantics

HTTP 404 is protocol success but application failure. **Don't conflate protocol and application semantics**.

### Lesson 5: Terminal Errors Don't Need Fancy Types

When all errors are terminal (print and exit), type erasure is fine. Save custom error types for **recoverable errors**.

## Conclusion

The `http-get` project, despite its brevity, demonstrates production-ready error handling architecture:
- **Type erasure** for terminal errors
- **Immediate propagation** without recovery
- **Layer boundaries** with automatic conversions
- **HTTP semantics** validated separately from network success
- **Clean separation** between validation and business logic

This architecture scales to larger systems by adding:
- Custom error types for recoverable errors
- Retry logic for transient failures
- Circuit breakers for cascading failures
- Structured logging for observability

The principles remain the same: **errors are values, layer boundaries define conversions, and architectural decisions determine when to recover vs propagate**.
