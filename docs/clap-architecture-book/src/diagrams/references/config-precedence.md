# Configuration Precedence and Layering

> Visual reference for configuration source hierarchy and merging strategies

This document provides detailed diagrams of how configuration values are resolved from multiple sources in Clap-based applications.

---

## Standard Precedence Hierarchy

```
+===========================================================================+
|                    CONFIGURATION PRECEDENCE PYRAMID                        |
+===========================================================================+

                            HIGHEST PRIORITY
                                   ^
                                   |
                         +--------+--------+
                         |                 |
                         |   CLI ARGUMENTS |      <- Explicit user intent
                         |   --port 8080   |         Always wins
                         |                 |
                         +-----------------+
                                   |
                         +---------+---------+
                         |                   |
                         | ENVIRONMENT VARS  |    <- Deployment config
                         | MYAPP_PORT=9000   |       Runtime override
                         |                   |
                         +-------------------+
                                   |
                         +---------+-----------+
                         |                     |
                         |   LOCAL CONFIG      |  <- Project-specific
                         |   ./.myapp.toml     |     Developer override
                         |                     |
                         +---------------------+
                                   |
                         +---------+-------------+
                         |                       |
                         |    USER CONFIG        |  <- Personal defaults
                         | ~/.config/myapp/      |     User preferences
                         |    config.toml        |
                         |                       |
                         +-----------------------+
                                   |
                         +---------+---------------+
                         |                         |
                         |    SYSTEM CONFIG        |  <- Organization defaults
                         |  /etc/myapp/config.toml |     Shared settings
                         |                         |
                         +-------------------------+
                                   |
                         +---------+-----------------+
                         |                           |
                         |    BUILT-IN DEFAULTS      |  <- Last resort
                         |    (compiled in binary)   |     Always available
                         |                           |
                         +---------------------------+
                                   |
                                   v
                            LOWEST PRIORITY
```

**Diagram Description**: This pyramid shows the standard configuration precedence from highest to lowest priority. CLI arguments always take precedence, followed by environment variables, then config files (local, user, system), and finally built-in defaults.

---

## Configuration Merge Flow

```
+===========================================================================+
|                    CONFIGURATION MERGE ALGORITHM                           |
+===========================================================================+

    START: Empty configuration
    ===========================

    Config { port: None, host: None, debug: None }
         |
         v
    +--------------------------------+
    | LAYER 1: Built-in defaults     |
    +--------------------------------+
    |                                |
    | Config {                       |
    |     port: Some(3000),          |
    |     host: Some("127.0.0.1"),   |
    |     debug: Some(false),        |
    | }                              |
    +--------------------------------+
         |
         v
    +--------------------------------+
    | LAYER 2: /etc/myapp/config.toml|
    | (if exists)                    |
    +--------------------------------+
    |                                |
    | Only override if set:          |
    | port = 8080                    |
    |                                |
    | Result:                        |
    | Config {                       |
    |     port: Some(8080),    <---  |
    |     host: Some("127.0.0.1"),   |
    |     debug: Some(false),        |
    | }                              |
    +--------------------------------+
         |
         v
    +--------------------------------+
    | LAYER 3: ~/.config/myapp/      |
    | config.toml (if exists)        |
    +--------------------------------+
    |                                |
    | debug = true                   |
    |                                |
    | Result:                        |
    | Config {                       |
    |     port: Some(8080),          |
    |     host: Some("127.0.0.1"),   |
    |     debug: Some(true),   <---  |
    | }                              |
    +--------------------------------+
         |
         v
    +--------------------------------+
    | LAYER 4: ./.myapp.toml         |
    | (if exists)                    |
    +--------------------------------+
    |                                |
    | host = "0.0.0.0"               |
    |                                |
    | Result:                        |
    | Config {                       |
    |     port: Some(8080),          |
    |     host: Some("0.0.0.0"), <-- |
    |     debug: Some(true),         |
    | }                              |
    +--------------------------------+
         |
         v
    +--------------------------------+
    | LAYER 5: Environment vars      |
    | MYAPP_PORT=9000                |
    +--------------------------------+
    |                                |
    | Result:                        |
    | Config {                       |
    |     port: Some(9000),    <---  |
    |     host: Some("0.0.0.0"),     |
    |     debug: Some(true),         |
    | }                              |
    +--------------------------------+
         |
         v
    +--------------------------------+
    | LAYER 6: CLI arguments         |
    | --host localhost               |
    +--------------------------------+
    |                                |
    | Final Result:                  |
    | Config {                       |
    |     port: Some(9000),          |
    |     host: Some("localhost"), <-|
    |     debug: Some(true),         |
    | }                              |
    +--------------------------------+
```

