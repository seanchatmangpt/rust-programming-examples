# Error Handling and Recovery

> Visual reference for Clap's error handling system and recovery patterns

This document provides detailed diagrams of Clap's error architecture, decision trees for error handling, and recovery patterns.

---

## Error Type Hierarchy

```
+===========================================================================+
|                    CLAP ERROR TYPE HIERARCHY                               |
+===========================================================================+

                            clap::Error
                                 |
                 +---------------+---------------+
                 |                               |
                 v                               v
    +------------------------+     +------------------------+
    |    ErrorKind           |     |    Error Context       |
    |    (enum)              |     |    (metadata)          |
    +------------------------+     +------------------------+
             |                              |
             |                              v
             |                 +------------------------+
             |                 | - Invalid argument     |
             |                 | - Expected value       |
             |                 | - Possible values      |
             |                 | - Suggestion           |
             |                 +------------------------+
             |
             +------------------+------------------+
             |                  |                  |
             v                  v                  v
    +----------------+  +----------------+  +------------------+
    | Usage Errors   |  | Parse Errors   |  | Display Requests |
    +----------------+  +----------------+  +------------------+
    |                |  |                |  |                  |
    | MissingRequired|  | InvalidValue   |  | DisplayHelp      |
    | UnknownArgument|  | InvalidSubcmd  |  | DisplayVersion   |
    | ArgumentConflict| | ValueValidation|  | DisplayHelpOnMiss|
    | WrongNumberOfVal| | UnknownArgument|  |                  |
    +----------------+  +----------------+  +------------------+
           |                   |                    |
           v                   v                    v
    +--------------+    +--------------+    +--------------+
    | Exit code: 2 |    | Exit code: 2 |    | Exit code: 0 |
    +--------------+    +--------------+    +--------------+
```

**Diagram Description**: This hierarchy shows how Clap categorizes errors into usage errors (wrong invocation), parse errors (invalid values), and display requests (help/version). Each category has specific error kinds and appropriate exit codes.

---

## Error Handling Decision Tree

```
+===========================================================================+
|                    ERROR HANDLING DECISION TREE                            |
+===========================================================================+

                        Cli::try_parse()
                              |
                              v
                    +------------------+
                    | Result<T, Error> |
                    +------------------+
                              |
              +---------------+---------------+
              |                               |
              v                               v
        +----------+                   +------------+
        |  Ok(cli) |                   | Err(error) |
        +----------+                   +------------+
              |                               |
              v                               v
    +-------------------+          +--------------------+
    | Proceed with      |          | Check error.kind() |
    | application logic |          +--------------------+
    +-------------------+                    |
                              +--------------+--------------+
                              |              |              |
                              v              v              v
                   +--------------+  +---------------+  +-------------+
                   | DisplayHelp  |  | DisplayVersion|  | Other kinds |
                   | DisplayHelpOn|  +---------------+  +-------------+
                   +--------------+         |                  |
                          |                 v                  v
                          v          +-----------+     +-----------------+
                   +-----------+     | Print     |     | Handle based on |
                   | Print     |     | version   |     | error category  |
                   | help text |     | and exit  |     +-----------------+
                   | and exit  |     | (code 0)  |            |
                   | (code 0)  |     +-----------+            |
                   +-----------+                              |
                                                              v
                                      +----------------------------------------+
                                      |                                        |
                                      v                                        v
                             +------------------+                    +------------------+
                             | Recoverable?     |                    | Non-recoverable  |
                             +------------------+                    +------------------+
                                      |                                        |
                             YES      |      NO                               |
                              +-------+-------+                               |
                              |               |                               |
                              v               v                               v
                    +----------------+  +----------------+          +------------------+
                    | Try fallback   |  | Show error     |          | error.exit()     |
                    | or default     |  | and suggest    |          | (print + exit 2) |
                    +----------------+  +----------------+          +------------------+
```

**Diagram Description**: This decision tree guides you through the process of handling Clap errors programmatically, showing how to differentiate between help/version requests (which should exit 0) and actual errors (which require handling or exit 2).

---

## Error Message Composition

```
+===========================================================================+
|                    ERROR MESSAGE ANATOMY                                   |
+===========================================================================+

    Complete error message structure:

    +------------------------------------------------------------------+
    |                                                                   |
    |  error: invalid value 'abc' for '--port <PORT>'                  |
    |  ^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^                     |
    |    |              |                                              |
    |  [Prefix]    [Main message]                                      |
    |  (red,bold)  (describes what went wrong)                         |
    |                                                                   |
    |    [possible values: 1024, 2048, 4096, 8080]                     |
    |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^                       |
    |              |                                                    |
    |    [Context: valid alternatives]                                 |
    |                                                                   |
    |    tip: a similar argument exists: '--ports'                     |
    |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^                    |
    |              |                                                    |
    |    [Suggestion: did-you-mean]                                    |
    |                                                                   |
    |  For more information, try '--help'.                             |
    |  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^                              |
    |              |                                                    |
    |    [Usage hint: next steps]                                      |
    |                                                                   |
    +------------------------------------------------------------------+


    COLOR SCHEME (when terminal supports)
    =====================================

    +----------------+------------------+
    | Element        | Color            |
    +----------------+------------------+
    | "error:"       | Red, Bold        |
    | argument name  | Yellow           |
    | invalid value  | Yellow           |
    | suggestion     | Green            |
    | "tip:"         | Cyan             |
    +----------------+------------------+
```

