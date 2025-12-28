# Programming Rust: Code Examples

## Project Overview

This is the **official code examples repository** for "Programming Rust" by Jim Blandy, Jason Orendorff, and Leonora Tindall. It contains 25 complete, self-contained Rust projects organized by book chapter, demonstrating Rust language features, patterns, and best practices.

Each subdirectory is an independent Rust project with its own `Cargo.toml` file, buildable and testable in isolation.

**Repository**: https://github.com/ProgrammingRust/code-examples
**License**: MIT License (see LICENSE-MIT)

---

## Stack & Versions

- **Language**: Rust
- **Edition**: 2018 (all projects)
- **MSRV** (Minimum Supported Rust Version): 1.56+ (typical for code examples from circa 2021-2022)
- **Target Platform**: Linux, macOS, Windows (cross-platform compatible)

### Key Dependencies by Project Type

| Feature Area | Projects | Key Crates | Versions |
|--------------|----------|-----------|----------|
| Web Frameworks | actix-gcd | actix-web | 4.1+ |
| HTTP Clients | http-get | reqwest | 0.11 (with blocking) |
| Async Runtime | cheapo-request, many-requests | async-std | 1.7+ |
| FFI | libgit2-rs, libgit2-rs-safe | Manual C bindings | — |
| Macros | json-macro | (procedural macros) | — |
| Testing | fern_sim, spawn-blocking | — | — |

---

## Repository Map

```
rust-programming-examples/
├── README.md                          # Chapter-by-chapter guide
├── LICENSE-MIT                        # MIT license
├── CLAUDE.md                          # This file - AI assistant guide
│
├── gcd/                               # Ch. 2: Simple CLI program
├── actix-gcd/                         # Ch. 2: Web service (actix-web)
│
├── fern_sim/                          # Ch. 8: Module structure
│   ├── src/lib.rs
│   ├── src/simulation.rs
│   ├── src/spores.rs
│   └── tests/
│
├── queue/                             # Ch. 9: Basic struct type
├── generic-queue/                     # Ch. 9: Generic types
│
├── binary-tree/                       # Ch. 10 & 15: Enums + Iterators
├── basic-router/                      # Ch. 14: Closures & callbacks
│
├── complex/                           # Ch. 12 & 17: Operator overloading + Display
├── interval/                          # Ch. 12: PartialOrd trait
│
├── grep/                              # Ch. 18: CLI tool for text search
├── copy/                              # Ch. 18: Directory tree copying
├── echo-server/                       # Ch. 18: Simple network service
├── http-get/                          # Ch. 18: HTTP client (reqwest)
│
├── cheapo-request/                    # Ch. 20: Async HTTP (async-std)
├── many-requests/                     # Ch. 20: Concurrent requests
├── many-requests-surf/                # Ch. 20: Async HTTP (surf crate)
├── spawn-blocking/                    # Ch. 20: Custom async primitives
├── block-on/                          # Ch. 20: Simple executor
│
├── json-macro/                        # Ch. 21: Procedural macros
│
├── ascii/                             # Ch. 22: Unsafe blocks & functions
├── ref-with-flag/                     # Ch. 22: Raw pointers
├── gap-buffer/                        # Ch. 22: Pointer arithmetic
│
└── libgit2-rs/                        # Ch. 22 FFI: Unsafe FFI bindings
    libgit2-rs-safe/                   # Ch. 22 FFI: Safe wrapper around libgit2
```

### Project Categories by Purpose

**Basic Examples** (Single-file implementations):
- `gcd`, `queue`, `generic-queue`, `interval`, `complex`, `echo-server`, `block-on`

**Binary Programs** (Executable tools):
- `gcd`, `grep`, `copy`, `http-get`, `echo-server`, `many-requests-surf`

**Library Examples** (Reusable components):
- `queue`, `generic-queue`, `binary-tree`, `complex`, `interval`, `ascii`, `ref-with-flag`, `gap-buffer`

**Web/Network** (HTTP and web services):
- `actix-gcd`, `http-get`, `echo-server`, `cheapo-request`, `many-requests`

