# Value Parsing Pipeline

> Visual reference for Clap's value parsing and validation flow

This document provides detailed diagrams of how Clap processes argument values from raw strings to typed Rust values.

---

## Complete Parsing Pipeline

```
+===========================================================================+
|                    CLAP VALUE PARSING PIPELINE                             |
+===========================================================================+

    USER INPUT: myapp --port 8080 --config /path/to/file.toml
                           |                    |
                           v                    v
    +------------------------------------------------------------------+
    |   STAGE 1: LEXICAL ANALYSIS (clap_lex)                           |
    +------------------------------------------------------------------+
    |                                                                   |
    |   Input: ["myapp", "--port", "8080", "--config", "/path/..."]    |
    |                                                                   |
    |   +------------------+    +------------------+                    |
    |   | Long("port")     |    | Long("config")   |                   |
    |   | Value("8080")    |    | Value("/path/...") |                 |
    |   +------------------+    +------------------+                    |
    |                                                                   |
    +------------------------------------------------------------------+
                                    |
                                    v
    +------------------------------------------------------------------+
    |   STAGE 2: ARGUMENT MATCHING                                      |
    +------------------------------------------------------------------+
    |                                                                   |
    |   Match tokens to defined arguments:                              |
    |                                                                   |
    |   "--port" ---------> Arg::new("port")                           |
    |                         .value_parser(u16)                        |
    |                                                                   |
    |   "--config" -------> Arg::new("config")                         |
    |                         .value_parser(PathBuf)                    |
    |                                                                   |
    +------------------------------------------------------------------+
                                    |
                                    v
    +------------------------------------------------------------------+
    |   STAGE 3: VALUE PARSING                                          |
    +------------------------------------------------------------------+
    |                                                                   |
    |   "8080" -----> TypedValueParser<u16>                            |
    |                        |                                          |
    |            +-----------+-----------+                              |
    |            |                       |                              |
    |            v                       v                              |
    |       +--------+              +--------+                          |
    |       | Ok(T)  |              | Err(E) |                          |
    |       +--------+              +--------+                          |
    |       | u16    |              | "abc is not a valid number"       |
    |       | (8080) |              +--------+                          |
    |       +--------+                                                  |
    |                                                                   |
    +------------------------------------------------------------------+
                                    |
                                    v
    +------------------------------------------------------------------+
    |   STAGE 4: VALIDATION                                             |
    +------------------------------------------------------------------+
    |                                                                   |
    |   Post-parsing validation checks:                                 |
    |                                                                   |
    |   [x] Required arguments present?                                 |
    |   [x] Mutual exclusions respected?                                |
    |   [x] Value ranges valid?                                         |
    |   [x] Argument groups satisfied?                                  |
    |                                                                   |
    +------------------------------------------------------------------+
                                    |
                                    v
    +------------------------------------------------------------------+
    |   STAGE 5: RESULT CONSTRUCTION                                    |
    +------------------------------------------------------------------+
    |                                                                   |
    |   ArgMatches {                                                    |
    |       args: {                                                     |
    |           "port": MatchedArg { value: 8080u16, ... },            |
    |           "config": MatchedArg { value: PathBuf(...), ... },     |
    |       }                                                           |
    |   }                                                               |
    |                                                                   |
    +------------------------------------------------------------------+
```

**Diagram Description**: This diagram shows the five stages of Clap's parsing pipeline: lexical analysis (tokenization), argument matching, value parsing (type conversion), validation (constraint checking), and result construction.

---

## ValueParser Type Hierarchy

```
+===========================================================================+
|                    VALUEPARSER TYPE SYSTEM                                 |
+===========================================================================+

                         TypedValueParser<T>
                                 |
                 +---------------+---------------+
                 |               |               |
                 v               v               v
    +----------------+  +----------------+  +------------------+
    | Built-in       |  | Enum-based     |  | Custom           |
    | Parsers        |  | Parsers        |  | Parsers          |
    +----------------+  +----------------+  +------------------+
           |                   |                    |
           v                   v                    v
    +------------+      +-------------+     +---------------+
    | Primitives |      | ValueEnum   |     | fn(&str)      |
    | - String   |      |             |     |   -> Result   |
    | - PathBuf  |      | #[derive    |     |               |
    | - bool     |      |  (ValueEnum)]    | TypedValue    |
    | - u8..u128 |      | enum Format {    |   Parser      |
    | - i8..i128 |      |   Json,     |     |   trait       |
    | - f32, f64 |      |   Yaml,     |     |               |
    | - OsString |      | }           |     +---------------+
    +------------+      +-------------+           |
                                                  v
                                          +---------------+
                                          | Examples:     |
                                          | - Duration    |
                                          | - SemVer      |
                                          | - URL         |
                                          | - Regex       |
                                          +---------------+


    PARSER SELECTION FLOW
    =====================

    #[arg(value_parser = ...)]
              |
              v
    +--------------------+
    | Is it a built-in   |----YES----> Use built-in parser
    | type (u32, String)?|
    +--------------------+
              |
              NO
              v
    +--------------------+
    | Does type impl     |----YES----> Use ValueEnum parser
    | ValueEnum?         |
    +--------------------+
              |
              NO
              v
    +--------------------+
    | Does type impl     |----YES----> Use FromStr parser
    | FromStr?           |
    +--------------------+
              |
              NO
              v
    +--------------------+
    | Provide custom     |-----------> Implement TypedValueParser
    | value_parser       |             or use closure
    +--------------------+
```

