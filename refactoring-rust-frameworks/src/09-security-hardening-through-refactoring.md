# Security Hardening Through Refactoring

Security is not a feature to be bolted on after development; it is a fundamental property that must be woven into the fabric of framework design from the earliest stages.

## Common Vulnerabilities in Framework Code

The most prevalent vulnerabilities include:

- **Command injection**: Unsanitized input passed to shell commands
- **Path traversal**: Unvalidated file paths allowing access outside intended directories
- **Information disclosure**: Error messages revealing sensitive details
- **Denial of service**: Unbounded resource consumption
- **Dependency vulnerabilities**: Transitive dependencies with known issues

## Input Validation and Sanitization

### Validation Layers

A defense-in-depth approach requires multiple validation layers:

```rust
/// Layer 1: Syntactic validation at the boundary
pub struct RawInput(String);

impl RawInput {
    pub fn new(input: &str) -> Result<Self, ValidationError> {
        if input.len() > MAX_INPUT_LENGTH {
            return Err(ValidationError::TooLong);
        }

        if input.bytes().any(|b| b == 0 || (b < 32 && b != b'\n')) {
            return Err(ValidationError::InvalidCharacters);
        }

        Ok(RawInput(input.to_string()))
    }
}

/// Layer 2: Semantic validation with domain types
pub struct CommandName(String);

impl TryFrom<RawInput> for CommandName {
    type Error = ValidationError;

    fn try_from(raw: RawInput) -> Result<Self, Self::Error> {
        let name = raw.0.trim();

        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(ValidationError::InvalidCommandName);
        }

        if name.starts_with('-') {
            return Err(ValidationError::InvalidCommandName);
        }

        Ok(CommandName(name.to_string()))
    }
}
```

### Defending Against Injection Attacks

```rust
use std::process::Command;

/// VULNERABLE: Shell injection possible
fn vulnerable_execute(user_input: &str) {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("echo {}", user_input))
        .spawn();
}

/// SAFE: Arguments passed directly
fn safe_execute(validated_arg: &ValidatedArg) {
    Command::new("echo")
        .arg(validated_arg.as_str())
        .spawn()
        .expect("Failed to execute command");
}
```

### Boundary Checking

Establish explicit limits on all resource-consuming operations:

```rust
pub struct ResourceLimits {
    pub max_arguments: usize,
    pub max_argument_length: usize,
    pub max_nesting_depth: usize,
    pub max_total_size: usize,
}

pub fn parse_with_limits(
    input: &[String],
    limits: &ResourceLimits,
) -> Result<ParsedCommand, ParseError> {
    if input.len() > limits.max_arguments {
        return Err(ParseError::TooManyArguments);
    }

    let total_size: usize = input.iter().map(|s| s.len()).sum();
    if total_size > limits.max_total_size {
        return Err(ParseError::InputTooLarge);
    }

    parse_internal(input, limits.max_nesting_depth)
}
```

## Memory Safety

### Unsafe Code Review During Refactoring

```rust
/// SAFETY: Document what invariants the unsafe code relies upon
pub struct AlignedBuffer {
    ptr: *mut u8,
    len: usize,
}

impl AlignedBuffer {
    pub fn new(size: usize, alignment: usize) -> Result<Self, AllocationError> {
        // SAFETY: Layout is valid. We must deallocate with same layout in Drop.
        let layout = std::alloc::Layout::from_size_align(size, alignment)?;
        let ptr = unsafe { std::alloc::alloc_zeroed(layout) };

        if ptr.is_null() {
            return Err(AllocationError::OutOfMemory);
        }

        Ok(AlignedBuffer { ptr, len: size })
    }
}
```

### Using Miri for Detection

```bash
# Install miri
rustup +nightly component add miri

# Run tests under miri
cargo +nightly miri test
```

## Capability-Based Security

Design APIs where access rights are granted through unforgeable tokens:

```rust
pub struct FsCapability {
    allowed_paths: Vec<PathBuf>,
    permissions: FsPermissions,
    _private: (),
}

impl FsCapability {
    pub fn check(&self, path: &Path) -> Result<(), AccessDenied> {
        let canonical = path.canonicalize().map_err(|_| AccessDenied)?;

        let allowed = self.allowed_paths.iter()
            .any(|allowed| canonical.starts_with(allowed));

        if !allowed {
            return Err(AccessDenied);
        }

        Ok(())
    }
}

pub fn read_file(cap: &FsCapability, path: &Path) -> Result<Vec<u8>, FileError> {
    cap.check(path)?;
    std::fs::read(path).map_err(FileError::from)
}
```

