# API Quick Reference

> **Chapter 20** | Part 5: Reference & Appendices | Estimated reading time: 15 minutes

This chapter provides a comprehensive quick-reference guide to the most commonly used Clap APIs, attributes, and patterns. Use this as a lookup resource while developing CLI applications.

## Derive Macro Attributes

### Command-Level Attributes

Use `#[command(...)]` to configure the overall command or subcommand:

| Attribute | Description | Example |
|-----------|-------------|---------|
| `name` | Command name (defaults to crate name) | `#[command(name = "myapp")]` |
| `version` | Version from Cargo.toml | `#[command(version)]` |
| `author` | Author from Cargo.toml | `#[command(author)]` |
| `about` | Short description (first line of docs) | `#[command(about = "Does things")]` |
| `long_about` | Extended description | `#[command(long_about = "Detailed...")]` |
| `after_help` | Text displayed after help | `#[command(after_help = "EXAMPLES:...")]` |
| `before_help` | Text displayed before help | `#[command(before_help = "NOTE:...")]` |
| `after_long_help` | Text after long help | `#[command(after_long_help = "...")]` |
| `alias` | Hidden command alias | `#[command(alias = "cmd")]` |
| `visible_alias` | Visible command alias | `#[command(visible_alias = "cmd")]` |
| `aliases` | Multiple hidden aliases | `#[command(aliases = ["a", "b"])]` |
| `visible_aliases` | Multiple visible aliases | `#[command(visible_aliases = ["a", "b"])]` |
| `subcommand_required` | Require a subcommand | `#[command(subcommand_required = true)]` |
| `arg_required_else_help` | Show help if no args | `#[command(arg_required_else_help = true)]` |
| `propagate_version` | Pass version to subcommands | `#[command(propagate_version = true)]` |
| `disable_help_flag` | Remove built-in --help | `#[command(disable_help_flag = true)]` |
| `disable_version_flag` | Remove built-in --version | `#[command(disable_version_flag = true)]` |
| `disable_help_subcommand` | Remove help subcommand | `#[command(disable_help_subcommand = true)]` |
| `hide` | Hide from help output | `#[command(hide = true)]` |
| `next_line_help` | Put help on next line | `#[command(next_line_help = true)]` |
| `flatten_help` | Show subcommand args in parent help | `#[command(flatten_help = true)]` |

### Argument-Level Attributes

Use `#[arg(...)]` to configure individual arguments:

| Attribute | Description | Example |
|-----------|-------------|---------|
| `short` | Short flag (-v) | `#[arg(short)]` or `#[arg(short = 'V')]` |
| `long` | Long flag (--verbose) | `#[arg(long)]` or `#[arg(long = "verb")]` |
| `required` | Mark as required | `#[arg(required = true)]` |
| `default_value` | String default | `#[arg(default_value = "default")]` |
| `default_value_t` | Typed default | `#[arg(default_value_t = 8080)]` |
| `default_value_if` | Conditional default | `#[arg(default_value_if("flag", "true", "value"))]` |
| `default_value_ifs` | Multiple conditions | `#[arg(default_value_ifs([...]))]` |
| `env` | Environment variable | `#[arg(env = "MY_VAR")]` |
| `value_name` | Placeholder in help | `#[arg(value_name = "FILE")]` |
| `help` | Short help text | `#[arg(help = "Help text")]` |
| `long_help` | Extended help text | `#[arg(long_help = "Detailed...")]` |
| `hide` | Hide from help | `#[arg(hide = true)]` |
| `hide_default_value` | Hide default in help | `#[arg(hide_default_value = true)]` |
| `hide_env` | Hide env var in help | `#[arg(hide_env = true)]` |
| `hide_possible_values` | Hide possible values | `#[arg(hide_possible_values = true)]` |
| `conflicts_with` | Conflict with argument | `#[arg(conflicts_with = "other")]` |
| `conflicts_with_all` | Conflict with multiple | `#[arg(conflicts_with_all = ["a", "b"])]` |
| `requires` | Require other argument | `#[arg(requires = "other")]` |
| `requires_all` | Require multiple | `#[arg(requires_all = ["a", "b"])]` |
| `requires_if` | Conditional requirement | `#[arg(requires_if("value", "other"))]` |
| `required_if_eq` | Required if other equals | `#[arg(required_if_eq("other", "value"))]` |
| `required_unless_present` | Required unless other | `#[arg(required_unless_present = "other")]` |
| `action` | Parsing action | `#[arg(action = ArgAction::Count)]` |
| `value_parser` | Custom value parser | `#[arg(value_parser = my_parser)]` |
| `num_args` | Number of values | `#[arg(num_args = 1..=3)]` |
| `value_delimiter` | Split on delimiter | `#[arg(value_delimiter = ',')]` |
| `global` | Available to subcommands | `#[arg(global = true)]` |
| `exclusive` | Must be only argument | `#[arg(exclusive = true)]` |
| `last` | Must appear after -- | `#[arg(last = true)]` |
| `trailing_var_arg` | Consume remaining args | `#[arg(trailing_var_arg = true)]` |