**Diagram Description**: This diagram traces a configuration value through all six layers of the precedence hierarchy, showing how each layer can override values from lower layers while preserving unset fields.

---

## Option Types for Layering

```
+===========================================================================+
|                    CLI ARGUMENT TYPES FOR LAYERING                         |
+===========================================================================+

    When implementing layered configuration, CLI struct field types
    determine how layering works:

    TYPE 1: Direct type with default_value
    ======================================

    #[arg(long, default_value = "3000")]
    port: u16,

    Behavior:
    - Always has a value
    - Cannot detect if user provided it
    - Config file cannot override default

    +--------+     +--------+     +---------+
    | Config |     |  Env   |     |   CLI   |
    | port=X |     | PORT=Y |     | (none)  |
    +--------+     +--------+     +---------+
         |              |              |
         v              v              v
    +---------------------------------------+
    | Result: 3000 (default always wins)   |
    +---------------------------------------+
                     ^^^ PROBLEM!


    TYPE 2: Option<T> (RECOMMENDED for layering)
    =============================================

    #[arg(long)]
    port: Option<u16>,

    Behavior:
    - None means "not provided"
    - Config file can fill in if CLI didn't provide
    - Explicitly enables layering

    +--------+     +--------+     +---------+
    | Config |     |  Env   |     |   CLI   |
    | port=X |     | PORT=Y |     | (none)  |
    +--------+     +--------+     +---------+
         |              |              |
         v              v              v
    +---------------------------------------+
    | Check CLI: None                       |
    | Check Env: Some(Y)                    |
    | Result: Y                             |
    +---------------------------------------+
                     ^^^ CORRECT!


    TYPE 3: Option<Option<T>> (Advanced)
    =====================================

    #[arg(long)]
    debug: Option<Option<bool>>,

    Behavior:
    - None = not provided at all
    - Some(None) = provided without value (--debug)
    - Some(Some(true)) = provided with explicit value

    +-------------------+-------------------+
    | Input             | Result            |
    +-------------------+-------------------+
    | (nothing)         | None              |
    | --debug           | Some(None)        |
    | --debug=true      | Some(Some(true))  |
    | --debug=false     | Some(Some(false)) |
    +-------------------+-------------------+


    LAYERING DECISION TREE:
    =======================

                    +-------------------+
                    | Need layering?    |
                    +-------------------+
                            |
                       YES  |  NO
                    +-------+-------+
                    |               |
                    v               v
            +--------------+  +--------------+
            | Use Option<T>|  | Use T with   |
            |              |  | default_value|
            +--------------+  +--------------+
```

**Diagram Description**: This diagram explains how different CLI field types (direct types, Option<T>, Option<Option<T>>) affect configuration layering behavior, showing why Option<T> is recommended for layered configurations.

---

## Profile-Based Configuration

