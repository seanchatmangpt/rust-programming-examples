# Performance Optimization

> **Chapter 19** | Part 4: Real-World Systems | Estimated reading time: 25 minutes

CLI tools are called frequently in scripts, automation pipelines, and development workflows. A 200ms startup penalty that seems imperceptible in isolation becomes frustrating when a script invokes the tool hundreds of times. This chapter covers optimization techniques for startup time, memory footprint, and binary size, with real benchmarks and before/after comparisons from production CLI tools.

## Startup Time Optimization

### Measuring Baseline Performance

Before optimizing, establish accurate measurements. Use `hyperfine` for statistical rigor.

```bash
# Install hyperfine
cargo install hyperfine

# Measure cold start (first run after cache clear)
hyperfine --warmup 0 --runs 50 './target/release/myapp --version'

# Measure warm start (subsequent runs)
hyperfine --warmup 3 --runs 100 './target/release/myapp --version'

# Compare with alternatives
hyperfine --warmup 3 \
  './target/release/myapp --help' \
  './target/release/myapp-old --help' \
  'python3 myapp.py --help'

# Detailed timing with shell overhead removed
hyperfine --shell=none './target/release/myapp --version'
```

**Baseline measurement for a typical CLI** (before optimization):

| Metric | Time | Notes |
|--------|------|-------|
| Cold start (`--version`) | 45ms | First run after build |
| Warm start (`--version`) | 28ms | Page cache populated |
| `--help` | 52ms | Full help generation |
| Simple subcommand | 85ms | Minimal I/O |
| Complex subcommand | 180ms | Config + network init |

### Identifying Bottlenecks

Use `perf` or `samply` to find where time is spent:

```bash
# Linux perf
perf record --call-graph dwarf ./target/release/myapp --version
perf report

# macOS/Linux with samply
cargo install samply
samply record ./target/release/myapp --version
# Opens interactive flamegraph in browser
```

**Common bottlenecks we found**:

1. **Dynamic linker** (5-15ms): Loading shared libraries
2. **Argument parsing** (2-8ms): Clap initialization
3. **Configuration loading** (10-50ms): Reading config files
4. **Plugin discovery** (20-100ms): Scanning directories
5. **Network initialization** (50-200ms): TLS handshakes

### Lazy Initialization Patterns

Defer expensive initialization until actually needed.

```rust
use std::sync::OnceLock;
use std::path::PathBuf;

/// Application context with lazy-initialized resources
pub struct AppContext {
    config_path: PathBuf,
    config: OnceLock<Config>,
    database: OnceLock<Database>,
    http_client: OnceLock<reqwest::Client>,
}

impl AppContext {
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            config: OnceLock::new(),
            database: OnceLock::new(),
            http_client: OnceLock::new(),
        }
    }

    /// Config is only loaded when first accessed
    pub fn config(&self) -> &Config {
        self.config.get_or_init(|| {
            Config::load(&self.config_path)
                .expect("Failed to load configuration")
        })
    }

    /// Database connection is only established when needed
    pub fn database(&self) -> &Database {
        self.database.get_or_init(|| {
            Database::connect(&self.config().database_url)
                .expect("Failed to connect to database")
        })
    }

    /// HTTP client initialization (TLS setup is expensive)
    pub fn http_client(&self) -> &reqwest::Client {
        self.http_client.get_or_init(|| {
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client")
        })
    }
}

// Commands that don't need database never pay the cost
fn cmd_version(ctx: &AppContext) -> Result<()> {
    // Does NOT trigger config, database, or HTTP client loading
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    Ok(())
}

fn cmd_status(ctx: &AppContext) -> Result<()> {
    // Only loads config
    println!("Config: {:?}", ctx.config().environment);
    Ok(())
}

fn cmd_sync(ctx: &AppContext) -> Result<()> {
    // Loads config, database, AND HTTP client
    let data = ctx.http_client().get(&ctx.config().api_url).send()?;
    ctx.database().insert(data)?;
    Ok(())
}
```

**Performance impact of lazy initialization**:

| Command | Before | After | Improvement |
|---------|--------|-------|-------------|
| `--version` | 45ms | 8ms | 82% faster |
| `--help` | 52ms | 12ms | 77% faster |
| `status` | 85ms | 35ms | 59% faster |
| `sync` | 180ms | 175ms | 3% faster (still needs resources) |

### Conditional Subcommand Loading

For CLIs with many subcommands, avoid building the full command tree until needed.

