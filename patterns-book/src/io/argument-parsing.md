# Argument Parsing

## Context

You are building a command-line tool that accepts arguments from the user. These arguments might be flags, options with values, positional parameters, or file paths. Your tool needs to parse these arguments, validate them, and use them to control program behavior.

Command-line interfaces are a primary way to interact with system tools, build scripts, automation utilities, and development tools. Users expect standard conventions like `--help`, error messages for invalid input, and consistent argument syntax.

## Problem

**How do you parse command-line arguments in a way that handles various input formats, provides good error messages, supports common conventions (help text, optional flags, required parameters), and integrates smoothly with Rust's type system and error handling?**

Manually parsing `std::env::args()` with string matching is tedious and error-prone. Handling all edge cases (missing arguments, invalid values, help text) requires significant boilerplate. Different argument styles (flags, options, positional) need different parsing logic.

## Forces

- **Ergonomics**: Argument parsing code should be concise and declarative
- **Validation**: Arguments need type checking and constraint validation
- **Error messages**: Users need clear feedback on what went wrong
- **Help text**: `--help` should show usage information automatically
- **Standards**: Should follow Unix conventions (-, --, etc.)
- **Flexibility**: Support flags, options with values, positional arguments
- **Type safety**: Convert strings to appropriate types (int, bool, path)
- **Required vs optional**: Distinguish mandatory and optional arguments

## Solution

**For simple cases, use `std::env::args()` directly with manual parsing and pattern matching. For complex cases, use a library like `clap` to declaratively define arguments with types, validation, and automatic help generation.**

### Structure (Simple Manual Parsing)

```rust
fn parse_args() -> Result<Config, String> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < expected {
        return Err(format!("usage: {} <arguments>", args[0]));
    }

    Ok(Config {
        param1: args[1].clone(),
        param2: args[2].parse().map_err(|e| format!("Invalid param2: {}", e))?,
    })
}
```

### Real Implementation (from grep)

```rust
use std::error::Error;
use std::path::PathBuf;

fn grep_main() -> Result<(), Box<dyn Error>> {
    // Get the command-line arguments. The first argument is the
    // string to search for; the rest are filenames.
    let mut args = std::env::args().skip(1);
    let target = match args.next() {
        Some(s) => s,
        None => Err("usage: grep PATTERN FILE...")?
    };
    let files: Vec<PathBuf> = args.map(PathBuf::from).collect();

    if files.is_empty() {
        // Process stdin
    } else {
        // Process files
    }

    Ok(())
}
```

### Real Implementation (from copy)

```rust
use std::path::Path;

fn copy_main() -> io::Result<()> {
    let args = std::env::args_os().collect::<Vec<_>>();

    if args.len() < 3 {
        println!("usage: copy FILE... DESTINATION");
    } else if args.len() == 3 {
        dwim_copy(&args[1], &args[2])?;
    } else {
        let dst = Path::new(&args[args.len() - 1]);
        if !dst.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("target '{}' is not a directory", dst.display())
            ));
        }
        for i in 1 .. args.len() - 1 {
            copy_into(&args[i], dst)?;
        }
    }
    Ok(())
}
```

### Real Implementation (from http-get)

```rust
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

### Key Elements

1. **std::env::args()**: Returns iterator over command-line arguments as `String`
2. **std::env::args_os()**: Returns `OsString` for non-UTF8 paths (more robust)
3. **skip(1)**: First argument is program name, skip it
4. **Pattern matching**: Use `match` to handle missing arguments
5. **Type conversion**: Use `.parse()` to convert strings to numbers
6. **Error messages**: Print usage information when arguments are invalid
7. **Positional vs optional**: Determine behavior based on argument count

### args() vs args_os()

```rust
// args() - String (assumes UTF-8, panics on invalid UTF-8)
let args: Vec<String> = std::env::args().collect();

// args_os() - OsString (handles any OS path encoding)
let args: Vec<OsString> = std::env::args_os().collect();