## Error Handling Security

### Information Disclosure Prevention

```rust
pub struct InternalError {
    pub kind: ErrorKind,
    pub message: String,
    pub sensitive_context: Option<String>,
}

pub struct DisplayError {
    pub code: ErrorCode,
    pub message: String,
}

impl From<InternalError> for DisplayError {
    fn from(internal: InternalError) -> Self {
        // Map to safe, generic messages
        let (code, message) = match internal.kind {
            ErrorKind::Database => (
                ErrorCode::ServiceError,
                "An internal error occurred".to_string(),
            ),
            ErrorKind::Validation => (
                ErrorCode::InvalidInput,
                internal.message, // Safe to expose
            ),
            _ => (ErrorCode::Unknown, "An unexpected error occurred".to_string()),
        };

        DisplayError { code, message }
    }
}
```

### Secure Logging

```rust
pub struct Redacted<T>(T);

impl<T> std::fmt::Debug for Redacted<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

fn log_auth_attempt(username: &str, password: &str, success: bool) {
    info!(
        username = %username,
        password = %Redacted(password),
        success = success,
        "Authentication attempt"
    );
}
```

## Dependency Vulnerability Scanning

### cargo-audit

```bash
# Install audit tool
cargo install cargo-audit

# Scan for vulnerabilities
cargo audit
```

### deny.toml Configuration

```toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"

[licenses]
allow = ["MIT", "Apache-2.0"]
deny = ["GPL-2.0", "AGPL-3.0"]
```

## Cryptographic Considerations

### Secure Random Generation

```rust
use rand::{rngs::OsRng, RngCore};

pub fn generate_token() -> [u8; 32] {
    let mut token = [0u8; 32];
    OsRng.fill_bytes(&mut token);
    token
}
```

### Timing-Attack Resistance

```rust
use subtle::ConstantTimeEq;

pub fn verify_token(provided: &[u8; 32], expected: &[u8; 32]) -> bool {
    provided.ct_eq(expected).into()
}
```

## Audit Logging

Implement log entries that can detect tampering:

```rust
use sha2::{Sha256, Digest};

pub struct AuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: String,
    pub actor: String,
    pub action: String,
    pub outcome: Outcome,
    pub previous_hash: [u8; 32],
    pub hash: [u8; 32],
}

impl AuditEntry {
    pub fn verify_chain(entries: &[AuditEntry]) -> bool {
        for window in entries.windows(2) {
            if window[1].previous_hash != window[0].hash {
                return false;
            }
        }
        true
    }
}
```

## Best Practices

### Defense in Depth

Layer security controls:

1. **Input validation** at boundaries
2. **Type-safe** internal representations
3. **Capability-based** authorization
4. **Secure error** handling
5. **Comprehensive** audit logging
6. **Dependency** vulnerability scanning

### Threat Modeling

Before refactoring, identify:

- **Assets**: What are you protecting?
- **Threats**: Who might attack and how?
- **Vulnerabilities**: Where are weak points?
- **Mitigations**: How will you address each?

### Secure Defaults

Design APIs where the easiest path is also the secure path:

```rust
pub struct CommandBuilder {
    validate_input: bool,      // Default: true
    sandbox_execution: bool,   // Default: true
    log_operations: bool,      // Default: true
    timeout: Duration,         // Default: 30 seconds
}

impl Default for CommandBuilder {
    fn default() -> Self {
        CommandBuilder {
            validate_input: true,
            sandbox_execution: true,
            log_operations: true,
            timeout: Duration::from_secs(30),
        }
    }
}
```

## Summary

Security hardening during refactoring is not a one-time activity but an ongoing process. By integrating these practices into your development workflow, you build frameworks that are resilient to attack and maintain their security properties as they evolve.

Key principles:

- **Validate early and often** at system boundaries
- **Use types to enforce security** at compile time
- **Log comprehensively** for forensics
- **Update dependencies** promptly for security fixes
- **Test security** like you test functionality
- **Communicate transparently** about vulnerabilities

Security is a process, not a product. Make it a continuous part of your refactoring culture.