### Container Attributes

| Attribute | Location | Description | Example |
|-----------|----------|-------------|---------|
| `subcommand` | Field | Mark enum field as subcommand | `#[command(subcommand)]` |
| `flatten` | Field | Flatten nested struct | `#[command(flatten)]` |
| `skip` | Field | Skip field during parsing | `#[arg(skip)]` |
| `from_global` | Field | Use global argument value | `#[arg(from_global)]` |
| `external_subcommand` | Enum variant | Capture unknown subcommands | `#[command(external_subcommand)]` |

## ArgAction Reference

Control how arguments are parsed and stored:

| Action | Description | Field Type |
|--------|-------------|------------|
| `Set` | Store single value (default) | `T` |
| `SetTrue` | Set bool to true | `bool` |
| `SetFalse` | Set bool to false | `bool` |
| `Count` | Count occurrences | `u8`, `u16`, `u32`, `u64`, `usize` |
| `Append` | Collect into Vec | `Vec<T>` |
| `Help` | Display help and exit | N/A |
| `HelpShort` | Display short help | N/A |
| `HelpLong` | Display long help | N/A |
| `Version` | Display version and exit | N/A |

```rust
use clap::{Parser, ArgAction};

#[derive(Parser)]
struct Cli {
    #[arg(short, long, action = ArgAction::SetTrue)]
    verbose: bool,

    #[arg(short, long, action = ArgAction::Count)]
    debug: u8,  // -d = 1, -dd = 2, -ddd = 3

    #[arg(short, long, action = ArgAction::Append)]
    include: Vec<String>,  // -i a -i b -i c
}
```

## ValueParser Reference

### Built-in Parsers

| Type | Implicit Parser | Notes |
|------|-----------------|-------|
| `String` | `value_parser!(String)` | UTF-8 validated |
| `OsString` | `value_parser!(OsString)` | Raw OS string |
| `PathBuf` | `value_parser!(PathBuf)` | File system path |
| `bool` | `BoolishValueParser` | true/false/yes/no/1/0 |
| `i8`-`i128` | `value_parser!(T)` | Signed integers |
| `u8`-`u128` | `value_parser!(T)` | Unsigned integers |
| `f32`, `f64` | `value_parser!(T)` | Floating point |
| `NonZeroU*` | `value_parser!(T)` | Non-zero unsigned |
| `NonZeroI*` | `value_parser!(T)` | Non-zero signed |

### Ranged Parsers

```rust
#[arg(value_parser = value_parser!(u16).range(1..=65535))]
port: u16,

#[arg(value_parser = value_parser!(i32).range(-100..=100))]
offset: i32,
```

### Possible Values

```rust
// String slice
#[arg(value_parser = ["debug", "info", "warn", "error"])]
level: String,

// PossibleValuesParser
#[arg(value_parser = PossibleValuesParser::new(["a", "b", "c"]))]
choice: String,

// ValueEnum (recommended)
#[derive(Clone, ValueEnum)]
enum Level { Debug, Info, Warn, Error }

#[arg(value_enum)]
level: Level,
```

### Custom ValueParser Signatures