// Use args_os() for file paths to support all possible filenames
```

## Resulting Context

### Benefits (Manual Parsing)

- **Zero dependencies**: Uses only standard library
- **Simple for simple cases**: Clear and explicit for 1-3 arguments
- **Full control**: Complete flexibility in parsing logic
- **Lightweight**: No additional compile-time or binary size cost

### Liabilities (Manual Parsing)

- **Boilerplate**: Repetitive code for validation and error messages
- **No help text**: Must manually implement `--help` flag
- **Error messages**: Must write custom messages for each error
- **Type conversion**: Manual parsing and error handling for each type
- **Scalability**: Gets unwieldy with many arguments or complex options

### When Manual Parsing Works

Use `std::env::args()` directly when:
- Tool has 1-3 simple positional arguments
- No optional flags or named options needed
- Error handling can be minimal
- You want zero dependencies

Examples: `cat file.txt`, `grep pattern file`, `echo hello world`

## Variations

### Parsing with Iterator Methods

```rust
fn parse_args() -> Result<(String, Vec<PathBuf>), String> {
    let mut args = std::env::args().skip(1);

    let pattern = args.next()
        .ok_or("Missing pattern argument")?;

    let files: Vec<PathBuf> = args.map(PathBuf::from).collect();

    Ok((pattern, files))
}
```

### Handling Flags

```rust
fn parse_with_flags() -> Result<Config, String> {
    let args: Vec<String> = std::env::args().collect();
    let mut config = Config::default();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "-v" | "--verbose" => {
                config.verbose = true;
                i += 1;
            }
            "-o" | "--output" => {
                if i + 1 < args.len() {
                    config.output = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--output requires a value".to_string());
                }
            }
            arg if arg.starts_with('-') => {
                return Err(format!("Unknown flag: {}", arg));
            }
            _ => {
                config.inputs.push(args[i].clone());
                i += 1;
            }
        }
    }

    Ok(config)
}
```

### stdin or Files Pattern

```rust
use std::io::{self, BufReader};
use std::fs::File;

fn process_inputs() -> io::Result<()> {
    let files: Vec<String> = std::env::args().skip(1).collect();

    if files.is_empty() {
        // No files: read from stdin
        let stdin = io::stdin();
        process_reader(stdin.lock())?;
    } else {
        // Process each file
        for filename in files {
            let file = File::open(&filename)?;
            process_reader(BufReader::new(file))?;
        }
    }

    Ok(())
}
```

### Using clap for Complex Parsing

```rust
use clap::{Arg, App};

fn main() {
    let matches = App::new("mytool")
        .version("1.0")
        .author("Author <email@example.com>")
        .about("Does awesome things")
        .arg(Arg::with_name("input")
            .short("i")
            .long("input")
            .value_name("FILE")
            .help("Sets the input file")
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .help("Enable verbose output"))
        .arg(Arg::with_name("count")
            .short("c")
            .long("count")
            .value_name("NUM")
            .help("Number of iterations")
            .default_value("1"))
        .get_matches();

    let input_file = matches.value_of("input").unwrap();
    let verbose = matches.is_present("verbose");
    let count: u32 = matches.value_of("count").unwrap().parse().unwrap();

    // Use the parsed values
    process(input_file, verbose, count);
}
```

### Using structopt (Derive-Based)

```rust
use structopt::StructOpt;
use std::path::PathBuf;

#[derive(StructOpt)]
#[structopt(name = "mytool", about = "An example CLI tool")]
struct Opt {
    /// Input file
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,

    /// Enable verbose output
    #[structopt(short, long)]
    verbose: bool,

    /// Number of iterations
    #[structopt(short, long, default_value = "1")]
    count: u32,

    /// Output file (optional)
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();

    process(&opt.input, opt.verbose, opt.count);
}
```

### Subcommands Pattern

```rust
use clap::{App, SubCommand};

fn main() {
    let matches = App::new("git-like")
        .subcommand(SubCommand::with_name("add")
            .about("Add files to index")
            .arg(Arg::with_name("files").multiple(true)))
        .subcommand(SubCommand::with_name("commit")
            .about("Commit changes")
            .arg(Arg::with_name("message").short("m").takes_value(true)))
        .get_matches();

    match matches.subcommand() {
        ("add", Some(sub_m)) => {
            let files: Vec<_> = sub_m.values_of("files").unwrap().collect();
            add_files(&files);
        }
        ("commit", Some(sub_m)) => {
            let message = sub_m.value_of("message").unwrap_or("No message");
            commit(message);
        }
        _ => {
            eprintln!("No subcommand specified");
        }
    }
}
```

## Related Patterns

- **Error Propagation**: Used to return errors from argument parsing
- **Line-Oriented Processing**: Often combined for file processing tools
- **Recursive Directory Walk**: File arguments may be directories
- **stdin or Files**: Common pattern in Unix tools

## Known Uses

### Standard Unix Tools

- **grep PATTERN [FILE...]**: Pattern and optional files
- **cat [FILE...]**: Optional files, defaults to stdin
- **wc [OPTIONS] [FILE...]**: Options and files
- **find PATH [OPTIONS]**: Path and search criteria
- **cp SOURCE DEST**: Two positional arguments

### Real Projects

```rust
// Simple tool with required positional argument
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    process_file(filename)?;

    Ok(())
}