**Advanced Features**:
- **Async/Await**: `cheapo-request`, `many-requests`, `many-requests-surf`, `spawn-blocking`
- **Unsafe Code**: `ascii`, `ref-with-flag`, `gap-buffer`
- **FFI**: `libgit2-rs`, `libgit2-rs-safe`
- **Macros**: `json-macro`
- **Modules**: `fern_sim`

---

## Standard Commands

### Building and Testing

```bash
# Build a specific project
cd <project-name> && cargo build

# Build with release optimizations
cargo build --release

# Run a binary project
cargo run

# Run with arguments
cargo run -- <arguments>

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test <test_name>

# Check compilation without building
cargo check

# Format code (if rustfmt installed)
cargo fmt

# Lint with clippy
cargo clippy

# Generate documentation
cargo doc --open
```

### Repository-Wide Operations

```bash
# Build all projects
for dir in */; do cd "$dir" && cargo build && cd ..; done

# Run tests on all projects
for dir in */; do cd "$dir" && cargo test && cd ..; done

# Check all projects compile
for dir in */; do cd "$dir" && cargo check && cd ..; done
```

### Git Workflow (for feature branch development)

```bash
# Check current status
git status

# Create feature branch
git checkout -b feature/<description>

# View changes
git diff

# Stage changes
git add <file-path>

# Commit changes
git commit -m "descriptive message"

# Push to feature branch
git push -u origin claude/create-claude-documentation-rCOwU

# Create pull request
gh pr create --title "PR Title" --body "Description"
```

---

## Code Style & Conventions

### Rust Edition and Format

- **Edition**: Rust 2018 across all projects
- **Formatting**: Follow `rustfmt` defaults (implied standard)
- **Naming**:
  - Types: `PascalCase` (e.g., `Queue`, `BinaryTree`, `Complex`)
  - Functions: `snake_case` (e.g., `new_queue`, `build_tree`)
  - Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_SIZE`)
  - Modules: `snake_case` (e.g., `simulation`, `spores`)

### Module Structure

Most projects use one of two patterns:

**Single-file pattern** (for simple projects):
```rust
// src/lib.rs or src/main.rs
// All code in one file
```

**Multi-file pattern** (for complex projects like fern_sim):
```
src/
├── lib.rs          // Main module declarations
├── simulation.rs   // Feature module
├── spores.rs       // Feature module
└── tests/          // Integration tests
```

### Type System Patterns

This repository showcases many trait implementations:

- **Custom Types**: Prefer `struct` for data containers, `enum` for variant types
- **Trait Implementations**: Common patterns include:
  - `Display` and `Debug` for formatting
  - `Operator traits` (Add, Sub, Mul, etc.) for operator overloading
  - `Iterator` for iteration support
  - `Deref` for smart pointers
  - `PartialOrd` and `PartialEq` for comparisons

Example from `complex/`:
```rust
use std::ops::Add;

#[derive(Clone, Copy, Debug)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Add for Complex {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { ... }
}
```

### Unsafe Code Patterns

Projects using unsafe (`ascii`, `ref-with-flag`, `gap-buffer`) follow these principles:

- **Minimal scope**: Unsafe code is isolated in dedicated functions/blocks
- **Documentation**: Unsafe invariants are clearly documented
- **Safety**: Caller/enclosing code must maintain invariants
- **Example** from `ascii/`:
```rust
pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Ascii {
    std::mem::transmute(bytes)
}
```

### Error Handling

Most examples use `Result` for fallible operations:

```rust
use std::io;

fn do_something() -> io::Result<String> {
    // Function body with ? operator
}
```

Simpler examples may use `unwrap()` or `expect()` for clarity in educational context.

### Testing Conventions

**Unit Tests** (within source files):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operation() {
        // Test implementation
    }
}
```

**Integration Tests** (in `tests/` directory):
```rust
// tests/integration_test.rs
#[test]
fn test_from_another_crate() {
    // Uses the library like an external consumer
}
```

Example from `fern_sim/`:
```
tests/
├── integration_test.rs
└── ...
```

---

## Development Workflows

### Setting Up a Project for Work