```rust
use clap::{Command, Arg, ArgMatches};

/// Build minimal CLI for fast common paths
fn build_cli() -> Command {
    let mut cmd = Command::new("myapp")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Fast CLI tool")
        .subcommand_required(true)
        .arg_required_else_help(true);

    // Peek at first argument to decide what to load
    let first_arg = std::env::args().nth(1);

    match first_arg.as_deref() {
        Some("--version") | Some("-V") => {
            // Already handled by clap, no extra subcommands needed
        }
        Some("--help") | Some("-h") | None => {
            // Load all subcommands for complete help
            cmd = add_all_subcommands(cmd);
        }
        Some("simple") => {
            // Only add the requested subcommand
            cmd = cmd.subcommand(build_simple_subcommand());
        }
        Some("complex") => {
            // Complex command needs its own expensive setup
            cmd = cmd.subcommand(build_complex_subcommand());
        }
        Some(_) => {
            // Unknown - load everything for error message
            cmd = add_all_subcommands(cmd);
        }
    }

    cmd
}

fn build_simple_subcommand() -> Command {
    Command::new("simple")
        .about("Quick operation")
        .arg(Arg::new("input").required(true))
}

fn build_complex_subcommand() -> Command {
    // This includes expensive validation, many options, etc.
    Command::new("complex")
        .about("Complex operation with many options")
        .arg(Arg::new("config").long("config").value_name("FILE"))
        .arg(Arg::new("format").long("format").value_parser(["json", "yaml", "toml"]))
        // ... 20 more arguments
        .after_help(include_str!("complex_help.txt")) // Embedded help text
}
```

### Profile-Guided Optimization (PGO)

PGO can provide 10-20% improvement for CLI tools by optimizing hot paths.

```bash
#!/bin/bash
# pgo-build.sh - Profile-guided optimization build script

set -e

# Step 1: Build with instrumentation
echo "Building with instrumentation..."
RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data" \
    cargo build --release

# Step 2: Run representative workloads
echo "Collecting profile data..."
rm -rf /tmp/pgo-data
mkdir -p /tmp/pgo-data

# Run common operations many times
for i in {1..100}; do
    ./target/release/myapp --version > /dev/null
    ./target/release/myapp --help > /dev/null
    ./target/release/myapp status > /dev/null
    ./target/release/myapp simple input.txt > /dev/null
done

# Step 3: Merge profile data
echo "Merging profile data..."
llvm-profdata merge -o /tmp/pgo-data/merged.profdata /tmp/pgo-data

# Step 4: Build with profile data
echo "Building optimized binary..."
RUSTFLAGS="-Cprofile-use=/tmp/pgo-data/merged.profdata -Cllvm-args=-pgo-warn-missing-function" \
    cargo build --release

echo "PGO build complete!"
ls -la target/release/myapp
```

**PGO benchmark results**:

| Operation | Standard Release | With PGO | Improvement |
|-----------|-----------------|----------|-------------|
| `--version` | 8ms | 6ms | 25% |
| `--help` | 12ms | 9ms | 25% |
| `status` | 35ms | 28ms | 20% |
| `sync` | 175ms | 155ms | 11% |

## Binary Size Reduction

### Cargo.toml Release Profile

Configure release builds for size optimization:

```toml
[profile.release]
# Size optimization (alternative: opt-level = 3 for speed)
opt-level = "z"  # Optimize for size

# Link-time optimization - crucial for size
lto = true

# Single codegen unit - better optimization, slower build
codegen-units = 1

# Abort on panic - smaller than unwinding
panic = "abort"

# Strip symbols - significant size reduction
strip = true

[profile.release-small]
# Even more aggressive size optimization
inherits = "release"
opt-level = "z"
lto = "fat"
codegen-units = 1
panic = "abort"
strip = "symbols"
```

**Size comparison** (real production CLI):

| Profile | Binary Size | Notes |
|---------|-------------|-------|
| Debug | 45 MB | Full debug info |
| Release (default) | 12 MB | Basic optimization |
| Release + LTO | 8.2 MB | Link-time optimization |
| Release + LTO + strip | 5.1 MB | Symbols removed |
| Release-small | 3.8 MB | Maximum size reduction |

### Feature Selection

Clap and dependencies have optional features. Only enable what you need.

```toml
[dependencies]
# Minimal clap - derive only
clap = { version = "4.5", default-features = false, features = ["derive", "std", "help"] }

# Compare to full clap
# clap = { version = "4.5", features = ["derive"] }  # Includes color, suggestions, etc.

# Feature comparison for dependencies
serde = { version = "1.0", default-features = false, features = ["derive"] }
tokio = { version = "1", default-features = false, features = ["rt", "macros"] }
reqwest = { version = "0.11", default-features = false, features = ["rustls-tls", "json"] }
```

