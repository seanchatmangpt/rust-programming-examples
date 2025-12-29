# Clap Architecture Book - Code Examples

Complete, working code examples demonstrating Clap's features from basic to advanced usage patterns.

## Quick Start

```bash
# Build all examples
cargo build --workspace

# Run a specific example
cargo run -p hello-world -- World

# Run tests
cargo test --workspace

# View help for any example
cargo run -p <example-name> -- --help
```

## Examples Overview

| Example | Description | Key Concepts |
|---------|-------------|--------------|
| **01-hello-world** | Minimal CLI with derive macros | Parser derive, basic args, help generation |
| **02-arguments-basic** | All common argument types | Integers, strings, booleans, paths, vectors |
| **03-subcommands** | Nested command structure | Subcommand enum, global args, routing |
| **04-groups-and-conflicts** | Argument relationships | Mutual exclusivity, required groups, conflicts |
| **05-custom-parsers** | Custom type parsing | ValueParser, FromStr, validation |
| **06-builder-pattern** | Dynamic CLI construction | Builder API, reusable components |
| **07-derive-macros** | Complete attribute reference | All derive attributes showcase |
| **08-environment-config** | Environment and config files | Env vars, TOML config, layering |
| **09-error-handling** | Professional error handling | Custom errors, suggestions, exit codes |
| **10-plugins-architecture** | Plugin system design | Command registry, dynamic loading |
| **11-testing** | Testing CLI applications | Unit tests, integration tests, assert_cmd |
| **12-real-world-project** | Production-quality CLI | Multi-binary, modules, full architecture |

---

## Example Details

### 01 - Hello World

The simplest possible Clap application.

```bash
cargo run -p hello-world -- World
cargo run -p hello-world -- Alice --greeting "Good morning" --count 3
```

**Learn:**
- `#[derive(Parser)]` macro
- Positional vs optional arguments
- Default values
- Auto-generated help and version

---

### 02 - Arguments Basic

Comprehensive guide to all argument types.

```bash
cargo run -p arguments-basic -- input.txt -o output.txt -c 5 -v
cargo run -p arguments-basic -- file1.txt file2.txt --output result.json --tags rust,cli
```

**Learn:**
- Integer, float, string, boolean arguments
- Path arguments with ValueHint
- Vector arguments with delimiters
- Count actions for verbosity

---

### 03 - Subcommands

Git/cargo-style nested commands.

```bash
cargo run -p subcommands -- repo create my-project --public
cargo run -p subcommands -- config set user.email "me@example.com"
cargo run -p subcommands -- auth login --provider github
```

**Learn:**
- `#[derive(Subcommand)]` for command enums
- Nested subcommands
- Global arguments with `global = true`
- Command routing patterns

---

### 04 - Groups and Conflicts

Complex argument relationships.

```bash
cargo run -p groups-and-conflicts -- --input file.txt --json
cargo run -p groups-and-conflicts -- --stdin --yaml --pretty
cargo run -p groups-and-conflicts -- --url https://example.com --json --user admin --password secret
```

**Learn:**
- `ArgGroup` for mutually exclusive options
- Required groups (at least one)
- `conflicts_with` attribute
- `requires` for conditional dependencies

---

### 05 - Custom Parsers

Parsing custom types and validation.

```bash
cargo run -p custom-parsers -- --color "#ff5500" --size 1920x1080 --range 10..20 --email user@example.com --port 8080
cargo run -p custom-parsers -- --color red --size 800x600 --range 1..100 --email test@test.com --port 443 --duration 30s
```

**Learn:**
- Implementing `FromStr` for custom types
- `value_parser` attribute
- Custom validation functions
- `ValueEnum` for enums

---

### 06 - Builder Pattern

Dynamic CLI construction at runtime.

```bash
cargo run -p builder-pattern -- process file.txt --format json
cargo run -p builder-pattern -- convert input.csv --to yaml --pretty
cargo run -p builder-pattern -- analyze data.log --depth 3 --metrics count,sum
```

**Learn:**
- Builder API vs derive macros
- Reusable argument functions
- Command composition
- `ArgMatches` for accessing values

---

### 07 - Derive Macros

Complete reference of all derive attributes.

```bash
cargo run -p derive-macros -- user create john_doe --email john@example.com --role admin
cargo run -p derive-macros -- project build my-app --release --target linux
cargo run -p derive-macros -- system logs -n 100 --follow --level warn
```

**Learn:**
- All `#[command(...)]` attributes
- All `#[arg(...)]` attributes
- `#[derive(Args)]` for reusable groups
- `#[derive(ValueEnum)]` for enums

---

### 08 - Environment and Configuration

Environment variables and config file support.

```bash
cargo run -p environment-config
APP_PORT=3000 APP_HOST=0.0.0.0 cargo run -p environment-config
cargo run -p environment-config -- --config app.toml --show-config
```

**Learn:**
- `env = "VAR_NAME"` attribute
- Config file loading with serde
- Configuration layering (CLI > Env > File > Default)
- Secrets handling (env-only)

---

### 09 - Error Handling

Professional error handling patterns.

```bash
cargo run -p error-handling -- file read existing.txt
cargo run -p error-handling -- user get admn  # Shows suggestions
cargo run -p error-handling -- --verbose file read missing.txt  # Full error chain
```

**Learn:**
- Custom error types with thiserror
- Error chains with anyhow
- Exit codes per error type
- User-friendly suggestions

---

### 10 - Plugins Architecture

Building extensible CLI applications.