```bash
# 1. Ensure on feature branch
git checkout claude/create-claude-documentation-rCOwU

# 2. Navigate to project
cd <project-name>

# 3. Verify it builds
cargo build

# 4. Run existing tests
cargo test

# 5. Start development
# (use cargo watch or editor-integrated tools)
```

### Adding New Code

1. **For new functions**: Add to appropriate module file
2. **For new types**: Create in dedicated section or new file if large
3. **For new modules**: Create new `.rs` file and declare in `lib.rs` or `main.rs`
4. **For tests**: Add `#[test]` functions to relevant modules or create test file

### Modifying Existing Code

1. **Understand first**: Read surrounding code and any documentation
2. **Plan changes**: Sketch out impact on dependent code
3. **Implement incrementally**: Make small, testable changes
4. **Test thoroughly**: Run full test suite after changes
5. **Review**: Ask AI to review for style and correctness

### Documentation Standards

- **Inline comments**: Explain "why", not "what" (code shows what)
- **Function docs**: Use `///` for public functions (shown in cargo doc)
- **Module docs**: Use `//!` at top of files if complex
- **Examples**: Include in doc comments for public APIs

---

## Testing Strategy

### Testing Framework

- **Unit testing**: Built-in `#[test]` attribute and `assert!` macros
- **Integration testing**: Separate crate in `tests/` directory
- **No external test frameworks** required (standard library sufficient)

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_behavior() {
        assert_eq!(result, expected);
    }

    #[test]
    fn test_edge_case() {
        assert!(condition);
    }

    #[test]
    #[should_panic]
    fn test_panics_appropriately() {
        // Code that should panic
    }
}
```

### Testing Best Practices

1. **Test structure**: Follow `Arrange → Act → Assert` pattern
2. **Test naming**: Use descriptive names like `test_<function>_<scenario>`
3. **Edge cases**: Include tests for boundary conditions
4. **Error cases**: Use `#[should_panic]` or `Result` returns for error testing
5. **Coverage**: Aim for tests of public APIs; internal functions tested implicitly

### Running Tests

```bash
# Run all tests with output
cargo test -- --nocapture --test-threads=1

# Run specific test
cargo test test_name

# Run tests and show panics
cargo test -- --nocapture

# Run ignoring some tests
cargo test --lib     # Only unit tests
cargo test --test '*' # Only integration tests
```

---

## Git & Branch Conventions

### Branch Strategy

- **Development branch**: `claude/create-claude-documentation-rCOwU` (current feature branch)
- **Feature branches**: Used for development on specific features/improvements
- **Main branch**: Production-ready code (protected)

### Commit Message Format

Keep commit messages clear and concise:

```
<type>: <short description>

<optional detailed explanation>
```

**Types**:
- `feat`: New feature or capability
- `fix`: Bug fix
- `docs`: Documentation update
- `refactor`: Code reorganization without functional change
- `test`: Test additions or modifications
- `chore`: Dependency updates, build configuration

**Examples**:
```
feat: Add CLAUDE.md documentation for AI assistants

docs: Update README chapter references

fix: Correct unsafe code invariant in gap-buffer

chore(deps): bump actix-web from 4.0 to 4.1
```

### Push to Feature Branch

When pushing to the designated feature branch:

```bash
# Push with upstream tracking
git push -u origin claude/create-claude-documentation-rCOwU

# Subsequent pushes
git push origin claude/create-claude-documentation-rCOwU
```

**Important**: The branch name must start with `claude/` and end with the session ID (`-rCOwU` in this case).

### Creating Pull Requests

When code is ready:

```bash
gh pr create --title "Feature: <description>" \
  --body "Description of changes and testing"
```

Use the template:
```markdown
## Summary
- Brief description of changes
- Any important implementation notes

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Code follows style guide
- [ ] No clippy warnings

## Related Issues
- Fixes #<issue-number> (if applicable)
```

---

## AI Assistant Guidelines

### Philosophy

This guide is designed for **AI assistants working alongside developers**. AI tools should:

1. **Enhance human judgment**, not replace it
2. **Accelerate common tasks** (boilerplate, refactoring, documentation)
3. **Maintain code quality** through testing and review
4. **Communicate clearly** about assumptions and trade-offs

### How to Work With Claude Code