**Diagram Description**: This diagram breaks down the anatomy of a Clap error message, showing how different parts (prefix, main message, context, suggestions, usage hints) combine to create helpful, actionable error output.

---

## Recovery Strategies

```
+===========================================================================+
|                    ERROR RECOVERY PATTERNS                                 |
+===========================================================================+

    PATTERN 1: Fallback to Defaults
    ================================

    +-------------------+       +-------------------+
    | Try parse value   |------>| Parse failed?     |
    +-------------------+       +-------------------+
                                        |
                                   YES  |  NO
                                +-------+-------+
                                |               |
                                v               v
                    +-------------------+  +------------------+
                    | Use default value |  | Use parsed value |
                    | with warning      |  +------------------+
                    +-------------------+


    PATTERN 2: Interactive Recovery
    ================================

                    +-------------------+
                    | Parse failed      |
                    +-------------------+
                            |
                            v
                    +-------------------+
                    | Is terminal?      |
                    +-------------------+
                            |
                       YES  |  NO
                    +-------+-------+
                    |               |
                    v               v
            +---------------+  +------------------+
            | Prompt user   |  | Exit with error  |
            | for valid     |  +------------------+
            | input         |
            +---------------+
                    |
                    v
            +---------------+
            | Retry parse   |
            +---------------+


    PATTERN 3: Graceful Degradation
    ================================

    +-------------------+
    | Optional feature  |
    | parse failed      |
    +-------------------+
            |
            v
    +-------------------+
    | Feature critical? |
    +-------------------+
            |
       YES  |  NO
    +-------+-------+
    |               |
    v               v
    +--------+  +-------------------+
    | Error  |  | Disable feature   |
    | exit   |  | and continue      |
    +--------+  | with warning      |
                +-------------------+


    PATTERN 4: Suggestion and Retry
    ================================

    +-------------------+       +-------------------+
    | Parse "--prot"    |------>| Unknown argument  |
    +-------------------+       +-------------------+
                                        |
                                        v
                                +-------------------+
                                | Find similar:     |
                                | "--port" (90%)    |
                                +-------------------+
                                        |
                                        v
                                +-------------------+
                                | "Did you mean     |
                                | '--port'?"        |
                                +-------------------+
```

**Diagram Description**: This diagram presents four common error recovery patterns: falling back to defaults, prompting for interactive input, gracefully degrading functionality, and offering suggestions based on fuzzy matching.

---

## Exit Code Standards

```
+===========================================================================+
|                    CLI EXIT CODE CONVENTIONS                               |
+===========================================================================+

    +--------+---------------------------+--------------------------------+
    | Code   | Meaning                   | Clap Behavior                  |
    +--------+---------------------------+--------------------------------+
    |   0    | Success                   | --help, --version              |
    +--------+---------------------------+--------------------------------+
    |   1    | General error             | Application errors             |
    +--------+---------------------------+--------------------------------+
    |   2    | Usage error               | Clap parse errors              |
    |        | (misuse of command)       | (invalid args, missing req'd)  |
    +--------+---------------------------+--------------------------------+
    |  64    | Command line usage error  | (EX_USAGE from sysexits.h)     |
    +--------+---------------------------+--------------------------------+
    |  65    | Data format error         | Input data incorrect           |
    +--------+---------------------------+--------------------------------+
    |  66    | Cannot open input         | File not found                 |
    +--------+---------------------------+--------------------------------+
    |  73    | Can't create output       | Output file error              |
    +--------+---------------------------+--------------------------------+
    |  74    | IO error                  | Generic I/O failure            |
    +--------+---------------------------+--------------------------------+
    | 126    | Command not executable    | Permission denied              |
    +--------+---------------------------+--------------------------------+
    | 127    | Command not found         | Binary missing                 |
    +--------+---------------------------+--------------------------------+
    | 130    | Script terminated by      | User pressed Ctrl+C            |
    |        | Control-C                 | (128 + SIGINT(2))              |
    +--------+---------------------------+--------------------------------+


    DECISION FLOW FOR EXIT CODES
    ============================

                    +-------------------+
                    | Application state |
                    +-------------------+
                            |
            +---------------+---------------+
            |               |               |
            v               v               v
    +-----------+   +-----------+   +---------------+
    | Success   |   | User      |   | Error         |
    +-----------+   | requested |   +---------------+
         |          | exit      |          |
         v          +-----------+          |
    +--------+          |           +------+------+
    | Exit 0 |          v           |             |
    +--------+     +--------+       v             v
                   | Exit 0 |  +---------+  +-----------+
                   +--------+  | Clap    |  | App error |
                               | error   |  +-----------+
                               +---------+        |
                                    |             v
                                    v        +---------+
                               +--------+    | Exit 1  |
                               | Exit 2 |    | or      |
                               +--------+    | 65-78   |
                                             +---------+
```

**Diagram Description**: This diagram documents standard CLI exit codes and how Clap uses them, providing a decision flow for choosing appropriate exit codes in your application.

---

## Cross-Reference

- For architecture overview, see [architecture-overview.md](./architecture-overview.md)
- For parsing pipeline, see [parsing-pipeline.md](./parsing-pipeline.md)
- For testing error paths, see [testing-strategy.md](./testing-strategy.md)

---

*This document is part of the Clap Architecture Book visual reference materials.*
