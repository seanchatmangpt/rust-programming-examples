# Backward Compatibility & Migration Strategies

Software evolution is inevitable. The challenge lies in managing this evolution without breaking the ecosystem of users and downstream dependencies that rely on your code.

## Semantic Versioning Deep Dive

Semantic Versioning (SemVer) provides a shared vocabulary for communicating the nature of changes.

### MAJOR.MINOR.PATCH Semantics

```
MAJOR.MINOR.PATCH
  │     │     │
  │     │     └── Bug fixes (backward compatible)
  │     │
  │     └──────── New features (backward compatible)
  │
  └────────────── Breaking changes
```

**Example version progression:**
```
1.0.0  - Initial stable release
1.0.1  - Bug fix (PATCH)
1.1.0  - New feature (MINOR)
1.2.0  - Another feature (MINOR)
2.0.0  - Breaking change (MAJOR)
```

### What Constitutes a Breaking Change?

**Definitely breaking:**
- Removing public items
- Changing function signatures
- Adding required parameters
- Changing trait bounds
- Removing trait implementations

**Surprisingly breaking:**
- Adding new methods to traits (users may have implemented them)
- Adding enum variants (for non-`#[non_exhaustive]` enums)

**Not breaking:**
- Adding public functions, types, modules
- Adding optional parameters via builders
- Implementing traits for existing types
- Adding `#[non_exhaustive]`

## Maintaining Compatibility

### Deprecation with Grace Periods

```rust
#[deprecated(
    since = "1.8.0",
    note = "use `process_v2` instead, will be removed in 2.0.0"
)]
pub fn process(data: &str) -> Output {
    process_v2(data).into()
}

pub fn process_v2(data: &str) -> DetailedOutput {
    // New implementation
}
```

**Grace period timeline:**

| Version | Action | User Impact |
|---------|--------|-------------|
| v1.8.0 | Deprecate `process()` | Warning on use |
| v1.9.0 | Add migration guide | Warning + docs |
| v1.10.0 | Final warning release | 6 months elapsed |
| v2.0.0 | Remove `process()` | Migration required |

### Feature Flags for Transitions

```toml
[features]
default = ["v1-compat"]
v1-compat = []
v2-api = []
```

```rust
#[cfg(feature = "v1-compat")]
#[deprecated]
pub fn old_api() { }

#[cfg(feature = "v2-api")]
pub fn new_api() { }
```

### Multiple API Versions

```rust
pub mod v1 {
    pub use crate::legacy::*;
}

pub mod v2 {
    pub use crate::current::*;
}

pub use v2::*;
```

## Deprecation Strategies

### Announcing Deprecation

Communicate through multiple channels:

1. **Code**: `#[deprecated]` attribute
2. **CHANGELOG**: Dedicated deprecation section
3. **Documentation**: Migration guide in rustdoc
4. **Release notes**: Highlight in GitHub releases
5. **Blog**: For major deprecations

### Timeline Best Practices

| Deprecation Scope | Minimum Grace Period |
|-------------------|---------------------|
| Minor function | 2 minor versions or 3 months |
| Major API component | 6 months minimum |
| Core functionality | 12 months, with LTS option |

### Automated Migration Tooling

```rust
// migrations/v1_to_v2.rs
fn main() {
    let re = Regex::new(r"process\(([^)]+)\)").unwrap();

    for entry in walkdir::WalkDir::new("src") {
        let path = entry.path();
        if path.extension() == Some("rs".as_ref()) {
            let content = fs::read_to_string(path).unwrap();
            let updated = re.replace_all(&content, "process_v2($1)");
            if content != updated {
                println!("Migrating: {}", path.display());
                fs::write(path, updated.as_ref()).unwrap();
            }
        }
    }
}
```

## LTS Releases

Long-Term Support releases provide stability for users who cannot migrate quickly:

```
Timeline:
├── v1.0.0 (January 2024)
├── v1.10.0 LTS (December 2024) ← LTS branch created
├── v2.0.0 (February 2025)
├── v1.10.1 LTS (March 2025) ← Security fix
└── v1.10.x EOL (December 2025) ← LTS ends
```

**LTS Policies:**

- **Duration**: 12 months from LTS designation
- **Scope**: Security fixes and critical bugs only
- **No features**: New functionality goes to current major
- **Backports**: Security fixes backported within 30 days

## Version Testing Matrices

Test across multiple versions to ensure compatibility claims:

```yaml
jobs:
  test:
    strategy:
      matrix:
        rust: [1.56, 1.65, 1.75, stable, beta, nightly]
        include:
          - rust: 1.56
            msrv: true
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
      - run: cargo test
```

## Case Study: clap-noun-verb Compatibility Roadmap

### Migration Roadmap

**Phase 1: v1.8.0 (Month 0)**
- Introduce builder API alongside macro
- Mark macro as `#[deprecated]`
- Publish migration guide

**Phase 2: v1.9.0 (Month 2)**
- Release migration tool
- Add clippy lint
- Update examples

**Phase 3: v1.10.0 (Month 4)**
- Macro emits warning
- Builder API marked stable
- LTS branch created

**Phase 4: v2.0.0 (Month 6)**
- Remove macro API
- Builder API is only option

### Compatibility Matrix

| User Scenario | v1.8 | v1.9 | v1.10 | v2.0 |
|--------------|------|------|-------|------|
| Macro API | Deprecated | Deprecated | Deprecated | Removed |
| Builder API | Available | Stable | Stable | Only |
| Migration tool | N/A | Available | Available | N/A |
| Security updates | Yes | Yes | LTS | Yes |

## Best Practices

### Version Bump Decision Tree

```
Removing public API? → MAJOR
Changing signatures? → MAJOR
Changing behavior? → MAJOR
Adding new API? → MINOR
Bug fixes only? → PATCH
```

### Pre-Release Checklist

Before releasing a breaking change:

- [ ] CHANGELOG updated
- [ ] Migration guide written
- [ ] API documentation updated
- [ ] Migration tooling tested
- [ ] All examples updated
- [ ] Minimum grace period elapsed
- [ ] LTS branch created if applicable

### User Communication

Maintain trust through clear communication:

- **CHANGELOG**: Keep it updated
- **Migration guides**: Detailed, step-by-step
- **Release notes**: Highlight breaking changes
- **Deprecation timeline**: Always specify removal version
- **Upgrade testing**: Provide example migrations

## Summary

Backward compatibility is a discipline that respects your users' time and builds trust. By understanding semantic versioning deeply, identifying breaking changes accurately, and planning deprecations thoughtfully, you can evolve your API while maintaining stability.

Key principles:

- **Communicate early and often** about upcoming changes
- **Provide migration paths** before removing functionality
- **Test across versions** to verify compatibility
- **Respect the timeline** you commit to
- **Document everything** so users can self-serve

When users can update your crate with confidence, they stay current with security fixes and new features—benefiting everyone.