#### Initial Exploration (Plan Phase)

When starting work on a feature or bug:

```
1. Have Claude explore relevant files
   - Understand current implementation
   - Identify related code patterns
   - Check existing tests

2. Request a plan before coding
   - Use Plan Mode for complex changes
   - Review the proposed approach
   - Suggest adjustments if needed

3. Get approval before implementation
   - Ensure alignment on strategy
   - Clarify any ambiguities
   - Set expectations for testing
```

#### Implementation (Code Phase)

During implementation:

```
1. Work in focused iterations
   - Target 5-20 file changes per iteration
   - Implement one feature at a time
   - Test frequently

2. Use checkpoints for rollback
   - Commit working states regularly
   - Keep git history clean
   - Enable easy reversion if needed

3. Ask for code review
   - "Review this for style compliance"
   - "Check for unsafe code issues"
   - "Verify test coverage"
```

#### Verification (Test Phase)

Before finalizing:

```
1. Run full test suite
   - Verify existing tests still pass
   - Add tests for new functionality
   - Check coverage of edge cases

2. Lint and format
   - Run clippy for warnings
   - Use cargo fmt for formatting
   - Verify no compiler warnings

3. Manual review
   - Read all changes once more
   - Verify against style guide
   - Check for unintended side effects
```

### Effective Prompts for AI Assistants

**Good Prompt**:
```
In the binary-tree project, add a method `depth(&self) -> usize` to the BinaryTree type that returns the maximum depth of the tree (height + 1 for consistency with the existing codebase). Follow the iterator pattern already used in this crate. Include tests for edge cases (empty tree, single node, balanced vs unbalanced trees).
```

**Vague Prompt**:
```
Add a method to binary-tree
```

**Specific Context**:
```
In the actix-gcd project, the web framework is actix-web 4.1. When adding the new route handler for /lcm (least common multiple), follow the same pattern as the existing /gcd route in src/main.rs (lines 23-35). The response should be JSON with the same structure as the GCD endpoint.
```

### Do Not Touch Zones

⚠️ **Critical Files** - Modify only with explicit user approval:

- `LICENSE-MIT` - License terms (immutable)
- Root `README.md` - Official chapter mapping (coordinate changes)
- `.git/` - Git internals (never modify)
- `Cargo.lock` files - Only update via `cargo update`

### Code Style Compliance

When adding code, ensure:

1. ✅ **Naming conventions**: Use `snake_case` for functions, `PascalCase` for types
2. ✅ **Module organization**: Follow existing project structure
3. ✅ **Documentation**: Add `///` comments for public items
4. ✅ **Testing**: Include `#[test]` functions for new functionality
5. ✅ **No compiler warnings**: Code should compile cleanly
6. ✅ **Consistent formatting**: Run `cargo fmt` before committing

### When to Ask for Clarification

Ask the developer (not just assume) about:

- **Design decisions**: "Should we use an enum or trait objects for X?"
- **Performance trade-offs**: "This recursive approach is elegant but slower—is that acceptable?"
- **Architecture impact**: "This change affects the module boundary. Should we refactor?"
- **Testing scope**: "Should we add benchmark tests or just functional tests?"
- **Backward compatibility**: "Should we keep the old function signature as deprecated?"

### Common Task Patterns

**Pattern: Adding a New Function**
```
1. Research: Where should it live? What's the existing API?
2. Plan: What will the signature be? What are edge cases?
3. Code: Implement with clear logic
4. Test: Add [#test] functions for coverage
5. Review: Check style, docs, performance
6. Commit: Message explaining the addition
```

**Pattern: Fixing a Bug**
```
1. Understand: What's the root cause? How does the code currently fail?
2. Plan: What's the minimal fix? Are there side effects?
3. Test: Write test that reproduces the bug
4. Fix: Apply the minimal change
5. Verify: Confirm test now passes
6. Commit: Reference any issue number, explain the fix
```

**Pattern: Refactoring**
```
1. Understand: What's the goal? Better readability? Performance?
2. Plan: What's the scope? What could break?
3. Code: Make incremental changes
4. Test: Ensure existing tests still pass (functionality unchanged)
5. Review: Is the code clearer/better?
6. Commit: Explain why the refactoring improves the code
```