```
+===========================================================================+
|                    PROFILE-BASED CONFIGURATION                             |
+===========================================================================+

    CONFIG FILE STRUCTURE:
    ======================

    # config.toml

    [default]                    <- Base profile (always loaded)
    log_level = "info"
    port = 8080

    [default.database]
    pool_size = 10

    [development]                <- Inherits from default, overrides
    log_level = "debug"

    [development.database]
    pool_size = 2

    [production]                 <- Inherits from default, overrides
    log_level = "warn"

    [production.database]
    pool_size = 50


    PROFILE RESOLUTION:
    ===================

    CLI: myapp --profile development
                           |
                           v
    +---------------------------------------------+
    | 1. Load [default] profile                   |
    +---------------------------------------------+
    |   Config {                                  |
    |       log_level: "info",                    |
    |       port: 8080,                           |
    |       database: { pool_size: 10 }           |
    |   }                                         |
    +---------------------------------------------+
                           |
                           v
    +---------------------------------------------+
    | 2. Overlay [development] profile            |
    +---------------------------------------------+
    |   Config {                                  |
    |       log_level: "debug",  <- overridden    |
    |       port: 8080,          <- inherited     |
    |       database: { pool_size: 2 } <- override|
    |   }                                         |
    +---------------------------------------------+
                           |
                           v
    +---------------------------------------------+
    | 3. Apply environment variables              |
    +---------------------------------------------+
                           |
                           v
    +---------------------------------------------+
    | 4. Apply CLI arguments                      |
    +---------------------------------------------+


    PROFILE INHERITANCE:
    ====================

    [base]
       |
       +---> [development] extends base
       |         |
       |         +---> [local] extends development
       |
       +---> [production] extends base
                 |
                 +---> [staging] extends production
```

**Diagram Description**: This diagram shows how profile-based configuration works, with profiles inheriting from base configurations and layering on top of each other in a predictable order.

---

## Deep Merge vs Replace Strategies

```
+===========================================================================+
|                    MERGE STRATEGIES                                        |
+===========================================================================+

    Given two configurations:
    =========================

    Base config:                    Overlay config:
    +-------------------+           +-------------------+
    | server:           |           | server:           |
    |   host: localhost |           |   port: 9000      |
    |   port: 8080      |           | features:         |
    |   workers: 4      |           |   - new_feature   |
    | features:         |           +-------------------+
    |   - base_feature  |
    +-------------------+


    STRATEGY 1: REPLACE (Shallow)
    =============================

    The overlay completely replaces any key it contains.

    Result:
    +-------------------+
    | server:           | <- Entire 'server' replaced
    |   port: 9000      |    (host and workers lost!)
    | features:         | <- Entire 'features' replaced
    |   - new_feature   |    (base_feature lost!)
    +-------------------+


    STRATEGY 2: DEEP MERGE (Recommended)
    =====================================

    Recursively merge nested structures.

    Result:
    +-------------------+
    | server:           | <- Merged recursively
    |   host: localhost |    (preserved from base)
    |   port: 9000      |    (overridden)
    |   workers: 4      |    (preserved from base)
    | features:         | <- Arrays need special handling
    |   - base_feature  |
    |   - new_feature   |
    +-------------------+


    STRATEGY 3: APPEND (for arrays)
    ================================

    Combine array elements from both sources.

    Base features:    ["base_feature"]
    Overlay features: ["new_feature"]
    Result features:  ["base_feature", "new_feature"]


    DECISION MATRIX:
    ================

    +---------------+-------------+--------------+---------------+
    | Value Type    | Replace     | Deep Merge   | Append        |
    +---------------+-------------+--------------+---------------+
    | Scalar        | New value   | New value    | N/A           |
    +---------------+-------------+--------------+---------------+
    | Object/Map    | Entire new  | Recursive    | N/A           |
    |               | object      | merge        |               |
    +---------------+-------------+--------------+---------------+
    | Array/List    | Entire new  | Element-wise | Concatenate   |
    |               | array       | if indexed   |               |
    +---------------+-------------+--------------+---------------+
    | None/null     | Remove key  | Keep base    | Keep base     |
    +---------------+-------------+--------------+---------------+
```

**Diagram Description**: This diagram compares three merge strategies (replace, deep merge, append) and shows how they handle different value types when combining configurations from multiple sources.

---

## Cross-Reference

- For architecture overview, see [architecture-overview.md](./architecture-overview.md)
- For command lifecycle, see [command-lifecycle.md](./command-lifecycle.md)
- For error handling, see [error-recovery.md](./error-recovery.md)

---

*This document is part of the Clap Architecture Book visual reference materials.*