// Tool with multiple arguments and validation
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 4 {
        eprintln!("usage: {} <host> <port> <command>", args[0]);
        std::process::exit(1);
    }

    let host = &args[1];
    let port: u16 = args[2].parse()
        .map_err(|_| "Invalid port number")?;
    let command = &args[3];

    connect(host, port, command)?;

    Ok(())
}

// Flexible argument handling
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        // Interactive mode
        run_interactive()?;
    } else if args[0] == "--help" || args[0] == "-h" {
        print_help();
    } else if args[0] == "--version" {
        println!("version 1.0.0");
    } else {
        // Process files
        for file in args {
            process_file(&file)?;
        }
    }

    Ok(())
}
```

### Integration Patterns

```rust
// Combined with error handling and I/O
use std::fs::File;
use std::io::{self, BufReader, BufRead};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);

    let pattern = args.next()
        .ok_or("Missing search pattern")?;

    let files: Vec<PathBuf> = args.map(PathBuf::from).collect();

    if files.is_empty() {
        let stdin = io::stdin();
        grep(&pattern, stdin.lock())?;
    } else {
        for path in files {
            let file = File::open(&path)
                .map_err(|e| format!("Cannot open {}: {}", path.display(), e))?;
            grep(&pattern, BufReader::new(file))?;
        }
    }

    Ok(())
}

fn grep<R: BufRead>(pattern: &str, reader: R) -> io::Result<()> {
    for line in reader.lines() {
        let line = line?;
        if line.contains(pattern) {
            println!("{}", line);
        }
    }
    Ok(())
}
```

## Implementation Notes

### Argument Count Validation

```rust
// Exact count
if args.len() != 3 {
    return Err("Expected exactly 2 arguments".to_string());
}

// Minimum count
if args.len() < 2 {
    return Err("At least 1 argument required".to_string());
}

// Range
if args.len() < 2 || args.len() > 5 {
    return Err("Expected 1 to 4 arguments".to_string());
}
```

### Type Conversion

```rust
// Parse integer
let count: i32 = args[1].parse()
    .map_err(|_| "Count must be an integer")?;

// Parse with validation
let port: u16 = args[2].parse()
    .map_err(|_| "Invalid port number")?;
if port < 1024 {
    return Err("Port must be >= 1024".into());
}

// Parse path (handles non-UTF8)
let path = PathBuf::from(&args[1]);
if !path.exists() {
    return Err(format!("Path does not exist: {}", path.display()).into());
}
```

### Help Text

```rust
fn print_help() {
    println!("Usage: mytool [OPTIONS] FILE...");
    println!();
    println!("Options:");
    println!("  -h, --help       Show this help message");
    println!("  -v, --verbose    Enable verbose output");
    println!("  -o, --output F   Write output to file F");
    println!();
    println!("Examples:");
    println!("  mytool input.txt");
    println!("  mytool -v -o out.txt input.txt");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        print_help();
        return;
    }

    // Continue with normal argument parsing
}
```

### Error Reporting

```rust
fn main() {
    match parse_and_run() {
        Ok(()) => {},
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!();
            eprintln!("Try --help for usage information");
            std::process::exit(1);
        }
    }
}
```

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_args() {
        // Note: Can't directly test std::env::args() in unit tests
        // Instead, create a function that takes args as parameter

        let args = vec!["prog".to_string(), "pattern".to_string()];
        let config = parse_args_from_vec(&args).unwrap();
        assert_eq!(config.pattern, "pattern");
    }

    // Testable version that doesn't use std::env::args()
    fn parse_args_from_vec(args: &[String]) -> Result<Config, String> {
        if args.len() < 2 {
            return Err("Missing pattern".to_string());
        }
        Ok(Config { pattern: args[1].clone() })
    }
}
```

### Comparison: Manual vs Library

| Aspect | Manual (`std::env::args`) | Library (`clap`/`structopt`) |
|--------|---------------------------|------------------------------|
| **Setup** | None | Add dependency |
| **Code** | ~10-30 lines | ~5-15 lines |
| **Help text** | Manual | Automatic |
| **Validation** | Manual | Declarative |
| **Type safety** | Manual parsing | Automatic |
| **Binary size** | Minimal | +200-500KB |
| **Compile time** | Fast | Slower |
| **Best for** | Simple tools | Complex CLIs |

## References

- Rust std::env documentation
- clap crate: https://docs.rs/clap
- structopt crate: https://docs.rs/structopt
- "Command Line Applications in Rust" book: https://rust-cli.github.io/book/
- "Programming Rust" Chapter 2: A Tour of Rust