**Feature flag impact on binary size**:

| Configuration | Size | Delta |
|--------------|------|-------|
| clap full features | 5.8 MB | baseline |
| clap minimal (`derive`, `std`) | 4.9 MB | -15% |
| + serde minimal | 4.2 MB | -28% |
| + tokio minimal | 3.6 MB | -38% |
| + reqwest rustls (not openssl) | 3.1 MB | -47% |

### Dependency Auditing

Use `cargo-bloat` to identify size contributors:

```bash
# Install
cargo install cargo-bloat

# Analyze by crate
cargo bloat --release --crates

# Sample output:
# File  .text     Size  Crate
# 25.2% 31.5%  1.8MiB  std
# 15.8% 19.8%  1.1MiB  clap
#  8.3% 10.4%  598KiB  regex
#  7.1%  8.9%  512KiB  serde_json
#  5.2%  6.5%  374KiB  tokio
# ...

# Analyze by function
cargo bloat --release -n 20

# Compare two builds
cargo bloat --release --crates > before.txt
# Make changes...
cargo bloat --release --crates > after.txt
diff before.txt after.txt
```

**Size reduction strategies from bloat analysis**:

1. **Replace regex with simple matching**: -598KB
2. **Use `miniserde` instead of full serde**: -300KB
3. **Remove unused clap features**: -200KB
4. **Static vs dynamic TLS (rustls vs openssl)**: -800KB

### Strip and Compress

Post-build size reduction:

```bash
# Strip symbols (if not done in Cargo.toml)
strip target/release/myapp

# UPX compression (if distribution size matters more than startup)
upx --best target/release/myapp

# Size comparison
ls -lh target/release/myapp*
# -rwxr-xr-x 1 user user 3.8M myapp          (stripped)
# -rwxr-xr-x 1 user user 1.4M myapp.upx      (compressed)
```

**UPX tradeoff**:
- Compressed binary: 1.4 MB (63% smaller)
- Decompression overhead: +15-30ms startup time
- Use only when distribution size is critical and startup time is acceptable

## Memory Footprint Reduction

### Measuring Memory Usage

```bash
# Peak memory with /usr/bin/time
/usr/bin/time -v ./target/release/myapp complex-operation 2>&1 | grep "Maximum resident"
# Maximum resident set size (kbytes): 45320

# Memory timeline with heaptrack
heaptrack ./target/release/myapp complex-operation
heaptrack_gui heaptrack.myapp.*.gz

# Valgrind massif for detailed heap analysis
valgrind --tool=massif ./target/release/myapp complex-operation
ms_print massif.out.*
```

### Efficient Argument Structures

Design argument structs to minimize allocations:

```rust
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

// BEFORE: String allocations for everything
#[derive(Parser)]
struct CliInefficient {
    #[arg(long)]
    format: String,  // Allocates string

    #[arg(long)]
    log_level: String,  // Allocates string

    #[arg(long)]
    output: String,  // Allocates string even for paths
}

// AFTER: Use enums and appropriate types
#[derive(Parser)]
struct CliEfficient {
    #[arg(long, value_enum, default_value_t = Format::Json)]
    format: Format,  // No allocation, stack-only

    #[arg(long, value_enum, default_value_t = LogLevel::Info)]
    log_level: LogLevel,  // No allocation

    #[arg(long)]
    output: PathBuf,  // Optimized for paths
}

#[derive(Clone, Copy, ValueEnum, Default)]
enum Format {
    #[default]
    Json,
    Yaml,
    Toml,
    Csv,
}

#[derive(Clone, Copy, ValueEnum, Default)]
enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}
```

**Memory comparison**:

| Approach | Heap Allocations | Total Allocated |
|----------|-----------------|-----------------|
| String-based | 12 | 2.4 KB |
| Enum + PathBuf | 3 | 0.8 KB |
| Improvement | 75% fewer | 67% less |

### Streaming vs Loading

For file processing, stream instead of loading entire files:

```rust
use std::io::{BufRead, BufReader, Write, BufWriter};
use std::fs::File;

// BEFORE: Load entire file into memory
fn process_file_inefficient(path: &Path) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(path)?;  // Entire file in memory
    let lines: Vec<String> = content.lines()
        .map(|l| process_line(l))
        .collect();  // All results in memory
    Ok(lines)
}

// AFTER: Stream processing
fn process_file_efficient(input: &Path, output: &Path) -> Result<u64> {
    let reader = BufReader::new(File::open(input)?);
    let mut writer = BufWriter::new(File::create(output)?);
    let mut count = 0u64;

    for line in reader.lines() {
        let processed = process_line(&line?);
        writeln!(writer, "{}", processed)?;
        count += 1;
    }

    writer.flush()?;
    Ok(count)
}
```

