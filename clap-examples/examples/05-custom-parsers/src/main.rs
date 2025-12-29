//! # Example 05: Custom Parsers
//!
//! Demonstrates ValueParser, FromStr implementation, custom validation,
//! and complex parsing logic for CLI arguments.
//!
//! ## Run Examples:
//! ```bash
//! # Parse various custom types
//! cargo run -p custom-parsers -- \
//!     --color "#ff5500" \
//!     --size 1920x1080 \
//!     --range 10..20 \
//!     --email "user@example.com" \
//!     --port 8080 \
//!     --duration 30s \
//!     --level info
//!
//! # Using color names instead of hex
//! cargo run -p custom-parsers -- --color red --size 800x600 --range 1..100 \
//!     --email test@test.com --port 443
//!
//! # Validation errors (try these to see error messages):
//! cargo run -p custom-parsers -- --color invalid --size 800x600 --range 1..100 \
//!     --email test --port 70000
//!
//! # Invalid port (out of range)
//! cargo run -p custom-parsers -- --color red --size 800x600 --range 1..100 \
//!     --email test@test.com --port 70000
//!
//! # Invalid email format
//! cargo run -p custom-parsers -- --color red --size 800x600 --range 1..100 \
//!     --email not-an-email --port 8080
//!
//! # Key-value pairs
//! cargo run -p custom-parsers -- --color red --size 800x600 --range 1..100 \
//!     --email test@test.com --port 8080 \
//!     --config key1=value1 --config key2=value2
//! ```

use clap::{Parser, builder::ValueParser, value_parser};
use std::str::FromStr;
use std::fmt;
use std::num::ParseIntError;

// =============================================================================
// CUSTOM TYPES
// =============================================================================

/// RGB Color that can be parsed from hex (#RRGGBB) or named colors.
#[derive(Debug, Clone)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try named colors first
        match s.to_lowercase().as_str() {
            "red" => return Ok(Color { r: 255, g: 0, b: 0 }),
            "green" => return Ok(Color { r: 0, g: 255, b: 0 }),
            "blue" => return Ok(Color { r: 0, g: 0, b: 255 }),
            "white" => return Ok(Color { r: 255, g: 255, b: 255 }),
            "black" => return Ok(Color { r: 0, g: 0, b: 0 }),
            _ => {}
        }

        // Try hex format (#RRGGBB or RRGGBB)
        let hex = s.trim_start_matches('#');
        if hex.len() != 6 {
            return Err(format!(
                "Invalid color '{}'. Use hex (#RRGGBB) or name (red, green, blue, white, black)",
                s
            ));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| format!("Invalid red component in '{}'", s))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| format!("Invalid green component in '{}'", s))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| format!("Invalid blue component in '{}'", s))?;

        Ok(Color { r, g, b })
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

/// Screen size parsed from WIDTHxHEIGHT format.
#[derive(Debug, Clone)]
struct Size {
    width: u32,
    height: u32,
}

impl FromStr for Size {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('x').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid size '{}'. Use WIDTHxHEIGHT format (e.g., 1920x1080)", s));
        }

        let width = parts[0].parse::<u32>()
            .map_err(|_| format!("Invalid width in '{}'", s))?;
        let height = parts[1].parse::<u32>()
            .map_err(|_| format!("Invalid height in '{}'", s))?;

        if width == 0 || height == 0 {
            return Err("Width and height must be greater than 0".to_string());
        }

        Ok(Size { width, height })
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

/// Numeric range parsed from START..END format.
#[derive(Debug, Clone)]
struct Range {
    start: i64,
    end: i64,
}

impl FromStr for Range {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split("..").collect();
        if parts.len() != 2 {
            return Err(format!("Invalid range '{}'. Use START..END format (e.g., 10..20)", s));
        }

        let start = parts[0].parse::<i64>()
            .map_err(|_| format!("Invalid start value in '{}'", s))?;
        let end = parts[1].parse::<i64>()
            .map_err(|_| format!("Invalid end value in '{}'", s))?;

        if start >= end {
            return Err(format!("Start ({}) must be less than end ({})", start, end));
        }

        Ok(Range { start, end })
    }
}

/// Duration parsed from human-readable format (e.g., 30s, 5m, 2h).
#[derive(Debug, Clone)]
struct Duration {
    seconds: u64,
}

impl FromStr for Duration {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err("Duration cannot be empty".to_string());
        }

        // Check for suffix
        let (num_str, multiplier) = if s.ends_with('s') {
            (&s[..s.len()-1], 1u64)
        } else if s.ends_with('m') {
            (&s[..s.len()-1], 60u64)
        } else if s.ends_with('h') {
            (&s[..s.len()-1], 3600u64)
        } else if s.ends_with('d') {
            (&s[..s.len()-1], 86400u64)
        } else {
            // Assume seconds if no suffix
            (s, 1u64)
        };

        let value = num_str.parse::<u64>()
            .map_err(|_| format!("Invalid duration '{}'. Use format like 30s, 5m, 2h, 1d", s))?;

        Ok(Duration { seconds: value * multiplier })
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.seconds >= 86400 && self.seconds % 86400 == 0 {
            write!(f, "{}d", self.seconds / 86400)
        } else if self.seconds >= 3600 && self.seconds % 3600 == 0 {
            write!(f, "{}h", self.seconds / 3600)
        } else if self.seconds >= 60 && self.seconds % 60 == 0 {
            write!(f, "{}m", self.seconds / 60)
        } else {
            write!(f, "{}s", self.seconds)
        }
    }
}

/// Log level enum with custom parsing.
#[derive(Debug, Clone, clap::ValueEnum)]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Key-value pair for configuration.
#[derive(Debug, Clone)]
struct KeyValue {
    key: String,
    value: String,
}