**Diagram Description**: This diagram illustrates Clap's ValueParser type hierarchy, showing how built-in parsers, enum-based parsers, and custom parsers relate to each other, along with the decision flow for parser selection.

---

## Multi-Value Parsing Patterns

```
+===========================================================================+
|                    MULTI-VALUE ARGUMENT PATTERNS                           |
+===========================================================================+

    PATTERN 1: Multiple Occurrences (Vec<T>)
    =========================================

    Input: --file a.txt --file b.txt --file c.txt
                   |           |           |
                   v           v           v
              +---------------------------------+
              |   ArgAction::Append             |
              +---------------------------------+
                            |
                            v
              +---------------------------+
              | Vec<PathBuf> [            |
              |   PathBuf("a.txt"),       |
              |   PathBuf("b.txt"),       |
              |   PathBuf("c.txt"),       |
              | ]                         |
              +---------------------------+


    PATTERN 2: Delimited Values
    ============================

    Input: --tags rust,cli,parsing
                       |
                       v
              +-------------------+
              | Custom parser:    |
              | split by ','      |
              +-------------------+
                       |
                       v
              +---------------------------+
              | Vec<String> [             |
              |   "rust", "cli", "parsing"|
              | ]                         |
              +---------------------------+


    PATTERN 3: Key-Value Pairs
    ===========================

    Input: --env KEY1=value1 --env KEY2=value2
                    |                  |
                    v                  v
              +-------------------------------------+
              | Custom parser: split by '='        |
              +-------------------------------------+
                            |
                            v
              +---------------------------+
              | Vec<(String, String)> [   |
              |   ("KEY1", "value1"),     |
              |   ("KEY2", "value2"),     |
              | ]                         |
              +---------------------------+


    PATTERN 4: Positional with Trailing
    ====================================

    Input: myapp process file1.txt file2.txt file3.txt
                            |           |           |
                            v           v           v
              +--------------------------------------+
              | #[arg(trailing_var_arg = true)]     |
              +--------------------------------------+
                            |
                            v
              +---------------------------+
              | files: Vec<PathBuf>       |
              +---------------------------+
```

**Diagram Description**: This diagram demonstrates four common patterns for parsing multiple values in Clap: repeated flag occurrences, delimiter-separated values, key-value pairs, and trailing variadic arguments.

---

## Error Flow in Parsing

```
+===========================================================================+
|                    PARSING ERROR FLOW                                      |
+===========================================================================+

    Raw Input: "--port invalid"
                    |
                    v
    +--------------------------------+
    | Lexer: Long("port")            |
    |        Value("invalid")        |
    +--------------------------------+
                    |
                    v
    +--------------------------------+
    | Match: Arg::new("port")        |
    |        .value_parser(u16)      |
    +--------------------------------+
                    |
                    v
    +--------------------------------+
    | Parse: "invalid".parse::<u16>()|
    |        = Err(ParseIntError)    |
    +--------------------------------+
                    |
                    v
    +------------------------------------------+
    | Error Construction                        |
    +------------------------------------------+
    |                                           |
    |  clap::Error {                           |
    |      kind: ErrorKind::InvalidValue,      |
    |      info: [                              |
    |          ("invalid_arg", "--port"),       |
    |          ("invalid_value", "invalid"),    |
    |      ],                                   |
    |  }                                        |
    |                                           |
    +------------------------------------------+
                    |
                    v
    +------------------------------------------+
    | Format for Display                        |
    +------------------------------------------+
    |                                           |
    |  error: invalid value 'invalid' for      |
    |         '--port <PORT>'                   |
    |                                           |
    |    For more information, try '--help'.   |
    |                                           |
    +------------------------------------------+
                    |
                    v
    +------------------------------------------+
    | Exit with code 2 (usage error)           |
    +------------------------------------------+
```

**Diagram Description**: This diagram traces the flow of a parsing error from invalid input through error construction and formatting to user output, showing how Clap transforms parse failures into helpful error messages.

---

## Cross-Reference

- For overall architecture, see [architecture-overview.md](./architecture-overview.md)
- For error handling strategies, see [error-recovery.md](./error-recovery.md)
- For testing value parsers, see [testing-strategy.md](./testing-strategy.md)

---

*This document is part of the Clap Architecture Book visual reference materials.*