**Memory usage for 100MB file**:

| Approach | Peak Memory | Processing Time |
|----------|-------------|-----------------|
| Load all | 312 MB | 1.2s |
| Streaming | 8 MB | 0.9s |

## Benchmarking CLI Performance

### Criterion Benchmarks for Parsing

Add benchmarks to catch performance regressions:

```rust
// benches/parsing.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use myapp::Cli;
use clap::Parser;

fn bench_argument_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");

    // Benchmark simple invocation
    group.bench_function("version", |b| {
        b.iter(|| {
            Cli::try_parse_from(["myapp", "--version"])
        });
    });

    // Benchmark help (builds full help string)
    group.bench_function("help", |b| {
        b.iter(|| {
            Cli::try_parse_from(["myapp", "--help"])
        });
    });

    // Benchmark with various argument counts
    for arg_count in [1, 5, 10, 20].iter() {
        let args: Vec<String> = (0..*arg_count)
            .map(|i| format!("--option{}=value{}", i, i))
            .collect();

        group.bench_with_input(
            BenchmarkId::new("args", arg_count),
            &args,
            |b, args| {
                let full_args: Vec<&str> = std::iter::once("myapp")
                    .chain(args.iter().map(|s| s.as_str()))
                    .collect();
                b.iter(|| Cli::try_parse_from(&full_args));
            },
        );
    }

    group.finish();
}

fn bench_subcommand_dispatch(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch");

    let subcommands = ["simple", "complex", "status", "sync"];

    for subcmd in subcommands {
        group.bench_with_input(
            BenchmarkId::new("subcommand", subcmd),
            subcmd,
            |b, subcmd| {
                b.iter(|| {
                    Cli::try_parse_from(["myapp", subcmd])
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_argument_parsing, bench_subcommand_dispatch);
criterion_main!(benches);
```

### CI Performance Regression Detection

Add benchmarks to CI to catch regressions:

```yaml
# .github/workflows/benchmark.yml
name: Performance Benchmarks

on:
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Install hyperfine
        run: cargo install hyperfine

      - name: Build release
        run: cargo build --release

      - name: Run benchmarks
        run: |
          hyperfine --warmup 3 --export-json bench.json \
            './target/release/myapp --version' \
            './target/release/myapp --help' \
            './target/release/myapp status'

      - name: Check thresholds
        run: |
          # Fail if any command exceeds threshold
          python3 << 'EOF'
          import json
          import sys

          with open('bench.json') as f:
              data = json.load(f)

          thresholds = {
              '--version': 15,  # ms
              '--help': 25,
              'status': 50,
          }

          failed = False
          for result in data['results']:
              cmd = result['command'].split()[-1]
              mean_ms = result['mean'] * 1000

              if cmd in thresholds and mean_ms > thresholds[cmd]:
                  print(f"FAIL: {cmd} took {mean_ms:.1f}ms (threshold: {thresholds[cmd]}ms)")
                  failed = True
              else:
                  print(f"OK: {cmd} took {mean_ms:.1f}ms")

          sys.exit(1 if failed else 0)
          EOF
```

## Caching Strategies

### Configuration Caching

Cache parsed configuration to avoid repeated disk I/O:

```rust
use std::fs;
use std::path::Path;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CachedConfig {
    config: Config,
    source_mtime: SystemTime,
    cached_at: SystemTime,
}

pub fn load_config_cached(config_path: &Path) -> Result<Config> {
    let cache_path = get_cache_path(config_path);

    // Check if cache exists and is valid
    if let Ok(cached) = load_cache(&cache_path) {
        let source_mtime = fs::metadata(config_path)?.modified()?;

        if cached.source_mtime == source_mtime {
            // Cache hit - return immediately
            return Ok(cached.config);
        }
    }

    // Cache miss - load and parse config
    let config = Config::load(config_path)?;

    // Update cache
    let cached = CachedConfig {
        config: config.clone(),
        source_mtime: fs::metadata(config_path)?.modified()?,
        cached_at: SystemTime::now(),
    };
    save_cache(&cache_path, &cached)?;

    Ok(config)
}

fn get_cache_path(config_path: &Path) -> PathBuf {
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("myapp");

    let hash = blake3::hash(config_path.to_string_lossy().as_bytes());
    cache_dir.join(format!("config-{}.cache", hash.to_hex()))
}
```

