# Performance Refactoring Patterns

Performance optimization during framework refactoring requires a disciplined approach that balances speed improvements against code maintainability and correctness. This chapter explores the tools, techniques, and patterns that enable you to refactor Rust frameworks while maintaining or improving performance characteristics.

## Introduction: Why Performance Matters in Framework Refactoring

Framework code sits at the foundation of countless applications. A 10% slowdown in your CLI parsing library affects thousands of downstream projects. A memory allocation pattern that seems harmless in isolation becomes catastrophic when multiplied across millions of invocations.

When refactoring frameworks, performance concerns manifest in several critical areas:

**Startup Time**: CLI tools and short-lived applications spend proportionally more time in initialization. A framework that adds 50ms of startup overhead transforms a snappy tool into a sluggish one.

**Memory Footprint**: Frameworks often instantiate data structures eagerly. Poor memory layout decisions compound as applications scale, leading to cache thrashing and excessive page faults.

**Compile Time**: Generic-heavy frameworks push work to compile time. While this can improve runtime performance, it degrades developer experience when compile times balloon.

**Binary Size**: Each generic instantiation and inline function contributes to binary bloat. Framework design choices directly impact deployment artifacts.

Common bottlenecks during refactoring include:

- Inadvertent allocation in hot paths
- Loss of inlining opportunities across new abstraction boundaries
- Hash map operations where perfect hashing could suffice
- String processing that allocates unnecessarily
- Trait objects introducing dynamic dispatch where static dispatch sufficed

The key insight is that refactoring changes code structure, and structure determines performance. Understanding this relationship allows you to refactor confidently while preserving the performance characteristics your users depend on.

## Profiling and Measurement

Before optimizing, you must measure. Rust's ecosystem provides excellent profiling tools that integrate seamlessly into development workflows.

### Cargo Flamegraph

Flamegraphs visualize where your program spends time, making hotspots immediately visible:

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph for your binary
cargo flamegraph --bin my_framework -- --example-args

# For tests or benchmarks
cargo flamegraph --test integration_tests
```

Flamegraphs aggregate stack traces, showing function call hierarchies with width proportional to time spent. When refactoring, generate flamegraphs before and after changes to visualize performance shifts.

### Criterion for Microbenchmarks

Criterion provides statistically rigorous benchmarking with automatic regression detection:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn parse_arguments_benchmark(c: &mut Criterion) {
    let args = vec!["program", "subcommand", "--flag", "value"];

    c.bench_function("parse_simple_args", |b| {
        b.iter(|| {
            let result = parse_args(black_box(&args));
            black_box(result)
        })
    });
}

fn parse_complex_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");

    for size in [10, 100, 1000].iter() {
        let args = generate_args(*size);
        group.bench_with_input(
            criterion::BenchmarkId::new("complex_args", size),
            &args,
            |b, args| b.iter(|| parse_args(black_box(args))),
        );
    }
    group.finish();
}

criterion_group!(benches, parse_arguments_benchmark, parse_complex_benchmark);
criterion_main!(benches);
```

Run benchmarks with `cargo bench` and examine the generated HTML reports in `target/criterion/`.

### System Profilers

For deeper analysis, system profilers provide hardware-level insights:

```bash
# Linux perf for CPU profiling
perf record --call-graph dwarf ./target/release/my_binary
perf report

# Memory profiling with heaptrack
heaptrack ./target/release/my_binary
heaptrack_gui heaptrack.my_binary.*.gz

# Cache analysis with Cachegrind
valgrind --tool=cachegrind ./target/release/my_binary
cg_annotate cachegrind.out.*
```

### Metrics to Track

Establish baseline metrics before refactoring:

| Metric | Tool | Target |
|--------|------|--------|
| Throughput | Criterion | Operations/second |
| Latency | Criterion | p50, p99, p999 |
| Memory peak | heaptrack | Maximum RSS |
| Allocations | heaptrack | Count and size |
| Cache misses | Cachegrind | L1, L2, L3 miss rates |
| Binary size | `ls -la` | Bytes |
| Compile time | `cargo build --timings` | Seconds |

Track these metrics in CI to catch regressions early:

```yaml
# .github/workflows/bench.yml
- name: Run benchmarks
  run: cargo bench -- --save-baseline refactor-branch

- name: Compare against main
  run: cargo bench -- --baseline main-branch
```

## Common Refactoring Patterns

### Compile-Time Computation with Macros

Moving computation from runtime to compile time eliminates overhead entirely. Declarative macros excel at generating static data:

```rust
/// Generate a static lookup table at compile time
macro_rules! define_command_table {
    ($($name:ident => $handler:expr),* $(,)?) => {
        const COMMAND_COUNT: usize = {
            let mut count = 0;
            $(
                let _ = stringify!($name);
                count += 1;
            )*
            count
        };

        static COMMANDS: [(&str, fn(&Args) -> Result<()>); COMMAND_COUNT] = [
            $(
                (stringify!($name), $handler),
            )*
        ];

        fn lookup_command(name: &str) -> Option<fn(&Args) -> Result<()>> {
            COMMANDS.iter()
                .find(|(n, _)| *n == name)
                .map(|(_, handler)| *handler)
        }
    };
}

define_command_table! {
    build => handle_build,
    test => handle_test,
    run => handle_run,
    clean => handle_clean,
}
```

### Memory Layout Optimization

Cache locality dramatically impacts performance. Structure your data for sequential access:

```rust
// Poor cache locality: Array of Structs (AoS)
struct CommandAoS {
    name: String,           // 24 bytes
    description: String,    // 24 bytes
    handler: fn() -> (),    // 8 bytes
    flags: u32,             // 4 bytes
    _padding: [u8; 4],      // 4 bytes (compiler-inserted)
}
// Total: 64 bytes per command, fields accessed together scattered

// Better cache locality: Struct of Arrays (SoA)
struct CommandsSoA {
    names: Vec<String>,
    descriptions: Vec<String>,
    handlers: Vec<fn() -> ()>,
    flags: Vec<u32>,
}

impl CommandsSoA {
    fn lookup_by_name(&self, target: &str) -> Option<usize> {
        // Sequential memory access through names array
        self.names.iter().position(|n| n == target)
    }

    fn get_handler(&self, idx: usize) -> fn() -> () {
        self.handlers[idx]
    }
}
```

### Perfect Hashing and Static Data Structures

When your set of keys is known at compile time, perfect hashing eliminates collision handling:

```rust
use phf::phf_map;

/// Zero-collision lookup table generated at compile time
static KEYWORDS: phf::Map<&'static str, Keyword> = phf_map! {
    "if" => Keyword::If,
    "else" => Keyword::Else,
    "while" => Keyword::While,
    "for" => Keyword::For,
    "fn" => Keyword::Fn,
    "let" => Keyword::Let,
    "mut" => Keyword::Mut,
    "const" => Keyword::Const,
};

pub fn classify_token(s: &str) -> TokenKind {
    match KEYWORDS.get(s) {
        Some(kw) => TokenKind::Keyword(*kw),
        None => TokenKind::Identifier,
    }
}
```

The `phf` crate computes hash parameters at compile time, guaranteeing O(1) lookup with no runtime collision resolution.

## Benchmarking During Refactoring

Maintaining performance during refactoring requires continuous measurement. Establish a benchmark suite before beginning:

```bash
# Before starting refactoring work
cargo bench -- --save-baseline before-refactor

# After each significant change
cargo bench -- --baseline before-refactor

# Criterion will report regressions and improvements
```

## Best Practices

### When to Optimize

Optimize when:

- **Profiling identifies a bottleneck**: Data shows specific code is slow
- **Users report performance issues**: Real-world usage reveals problems
- **Performance is a documented requirement**: API contracts specify latency bounds
- **Framework position demands it**: Hot paths in widely-used code

### When Not to Optimize

Avoid premature optimization when:

- **Clarity suffers significantly**: Unreadable code has maintenance costs
- **Measurements don't support it**: Intuition often misleads
- **Flexibility is needed**: Optimization often reduces abstraction
- **Development velocity matters more**: Ship first, optimize later

### Optimization Checklist

Before committing an optimization:

- [ ] Baseline benchmark exists
- [ ] Improvement is measurable and significant
- [ ] Code includes comments explaining the optimization
- [ ] Edge cases are tested
- [ ] Optimization doesn't break public API
- [ ] Performance regression tests are added
- [ ] Documentation updated if behavior changes

## Summary

Performance optimization during framework refactoring demands rigor: measure before and after, understand the trade-offs, and document your decisions. The patterns in this chapter, from compile-time computation to cache-conscious data structures, provide a toolkit for maintaining performance through architectural changes.

Remember that performance is a feature, but not the only feature. Balance optimization against code clarity, development velocity, and long-term maintainability. When in doubt, prefer simple correct code and optimize only what measurements prove necessary.
