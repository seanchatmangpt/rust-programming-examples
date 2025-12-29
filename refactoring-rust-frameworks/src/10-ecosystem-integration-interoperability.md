# Ecosystem Integration & Interoperability

A CLI framework's value extends far beyond parsing command-line arguments. Modern applications operate within complex ecosystems spanning multiple programming languages, deployment environments, and integration points.

## Cross-Language Interoperability

### Foreign Function Interface for C/C++

The Foreign Function Interface forms the foundation of Rust's cross-language capabilities:

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[repr(C)]
pub struct CliParser {
    inner: *mut ParserInner,
}

#[repr(C)]
pub enum CliError {
    Success = 0,
    InvalidArgument = 1,
    ParseError = 2,
    OutOfMemory = 3,
}

#[no_mangle]
pub extern "C" fn cli_parser_new(program_name: *const c_char) -> *mut CliParser {
    let name = unsafe {
        if program_name.is_null() {
            return std::ptr::null_mut();
        }
        match CStr::from_ptr(program_name).to_str() {
            Ok(s) => s.to_owned(),
            Err(_) => return std::ptr::null_mut(),
        }
    };

    let inner = Box::new(ParserInner::new(name));
    let parser = Box::new(CliParser {
        inner: Box::into_raw(inner),
    });
    Box::into_raw(parser)
}

#[no_mangle]
pub extern "C" fn cli_parser_free(parser: *mut CliParser) {
    if !parser.is_null() {
        unsafe {
            let p = Box::from_raw(parser);
            if !p.inner.is_null() {
                drop(Box::from_raw(p.inner));
            }
        }
    }
}
```

### Python Bindings with PyO3

PyO3 provides ergonomic bindings for Python:

```rust
use pyo3::prelude::*;

#[pyclass]
struct ArgumentParser {
    name: String,
    arguments: Vec<ArgumentDef>,
}

#[pymethods]
impl ArgumentParser {
    #[new]
    fn new(name: String) -> Self {
        ArgumentParser {
            name,
            arguments: Vec::new(),
        }
    }

    fn add_argument(
        &mut self,
        name: String,
        required: Option<bool>,
    ) -> PyResult<()> {
        let def = ArgumentDef {
            name,
            required: required.unwrap_or(false),
        };
        self.arguments.push(def);
        Ok(())
    }

    fn parse_args(&self, py: Python<'_>, args: Vec<String>) -> PyResult<PyObject> {
        let parsed = self.parse_internal(&args)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        let dict = pyo3::types::PyDict::new(py);
        for (key, value) in parsed {
            dict.set_item(key, value)?;
        }
        Ok(dict.into())
    }
}

#[pymodule]
fn rust_cli(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<ArgumentParser>()?;
    Ok(())
}
```

### WebAssembly Compilation

WASM enables CLI logic to run in browsers and serverless environments:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmParser {
    config: ParserConfig,
}

#[wasm_bindgen]
impl WasmParser {
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: &str) -> Result<WasmParser, JsValue> {
        let config: ParserConfig = serde_json::from_str(config_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(WasmParser { config })
    }

    pub fn parse(&self, args: Box<[JsValue]>) -> Result<JsValue, JsValue> {
        let string_args: Vec<String> = args
            .iter()
            .filter_map(|v| v.as_string())
            .collect();

        let result = self.parse_internal(&string_args)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
```

## Standard Integration Points

### Configuration Format Support

CLI frameworks must read configuration from multiple sources:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub verbose: bool,
    #[serde(default)]
    pub output_format: OutputFormat,
}

impl AppConfig {
    /// Load with precedence:
    /// 1. Command-line arguments
    /// 2. Environment variables
    /// 3. User config file
    /// 4. Project config file
    /// 5. Default values
    pub fn load(cli_args: &CliArgs) -> Result<Self, ConfigError> {
        let mut config = Self::default();

        if let Some(user_config) = Self::load_user_config()? {
            config = config.merge(user_config);
        }

        config = config.apply_env()?;
        config = config.apply_cli(cli_args);

        Ok(config)
    }
}
```

### Shell Integration

Shell completions transform CLI usability:

```rust
pub trait CompletionGenerator {
    fn generate(&self, shell: Shell) -> String;
}

pub fn generate_bash(&self) -> String {
    let mut script = String::new();
    script.push_str(&format!(
        r#"_{name}_completions() {{
    local cur="${{COMP_WORDS[COMP_CWORD]}}"
    local commands="{commands}"
    COMPREPLY=($(compgen -W "$commands" -- "$cur"))
}}
complete -F _{name}_completions {name}
"#,
        name = self.name,
        commands = self.subcommands.join(" "),
    ));
    script
}
```

### Exit Codes and Output Protocols

Consistent exit codes enable scripting:

```rust
#[repr(u8)]
pub enum ExitCode {
    Success = 0,
    GeneralError = 1,
    UsageError = 64,
    DataError = 65,
    NoInput = 66,
    NoUser = 67,
    NoHost = 68,
    Unavailable = 69,
    Software = 70,
    OsError = 71,
    OsFile = 72,
    CantCreate = 73,
    IoError = 74,
    TempFail = 75,
    Protocol = 76,
    NoPermission = 77,
    Config = 78,
}
```

## Framework Composition

### Async Runtime Integration

```rust
pub trait AsyncExecutor {
    fn block_on<F: Future>(&self, future: F) -> F::Output;
}