**Caching impact**:

| Scenario | Time | Notes |
|----------|------|-------|
| Config parse (no cache) | 45ms | TOML parsing + validation |
| Cache read (valid) | 3ms | Binary deserialization |
| Cache read (stale) | 48ms | Reload + update cache |

### Completion Script Caching

Shell completion generation is expensive. Cache the result:

```rust
use clap::CommandFactory;
use clap_complete::{generate, Shell};

fn get_completions(shell: Shell) -> String {
    let cache_path = get_completion_cache_path(shell);
    let binary_mtime = get_binary_mtime();

    // Check cache
    if let Ok(cached) = std::fs::read_to_string(&cache_path) {
        if let Ok(cache_mtime) = std::fs::metadata(&cache_path).and_then(|m| m.modified()) {
            if cache_mtime >= binary_mtime {
                return cached;
            }
        }
    }

    // Generate completions
    let mut cmd = Cli::command();
    let mut buffer = Vec::new();
    generate(shell, &mut cmd, "myapp", &mut buffer);
    let completions = String::from_utf8(buffer).unwrap();

    // Cache for next time
    std::fs::create_dir_all(cache_path.parent().unwrap()).ok();
    std::fs::write(&cache_path, &completions).ok();

    completions
}
```

## Lessons Learned

### What Worked Well

1. **OnceLock Everywhere**: Lazy initialization with `OnceLock` eliminated startup costs for commands that don't need expensive resources. The `--version` command went from 45ms to 8ms.

2. **Feature Flag Discipline**: Carefully selecting dependency features reduced binary size by 47% without losing functionality.

3. **CI Benchmarks**: Catching performance regressions in PRs prevented shipping slow changes. We caught 3 regressions before release.

### What We Would Do Differently

1. **Measure First, Always**: We spent time optimizing the wrong things early on. Profile before optimizing.

2. **PGO from the Start**: Profile-guided optimization is free performance. We should have set it up in CI earlier.

3. **Streaming by Default**: We retrofitted streaming for large file handling. Designing for streaming from the start would have been easier.

### Performance Benchmarks Summary

**Final optimized CLI performance**:

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| `--version` cold | 45ms | 6ms | 87% |
| `--version` warm | 28ms | 4ms | 86% |
| `--help` | 52ms | 9ms | 83% |
| `status` command | 85ms | 28ms | 67% |
| `sync` command | 180ms | 155ms | 14% |
| Binary size | 12 MB | 3.1 MB | 74% |
| Peak memory (simple) | 45 MB | 12 MB | 73% |

**Optimization effort vs impact**:

| Technique | Effort | Impact | Recommendation |
|-----------|--------|--------|----------------|
| Lazy init (`OnceLock`) | Low | High | Always |
| Release profile tuning | Low | Medium | Always |
| Feature flag audit | Medium | High | Always |
| PGO | Medium | Medium | For release builds |
| Conditional subcommands | Medium | Medium | For large CLIs |
| UPX compression | Low | Mixed | Only for size-critical |

## Summary

CLI performance optimization is about understanding where time and resources go, then applying targeted techniques. The most impactful optimizations are often the simplest: lazy initialization, proper release profiles, and careful feature selection.

### Key Takeaways

1. **Measure before optimizing** with `hyperfine`, `perf`, and `cargo-bloat`
2. **Lazy initialization** with `OnceLock` for expensive resources
3. **Configure release profile** with LTO, strip, and appropriate opt-level
4. **Audit dependencies** for unused features and size contributors
5. **Profile-guided optimization** provides free 10-20% improvement
6. **Cache expensive computations** like config parsing and completions
7. **Add CI benchmarks** to prevent performance regressions

### Quick Reference: Optimization Checklist

```toml
# Cargo.toml release profile
[profile.release]
opt-level = "z"        # or "3" for speed over size
lto = true
codegen-units = 1
panic = "abort"
strip = true

# Dependency features
[dependencies]
clap = { version = "4.5", default-features = false, features = ["derive", "std"] }
```

```rust
// Lazy initialization pattern
use std::sync::OnceLock;
static EXPENSIVE: OnceLock<ExpensiveType> = OnceLock::new();
fn get_expensive() -> &'static ExpensiveType {
    EXPENSIVE.get_or_init(|| ExpensiveType::new())
}
```

```bash
# Measurement commands
hyperfine --warmup 3 './myapp --version'
cargo bloat --release --crates
/usr/bin/time -v ./myapp command
```

---

*Next: [API Quick Reference](../part5-reference/20-api-quick-reference.md)*