---

## Security & Compliance

### Code Safety

- **Unsafe code**: Clearly documented in `ascii`, `ref-with-flag`, `gap-buffer` projects
- **Invariants**: Documented when unsafe code relies on assumptions
- **Public APIs**: Should be safe to use (unsafe hidden in implementation)

### Dependencies

- **Minimize external crates**: Most examples use few dependencies
- **Review transitive dependencies**: Check `cargo tree` output
- **Security updates**: Keep dependencies current for security patches

### Testing Security

- **Input validation**: Test edge cases and invalid inputs
- **Error handling**: Don't panic on bad input (use Result types)
- **Unsafe boundaries**: Test assumptions made by unsafe code

---

## Troubleshooting Common Issues

### Build Failures

**Issue**: `error: failed to find required target libgit2`

**Solution**: For libgit2 projects, ensure libgit2 is installed:
```bash
# macOS
brew install libgit2

# Linux
apt-get install libgit2-dev

# Windows
# Follow instructions in libgit2-rs/build.rs comments
```

**Issue**: Dependency version conflicts

**Solution**: Check `Cargo.lock` and run `cargo update` to resolve:
```bash
cd <project>
cargo clean
cargo update
cargo build
```

### Test Failures

**Issue**: Tests pass locally but not in CI

**Solution**: Run tests with full output:
```bash
cargo test -- --nocapture --test-threads=1
```

### Performance Issues

**Issue**: Binary runs slowly

**Solution**: Build with optimizations:
```bash
cargo build --release
./target/release/<binary>
```

---

## Best Practices Summary for AI Assistants

### When Analyzing Code

1. ✅ Read the entire source file first
2. ✅ Understand the surrounding context
3. ✅ Check for existing patterns to follow
4. ✅ Look at related code in other projects
5. ✅ Review existing tests to understand expected behavior

### When Writing Code

1. ✅ Follow project style conventions
2. ✅ Add documentation comments for public items
3. ✅ Include tests for new functionality
4. ✅ Use meaningful variable and function names
5. ✅ Keep functions focused and testable

### When Committing

1. ✅ Write clear, descriptive commit messages
2. ✅ Group related changes together
3. ✅ Verify tests pass before committing
4. ✅ Use conventional commit format
5. ✅ Reference related issues if applicable

### When Stuck

1. ✅ Explain the current understanding of the problem
2. ✅ Describe what has been tried so far
3. ✅ Ask specific questions (not vague ones)
4. ✅ Provide code examples of the issue
5. ✅ Request a plan or architecture discussion

---

## Additional Resources

### External Repositories (Mentioned in README)

- [Mandelbrot Set Plotter](https://github.com/ProgrammingRust/mandelbrot) - Multi-threaded graphics
- [Fingertips Search Engine](https://github.com/ProgrammingRust/fingertips) - Concurrency patterns
- [Async Chat Application](https://github.com/ProgrammingRust/async-chat) - Complete async example

### Rust Documentation

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust Standard Library](https://doc.rust-lang.org/std/)
- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
- [Rustlings (Interactive Learning)](https://github.com/rust-lang/rustlings)

### Crate Documentation

- [Actix-web](https://actix.rs/) - Web framework used in actix-gcd
- [reqwest](https://docs.rs/reqwest/) - HTTP client
- [async-std](https://docs.rs/async-std/) - Async runtime
- [serde](https://serde.rs/) - Serialization framework

---

## Document History

- **Created**: December 2025
- **Purpose**: Comprehensive guide for AI assistants working on Rust examples
- **Scope**: All 25 projects in the repository
- **Status**: Active and maintained during development sessions

---

## Questions for AI Assistants

Before starting work, clarify:

1. **Scope**: Which project(s) are we modifying?
2. **Goal**: What feature/fix are we implementing?
3. **Testing**: What should the test coverage include?
4. **Documentation**: Should existing docs be updated?
5. **Compatibility**: Any backward compatibility concerns?

---

*This guide enables clear communication between developers and AI assistants to build high-quality, well-tested Rust code that demonstrates language best practices.*
