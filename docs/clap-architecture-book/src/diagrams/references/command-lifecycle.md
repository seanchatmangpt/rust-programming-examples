# Command Execution Lifecycle

> Visual reference for the complete lifecycle of a Clap command from invocation to completion

This document provides detailed diagrams of command execution phases, routing patterns, and lifecycle hooks.

---

## Complete Command Lifecycle

```
+===========================================================================+
|                    COMMAND EXECUTION LIFECYCLE                             |
+===========================================================================+

    PHASE 1: INITIALIZATION
    =======================

    Binary Start
         |
         v
    +-------------------+
    | main() entry      |
    +-------------------+
         |
         v
    +-------------------+
    | Build Command     |
    | definition        |
    | (derive or        |
    |  builder)         |
    +-------------------+
         |
         v
    +-------------------+
    | Register          |
    | subcommands       |
    +-------------------+
         |
         v
    +-------------------+
    | Configure         |
    | global args       |
    +-------------------+


    PHASE 2: PARSING
    ================
         |
         v
    +-------------------+
    | Cli::parse() or   |
    | cmd.get_matches() |
    +-------------------+
         |
         v
    +-------------------+
    | Tokenize argv     |
    | (clap_lex)        |
    +-------------------+
         |
         v
    +-------------------+
    | Match arguments   |
    | to definitions    |
    +-------------------+
         |
         v
    +-------------------+
    | Parse values      |
    | (ValueParser)     |
    +-------------------+
         |
         v
    +-------------------+
    | Validate          |
    | constraints       |
    +-------------------+
         |
    +----+----+
    |         |
    v         v
 SUCCESS    ERROR
    |         |
    v         v
+--------+ +--------+
|ArgMatch| |Display |
|or Struct| |& exit |
+--------+ +--------+


    PHASE 3: DISPATCH
    =================
         |
         v
    +-------------------+
    | Route to          |
    | subcommand        |
    +-------------------+
         |
         v
    +-------------------+
    | Extract typed     |
    | values            |
    +-------------------+
         |
         v
    +-------------------+
    | Execute command   |
    | logic             |
    +-------------------+


    PHASE 4: COMPLETION
    ===================
         |
         v
    +-------------------+
    | Return Result     |
    +-------------------+
         |
    +----+----+
    |         |
    v         v
  Ok(())   Err(e)
    |         |
    v         v
+--------+ +--------+
| exit 0 | | exit N |
+--------+ +--------+
```

**Diagram Description**: This diagram shows the four main phases of command execution: initialization (building the command structure), parsing (processing arguments), dispatch (routing to handlers), and completion (returning results).

---

## Subcommand Routing Flow

```
+===========================================================================+
|                    SUBCOMMAND ROUTING PATTERNS                             |
+===========================================================================+

    INPUT: myapp config set key value
                   |     |    |    |
                   v     v    v    v
           +-------+-----+----+----+
           | Subcommand tree matching |
           +-------------------------+

    ROUTING TREE:
    =============

                        myapp (root)
                            |
            +---------------+---------------+
            |               |               |
            v               v               v
        +-------+       +-------+       +-------+
        | init  |       | config|       | run   |
        +-------+       +-------+       +-------+
                            |
                    +-------+-------+
                    |       |       |
                    v       v       v
                +-----+ +-----+ +------+
                | get | | set | | list |
                +-----+ +-----+ +------+
                            |
                            v
                    +---------------+
                    | Matched:      |
                    | config -> set |
                    +---------------+
                            |
                            v
                    +---------------+
                    | Positional:   |
                    | key = "key"   |
                    | value = "value"|
                    +---------------+


    CODE PATTERN (Match-based routing):
    ====================================

    match cli.command {
        Commands::Init { path } => {
            // Handle init
        }
        Commands::Config(config_cmd) => {
            match config_cmd {
                ConfigCommands::Get { key } => { ... }
                ConfigCommands::Set { key, value } => { ... }
                ConfigCommands::List => { ... }
            }
        }
        Commands::Run { target } => {
            // Handle run
        }
    }


    CODE PATTERN (Trait-based routing):
    ====================================

    trait Runnable {
        fn run(&self) -> Result<()>;
    }

    match cli.command {
        Commands::Init(cmd) => cmd.run(),
        Commands::Config(cmd) => cmd.run(),
        Commands::Run(cmd) => cmd.run(),
    }
```