impl FromStr for KeyValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid key=value pair: '{}'. Use KEY=VALUE format", s));
        }

        let key = parts[0].trim().to_string();
        let value = parts[1].trim().to_string();

        if key.is_empty() {
            return Err("Key cannot be empty".to_string());
        }

        Ok(KeyValue { key, value })
    }
}

// =============================================================================
// CUSTOM VALIDATORS (as functions)
// =============================================================================

/// Validate email address format (simple validation).
fn parse_email(s: &str) -> Result<String, String> {
    // Simple validation - contains @ and at least one .
    if !s.contains('@') {
        return Err(format!("Invalid email '{}': missing @", s));
    }

    let parts: Vec<&str> = s.split('@').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid email '{}': multiple @ symbols", s));
    }

    if parts[0].is_empty() {
        return Err(format!("Invalid email '{}': empty local part", s));
    }

    if !parts[1].contains('.') {
        return Err(format!("Invalid email '{}': domain missing .", s));
    }

    Ok(s.to_string())
}

/// Validate port number with custom error message.
fn parse_port(s: &str) -> Result<u16, String> {
    let port: u16 = s.parse()
        .map_err(|_: ParseIntError| format!("'{}' is not a valid port number", s))?;

    if port == 0 {
        return Err("Port 0 is reserved".to_string());
    }

    Ok(port)
}

// =============================================================================
// CLI DEFINITION
// =============================================================================

/// Demonstrates custom value parsing in Clap.
#[derive(Parser, Debug)]
#[command(name = "custom-parsers")]
#[command(version, about)]
struct Cli {
    /// Color value (hex: #RRGGBB or name: red, green, blue, white, black)
    #[arg(long, value_parser = clap::value_parser!(Color))]
    color: Color,

    /// Screen size (format: WIDTHxHEIGHT, e.g., 1920x1080)
    #[arg(long, value_parser = clap::value_parser!(Size))]
    size: Size,

    /// Numeric range (format: START..END, e.g., 10..20)
    #[arg(long, value_parser = clap::value_parser!(Range))]
    range: Range,

    /// Email address (validated)
    #[arg(long, value_parser = parse_email)]
    email: String,

    /// Port number (1-65535)
    #[arg(long, value_parser = parse_port)]
    port: u16,

    /// Duration (format: 30s, 5m, 2h, 1d)
    #[arg(long, value_parser = clap::value_parser!(Duration), default_value = "60s")]
    duration: Duration,

    /// Log level
    #[arg(long, value_enum, default_value_t = LogLevel::Info)]
    level: LogLevel,

    /// Configuration key=value pairs (can be repeated)
    #[arg(long = "config", value_parser = clap::value_parser!(KeyValue))]
    configs: Vec<KeyValue>,

    /// Validated integer with range (using built-in range validator)
    #[arg(long, value_parser = value_parser!(u8).range(1..=100), default_value_t = 50)]
    percentage: u8,

    /// Possible values from a fixed set
    #[arg(long, value_parser = ["small", "medium", "large"], default_value = "medium")]
    shirt_size: String,
}

// =============================================================================
// MAIN
// =============================================================================

fn main() {
    let cli = Cli::parse();

    println!("=== Parsed Custom Types ===\n");

    println!("Color: {} (R={}, G={}, B={})",
             cli.color, cli.color.r, cli.color.g, cli.color.b);
    println!("Size: {} ({}x{} pixels)",
             cli.size, cli.size.width, cli.size.height);
    println!("Range: {}..{} ({} values)",
             cli.range.start, cli.range.end, cli.range.end - cli.range.start);
    println!("Email: {}", cli.email);
    println!("Port: {}", cli.port);
    println!("Duration: {} ({} seconds)", cli.duration, cli.duration.seconds);
    println!("Log level: {:?}", cli.level);
    println!("Percentage: {}%", cli.percentage);
    println!("Shirt size: {}", cli.shirt_size);

    if !cli.configs.is_empty() {
        println!("\nConfiguration:");
        for kv in &cli.configs {
            println!("  {} = {}", kv.key, kv.value);
        }
    }
}

// =============================================================================
// KEY CONCEPTS:
// =============================================================================
//
// 1. FROM_STR TRAIT:
//    Implement FromStr for custom types to enable automatic parsing.
//    Return descriptive error messages for user-friendly error output.
//
// 2. VALUE_PARSER ATTRIBUTE:
//    #[arg(value_parser = ...)] specifies how to parse the argument.
//    - clap::value_parser!(Type) - for types implementing FromStr
//    - function_name - for custom validation functions
//    - value_parser!(u8).range(1..=100) - built-in validators
//    - ["a", "b", "c"] - fixed set of allowed values
//
// 3. CUSTOM VALIDATOR FUNCTIONS:
//    fn parse_thing(s: &str) -> Result<T, String>
//    Return Ok(value) on success, Err(message) on failure.
//    Error messages should be user-friendly.
//
// 4. VALUE_ENUM:
//    #[derive(ValueEnum)] for enum types with automatic parsing.
//    Clap generates valid values list for help text.
//
// 5. ERROR MESSAGES:
//    Clap displays your error messages when parsing fails.
//    Include the invalid value and expected format in errors.
//
// 6. BUILT-IN VALIDATORS:
//    - .range(1..100) - numeric range validation
//    - PossibleValuesParser - restrict to specific strings
//    - PathBufValueParser - path validation
//
// 7. COMPLEX PARSING:
//    For complex formats (WIDTHxHEIGHT, KEY=VALUE, START..END),
//    implement FromStr with detailed parsing logic.
//
// BEST PRACTICES:
//
// - Always provide helpful error messages
// - Include the invalid value in error messages
// - Suggest the correct format in error messages
// - Use type system to enforce constraints
// - Test edge cases in parser implementations