```bash
cargo run -p plugins-architecture -- file list /tmp
cargo run -p plugins-architecture -- user create john --email john@example.com
cargo run -p plugins-architecture -- plugin list
cargo run -p plugins-architecture -- plugin info file
```

**Learn:**
- Plugin trait definition
- Plugin registry pattern
- Dynamic command registration
- Command routing to plugins

---

### 11 - Testing

Testing CLI applications effectively.

```bash
cargo run -p testing -- hello World
cargo run -p testing -- add 1 2 3 4 5
cargo test -p testing
cargo test -p testing --test integration
```

**Learn:**
- Unit testing business logic
- CLI parsing tests with `try_parse_from()`
- Integration tests with assert_cmd
- Test organization patterns

---

### 12 - Real-World Project

A production-quality CLI like gcloud/aws.

```bash
cargo run -p cloudctl -- compute instances list
cargo run -p cloudctl -- storage buckets create my-bucket --region us-east-1
cargo run -p cloudctl -- --output json iam users list
cargo run -p cloudctl -- --profile production network vpcs list

# Additional binaries
cargo run -p cloudctl --bin cloud-auth -- login
cargo run -p cloudctl --bin cloud-config -- list
```

**Learn:**
- Multi-binary workspace
- Module organization
- Output formatters (table, JSON, YAML)
- Profile management
- Complete error handling

---

## Project Structure

```
clap-examples/
├── Cargo.toml                    # Workspace configuration
├── README.md                     # This file
└── examples/
    ├── 01-hello-world/
    │   ├── Cargo.toml
    │   └── src/main.rs
    ├── 02-arguments-basic/
    │   ├── Cargo.toml
    │   └── src/main.rs
    ├── 03-subcommands/
    │   ├── Cargo.toml
    │   └── src/main.rs
    ├── 04-groups-and-conflicts/
    │   ├── Cargo.toml
    │   └── src/main.rs
    ├── 05-custom-parsers/
    │   ├── Cargo.toml
    │   └── src/main.rs
    ├── 06-builder-pattern/
    │   ├── Cargo.toml
    │   └── src/main.rs
    ├── 07-derive-macros/
    │   ├── Cargo.toml
    │   └── src/main.rs
    ├── 08-environment-config/
    │   ├── Cargo.toml
    │   └── src/main.rs
    ├── 09-error-handling/
    │   ├── Cargo.toml
    │   └── src/main.rs
    ├── 10-plugins-architecture/
    │   ├── Cargo.toml
    │   └── src/main.rs
    ├── 11-testing/
    │   ├── Cargo.toml
    │   ├── src/main.rs
    │   └── tests/integration.rs
    └── 12-real-world-project/
        ├── Cargo.toml
        └── src/
            ├── main.rs
            ├── cli.rs
            ├── commands.rs
            ├── config.rs
            ├── error.rs
            ├── output.rs
            └── bin/
                ├── auth.rs
                └── config.rs
```

---

## Learning Path

### Beginner Path (2-3 hours)
1. **01-hello-world** - Get started with the basics
2. **02-arguments-basic** - Learn all argument types
3. **03-subcommands** - Build command hierarchies

### Intermediate Path (3-4 hours)
4. **04-groups-and-conflicts** - Master argument relationships
5. **05-custom-parsers** - Parse custom types
6. **07-derive-macros** - Comprehensive attribute reference
7. **08-environment-config** - Production configuration

### Advanced Path (4-5 hours)
8. **06-builder-pattern** - Dynamic CLI construction
9. **09-error-handling** - Professional error handling
10. **10-plugins-architecture** - Extensible design
11. **11-testing** - Testing strategies
12. **12-real-world-project** - Put it all together

---

## Common Commands

```bash
# Build specific example
cargo build -p hello-world

# Run with release optimizations
cargo run -p hello-world --release -- World

# Run all tests
cargo test --workspace

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --workspace

# Generate documentation
cargo doc --workspace --open
```

---

## Requirements

- **Rust**: 1.70+ (2021 edition)
- **Clap**: 4.5+

All dependencies are managed through the workspace `Cargo.toml`.

---

## Key Concepts Reference

### Derive Macros

| Derive | Purpose |
|--------|---------|
| `Parser` | Main CLI struct |
| `Subcommand` | Enum for subcommands |
| `Args` | Reusable argument groups |
| `ValueEnum` | Enum for argument values |

### Common Attributes

| Attribute | Purpose | Example |
|-----------|---------|---------|
| `short` | Short flag (-v) | `#[arg(short)]` |
| `long` | Long flag (--verbose) | `#[arg(long)]` |
| `default_value` | Default string | `#[arg(default_value = "x")]` |
| `default_value_t` | Default with Display | `#[arg(default_value_t = 1)]` |
| `env` | Environment variable | `#[arg(env = "VAR")]` |
| `value_parser` | Custom parser | `#[arg(value_parser = fn)]` |
| `value_enum` | Use ValueEnum | `#[arg(value_enum)]` |
| `global` | Available to subcommands | `#[arg(global = true)]` |
| `conflicts_with` | Mutual exclusivity | `#[arg(conflicts_with = "x")]` |
| `requires` | Conditional requirement | `#[arg(requires = "x")]` |

### Command Attributes

| Attribute | Purpose |
|-----------|---------|
| `name` | Override command name |
| `version` | Version string |
| `about` | Short description |
| `long_about` | Long description |
| `author` | Author information |
| `propagate_version` | Show version in subcommands |
| `arg_required_else_help` | Show help if no args |
| `subcommand_required` | Require a subcommand |

---

## License

MIT License - See LICENSE file for details.