**Diagram Description**: This diagram illustrates how Clap routes to nested subcommands through a tree structure, with two common code patterns for handling the routing (match-based and trait-based).

---

## Argument Resolution Order

```
+===========================================================================+
|                    ARGUMENT RESOLUTION ORDER                               |
+===========================================================================+

    When a command has multiple sources for an argument value,
    Clap resolves them in this order (highest to lowest priority):

    +------------------------------------------------------------------+
    |                                                                   |
    |  PRIORITY 1 (HIGHEST): Explicit CLI argument                     |
    |  ========================================================        |
    |  myapp --port 8080                                               |
    |              ^^^^^                                                |
    |  This value is used if provided                                  |
    |                                                                   |
    +------------------------------------------------------------------+
                                    |
                                    v (if not provided)
    +------------------------------------------------------------------+
    |                                                                   |
    |  PRIORITY 2: Environment variable                                 |
    |  ========================================================        |
    |  #[arg(env = "MYAPP_PORT")]                                      |
    |  MYAPP_PORT=9000 myapp                                           |
    |             ^^^^                                                  |
    |  Falls back to env var if arg not provided                       |
    |                                                                   |
    +------------------------------------------------------------------+
                                    |
                                    v (if env not set)
    +------------------------------------------------------------------+
    |                                                                   |
    |  PRIORITY 3: Default value                                        |
    |  ========================================================        |
    |  #[arg(default_value = "3000")]                                  |
    |                        ^^^^^^                                     |
    |  Falls back to default if neither arg nor env provided           |
    |                                                                   |
    +------------------------------------------------------------------+
                                    |
                                    v (if no default)
    +------------------------------------------------------------------+
    |                                                                   |
    |  PRIORITY 4: Required check                                       |
    |  ========================================================        |
    |  If no value from any source and arg is required:                |
    |  -> Error: "required argument not provided"                      |
    |                                                                   |
    |  If arg is optional (Option<T>):                                 |
    |  -> Value is None                                                 |
    |                                                                   |
    +------------------------------------------------------------------+


    RESOLUTION EXAMPLE:
    ===================

    #[derive(Parser)]
    struct Cli {
        #[arg(short, long, env = "PORT", default_value = "3000")]
        port: u16,
    }

    Scenario 1: myapp --port 8080        -> port = 8080 (CLI wins)
    Scenario 2: PORT=9000 myapp          -> port = 9000 (env var)
    Scenario 3: myapp                    -> port = 3000 (default)
    Scenario 4: PORT=9000 myapp -p 8080  -> port = 8080 (CLI wins)
```

**Diagram Description**: This diagram explains the priority order for resolving argument values in Clap: explicit CLI arguments take precedence over environment variables, which take precedence over default values.

---

## Global vs Local Arguments