```rust
// Function signature
fn parse_custom(s: &str) -> Result<MyType, String>

// Closure
value_parser!(|s: &str| -> Result<MyType, String> { ... })

// TypedValueParser trait
impl TypedValueParser for MyParser {
    type Value = MyType;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, clap::Error> {
        // Parse implementation
    }
}
```

## ErrorKind Reference

| Kind | Description | Common Cause |
|------|-------------|--------------|
| `InvalidValue` | Value failed validation | Out of range, wrong format |
| `UnknownArgument` | Unrecognized argument | Typo in flag name |
| `MissingRequiredArgument` | Required arg missing | User forgot argument |
| `WrongNumberOfValues` | Too few/many values | Wrong num_args |
| `ArgumentConflict` | Conflicting args used | conflicts_with violated |
| `MissingSubcommand` | Subcommand required | subcommand_required = true |
| `InvalidSubcommand` | Unknown subcommand | Typo in subcommand |
| `InvalidUtf8` | Non-UTF8 input | Binary data in string arg |
| `TooManyValues` | Too many values | Exceeded num_args max |
| `TooFewValues` | Too few values | Below num_args min |
| `ValueValidation` | Custom validation failed | value_parser error |
| `DisplayHelp` | Help requested | --help used |
| `DisplayVersion` | Version requested | --version used |
| `Io` | IO error | File operations |
| `Format` | Formatting error | Display issues |

### Creating Custom Errors

```rust
use clap::error::{Error, ErrorKind};

fn validate(value: &str) -> Result<String, Error> {
    if value.is_empty() {
        Err(Error::raw(ErrorKind::InvalidValue, "Value cannot be empty"))
    } else if value.len() > 100 {
        Err(Error::raw(ErrorKind::InvalidValue, "Value too long (max 100)"))
    } else {
        Ok(value.to_string())
    }
}
```

## Common Patterns Quick Reference

### Pattern 1: Basic CLI with Options

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, author, about)]
struct Cli {
    /// Input file
    input: PathBuf,

    /// Output file [default: stdout]
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}
```

### Pattern 2: Subcommands with Enum

```rust
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new item
    Add { name: String },
    /// Remove an item
    Remove { name: String },
    /// List all items
    List {
        #[arg(short, long)]
        all: bool,
    },
}
```

### Pattern 3: Shared Arguments with Flatten

```rust
#[derive(Args)]
struct GlobalArgs {
    #[arg(short, long, global = true)]
    verbose: bool,

    #[arg(long, global = true, env = "CONFIG")]
    config: Option<PathBuf>,
}

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    global: GlobalArgs,

    #[command(subcommand)]
    command: Commands,
}
```

### Pattern 4: Argument Groups

```rust
#[derive(Parser)]
struct Cli {
    #[arg(long, group = "input")]
    file: Option<PathBuf>,

    #[arg(long, group = "input")]
    url: Option<String>,

    #[arg(long, group = "input")]
    stdin: bool,
}
```

### Pattern 5: Environment Variable Fallback

```rust
#[derive(Parser)]
struct Cli {
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,

    #[arg(long, env = "LOG_LEVEL", default_value = "info")]
    log_level: String,
}
```

## Field Type Quick Reference

| Desired Behavior | Field Type | Attributes |
|------------------|------------|------------|
| Required single value | `T` | (none) |
| Optional single value | `Option<T>` | (none) |
| Multiple values | `Vec<T>` | (none) |
| Boolean flag | `bool` | `#[arg(short, long)]` |
| Occurrence count | `u8` | `#[arg(action = ArgAction::Count)]` |
| Optional with default | `T` | `#[arg(default_value_t = val)]` |
| Value from choices | `T` | `#[arg(value_enum)]` where T: ValueEnum |

## See Also

- [Chapter 7: Derive Macro Mastery](../part2-core-patterns/07-derive-macro-mastery.md) - Deep dive into derive attributes
- [Chapter 9: Value Parsing and Validation](../part2-core-patterns/09-value-parsing-validation.md) - Custom parsers
- [Chapter 5: Error Handling Foundations](../part1-foundations/05-error-handling-foundations.md) - Error types
- [Official Clap Documentation](https://docs.rs/clap) - Complete API reference

---

*Next: [Migration Guide](./21-migration-guide.md)*