pub struct TokioExecutor {
    runtime: tokio::runtime::Runtime,
}

impl AsyncExecutor for TokioExecutor {
    fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }
}
```

### Pluggable Backend Pattern

```rust
pub trait StorageBackend: Send + Sync {
    fn store(&self, key: &str, value: &[u8]) -> Result<(), StorageError>;
    fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError>;
}

pub struct BackendRegistry {
    backends: HashMap<String, Box<dyn StorageBackend>>,
}

impl BackendRegistry {
    pub fn get(&self, name: &str) -> Option<&dyn StorageBackend> {
        self.backends.get(name).map(|b| b.as_ref())
    }
}
```

### Middleware Pattern

```rust
pub trait Middleware: Send + Sync {
    fn before(&self, ctx: &mut Context) -> Result<(), MiddlewareError>;
    fn after(&self, ctx: &mut Context, result: &CommandResult) -> Result<(), MiddlewareError>;
}

pub struct MiddlewareChain {
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl MiddlewareChain {
    pub fn execute<F>(&self, mut ctx: Context, handler: F) -> Result<CommandResult, Error>
    where
        F: FnOnce(&Context) -> Result<CommandResult, Error>,
    {
        for mw in &self.middlewares {
            mw.before(&mut ctx)?;
        }

        let result = handler(&ctx)?;

        for mw in self.middlewares.iter().rev() {
            mw.after(&mut ctx, &result)?;
        }

        Ok(result)
    }
}
```

## Distributed Systems Patterns

### gRPC Integration

CLI tools often need to communicate with gRPC services using generated Rust bindings.

### Message Queue Compatibility

Integrate with message queues for async command processing:

```rust
pub trait MessageQueue: Send + Sync {
    fn publish(&self, topic: &str, message: &[u8]) -> Result<(), QueueError>;
}

pub struct CommandQueue<Q: MessageQueue> {
    queue: Q,
    topic: String,
}

impl<Q: MessageQueue> CommandQueue<Q> {
    pub fn submit(&self, command: Command) -> Result<JobId, QueueError> {
        let message = CommandMessage {
            job_id: JobId::new(),
            command,
            submitted_at: chrono::Utc::now(),
        };

        let encoded = serde_json::to_vec(&message)?;
        self.queue.publish(&self.topic, &encoded)?;

        Ok(message.job_id)
    }
}
```

## Documentation for Integrators

Clear documentation accelerates adoption:

```rust
/// # Integration Guide
///
/// ## Quick Start
///
/// Add to your `Cargo.toml`:
/// ```toml
/// [dependencies]
/// my-cli-framework = "1.0"
/// ```
///
/// ## Python Integration
///
/// ```python
/// from my_cli import Parser
/// parser = Parser("myapp")
/// parser.run()
/// ```
///
/// ## C Integration
///
/// ```c
/// #include <my_cli.h>
/// CliParser* parser = cli_parser_new("myapp");
/// cli_parser_free(parser);
/// ```
```

## Performance in Multi-Language Contexts

Minimize FFI crossings by batching operations:

```rust
#[no_mangle]
pub extern "C" fn cli_batch_execute(
    parser: *mut CliParser,
    commands: *const BatchCommand,
    count: usize,
) -> CliError {
    // Process all commands in single FFI call
}
```

## Best Practices

**Stable Interfaces**: Version your FFI and public APIs. Use semantic versioning rigorously.

**Minimal Coupling**: Design integration points that depend on abstractions, not implementations.

**Clear Contracts**: Document preconditions, postconditions, and invariants.

**Graceful Degradation**: Handle missing optional integrations gracefully.

**Consistent Behavior**: Ensure the same inputs produce the same outputs regardless of integration path.

**Security Boundaries**: Validate all data crossing FFI boundaries.

## Summary

The goal is building CLI frameworks that become natural parts of larger systems—tools that developers reach for instinctively because they integrate smoothly with everything else in their environment.

Key principles:

- **Support multiple languages** through FFI, Python, WASM
- **Standard integration points** for configuration, shell, processes
- **Composable architecture** with pluggable backends and middleware
- **Clear contracts** for distributed systems integration
- **Security-conscious** FFI design with validated boundaries

By designing for ecosystem integration, you multiply the impact and value of your framework.