```
+===========================================================================+
|                    GLOBAL vs LOCAL ARGUMENT SCOPE                          |
+===========================================================================+

    GLOBAL ARGUMENTS:
    =================

    #[derive(Parser)]
    struct Cli {
        #[arg(global = true, short, long)]
        verbose: bool,

        #[command(subcommand)]
        command: Commands,
    }

    These are equivalent:
    ---------------------
    myapp --verbose config set key value
    myapp config --verbose set key value
    myapp config set --verbose key value


    PROPAGATION DIAGRAM:
    ====================

                    +-------------+
                    |   myapp     |
                    | --verbose   | <-- Global defined here
                    +-------------+
                          |
            +-------------+-------------+
            |                           |
            v                           v
    +---------------+           +---------------+
    |    config     |           |     run       |
    |  (--verbose   |           |  (--verbose   |
    |   available)  |           |   available)  |
    +---------------+           +---------------+
            |
            +-------+-------+
            |               |
            v               v
    +----------+    +----------+
    |   get    |    |   set    |
    |(--verbose|    |(--verbose|
    | available)|   | available)|
    +----------+    +----------+


    LOCAL ARGUMENTS:
    ================

    #[derive(Subcommand)]
    enum Commands {
        Run {
            #[arg(long)]  // NOT global
            dry_run: bool,  // Only for 'run' subcommand
        },
    }

    Valid:   myapp run --dry-run
    Invalid: myapp --dry-run run
              ^^^^^^^^^^
              Error: unexpected argument


    SCOPE COMPARISON:
    =================

    +------------------+-------------------+---------------------+
    | Argument Type    | Position          | Available In        |
    +------------------+-------------------+---------------------+
    | Global           | Before or after   | All subcommands     |
    |                  | any subcommand    |                     |
    +------------------+-------------------+---------------------+
    | Root-level       | Before first      | Root command only   |
    | (not global)     | subcommand        |                     |
    +------------------+-------------------+---------------------+
    | Subcommand-local | After subcommand  | That subcommand     |
    |                  | name              | only                |
    +------------------+-------------------+---------------------+
```

**Diagram Description**: This diagram clarifies the difference between global arguments (available at any position and in all subcommands) and local arguments (only available within their specific scope).

---

## Execution Context Flow

```
+===========================================================================+
|                    EXECUTION CONTEXT PROPAGATION                           |
+===========================================================================+

    BUILDING CONTEXT:
    =================

    +-------------------+
    | Parse CLI args    |
    +-------------------+
            |
            v
    +-------------------+
    | Load config file  |
    | (if specified)    |
    +-------------------+
            |
            v
    +-------------------+
    | Read environment  |
    | variables         |
    +-------------------+
            |
            v
    +-------------------+
    | Apply defaults    |
    +-------------------+
            |
            v
    +-------------------+
    | Construct         |
    | AppContext        |
    +-------------------+


    CONTEXT STRUCTURE:
    ==================

    struct AppContext {
        // From CLI
        verbose: bool,
        output_format: Format,

        // From config/env
        api_endpoint: String,
        credentials: Option<Credentials>,

        // Runtime
        working_dir: PathBuf,
        logger: Logger,
    }


    PASSING CONTEXT TO COMMANDS:
    ============================

    fn main() -> Result<()> {
        let cli = Cli::parse();
        let ctx = AppContext::build(&cli)?;

        match cli.command {
            Commands::Fetch(cmd) => cmd.run(&ctx),
            Commands::Process(cmd) => cmd.run(&ctx),
            Commands::Upload(cmd) => cmd.run(&ctx),
        }
    }


    CONTEXT FLOW DIAGRAM:
    =====================

    +--------+     +--------+     +--------+
    |  CLI   |---->| Context|---->| Handler|
    | Parser |     | Builder|     |        |
    +--------+     +--------+     +--------+
        |              |              |
        v              v              v
    +---------+   +---------+   +---------+
    |ArgMatches|  |AppContext|  | Result  |
    +---------+   +---------+   +---------+
                       |
                       v
              +----------------+
              | Shared across  |
              | all commands:  |
              | - Logger       |
              | - HTTP client  |
              | - DB pool      |
              +----------------+
```

**Diagram Description**: This diagram shows how to build and propagate execution context through your CLI application, combining parsed arguments, configuration, and runtime resources into a shared context struct.

---

## Cross-Reference

- For architecture overview, see [architecture-overview.md](./architecture-overview.md)
- For configuration layering, see [config-precedence.md](./config-precedence.md)
- For testing commands, see [testing-strategy.md](./testing-strategy.md)

---

*This document is part of the Clap Architecture Book visual reference materials.*
