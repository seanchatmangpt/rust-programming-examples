# FFI as Architectural Boundary

## Understanding the Boundary

Foreign Function Interface (FFI) represents more than just a technical mechanism for calling C code—it defines a critical architectural boundary in Rust systems. This boundary separates the world of compile-time memory safety guarantees from the world of runtime assumptions and manual verification. Understanding this boundary is essential for building robust systems that leverage existing C libraries while maintaining Rust's safety properties.

When we examine the `libgit2-rs` and `libgit2-rs-safe` projects in this repository, we see this boundary in practice. The raw FFI bindings in `libgit2-rs` expose C functions directly, placing the burden of safety entirely on the caller. The safe wrapper in `libgit2-rs-safe` encapsulates this unsafe boundary, presenting a safe Rust API to consumers.

## When to Cross the Boundary

The decision to use FFI should be deliberate and justified. There are several compelling reasons to call C code from Rust:

**Leveraging Existing Libraries**: Mature C libraries like libgit2, libcurl, or OpenSSL represent decades of development, testing, and battle-hardening. Rewriting these libraries in Rust would be a massive undertaking, and the reimplementation might introduce bugs that the original has already discovered and fixed. The pragmatic choice is often to wrap the C library safely rather than reimplement it.

**Performance-Critical Operations**: While Rust can be as fast as C, some highly optimized C libraries use techniques that are difficult to express in safe Rust. SIMD operations, cache-aware algorithms, and platform-specific optimizations might already exist in C. However, this justification should be backed by profiling data—perceived performance benefits often don't materialize in practice.

**Platform Integration**: Operating system APIs are typically C interfaces. Windowing systems, graphics drivers, and kernel interfaces all expose C ABIs. To interact with the platform, FFI is often the only option.

**Gradual Migration**: When modernizing a legacy C codebase, FFI allows incremental migration to Rust. You can rewrite critical components in Rust while keeping the rest in C, gradually improving safety over time.

## Cost-Benefit Analysis

Every FFI boundary comes with costs that must be weighed against benefits:

### Costs of FFI

**Safety Verification Burden**: Every call across the FFI boundary must be manually verified for safety. You must ensure pointers are valid, lifetimes are respected, and invariants are maintained. The compiler cannot help you here.

From `libgit2-rs/src/main.rs`:

```rust
unsafe {
    check("initializing library", raw::git_libgit2_init());

    let mut repo = ptr::null_mut();
    check("opening repository",
          raw::git_repository_open(&mut repo, path.as_ptr()));

    // ... use repo ...

    raw::git_repository_free(repo);
}
```

Every function call here requires manual verification. Did we initialize the library first? Is the path pointer valid? Do we free resources in the correct order? Miss any of these, and we have undefined behavior.

**Maintenance Complexity**: FFI code is harder to maintain. Changes to the C library require updates to bindings. Platform differences in C APIs require conditional compilation. Type mismatches between Rust and C types must be carefully managed.

**Error Handling Impedance**: C libraries typically use return codes or errno for errors, while Rust uses Result types. Converting between these paradigms adds boilerplate and potential for mistakes.

**Testing Challenges**: FFI code is harder to test. You need the C library available at build and test time. Mocking FFI dependencies is difficult. Sanitizers may not work across the language boundary.

### Benefits of FFI

**Immediate Access to Ecosystems**: FFI provides instant access to the vast ecosystem of C libraries. Instead of waiting for pure-Rust alternatives, you can use proven solutions today.

**Interoperability**: FFI enables Rust code to integrate into existing systems. You can call Rust from C and C from Rust, enabling gradual adoption.

**Zero-Cost Abstraction**: When done correctly, FFI calls have minimal overhead—typically just a function call. The Rust wrapper around C code can be as fast as using C directly.

## Unsafe as Encapsulation

The `unsafe` keyword in Rust serves a dual purpose at FFI boundaries. It marks code that requires manual verification, but more importantly, it enables encapsulation of unsafe operations behind safe APIs.

Consider the evolution from `libgit2-rs` to `libgit2-rs-safe`. The raw version exposes unsafe everywhere:

```rust
// Raw FFI - unsafe everywhere
unsafe {
    let mut commit = ptr::null_mut();
    check("looking up commit",
          raw::git_commit_lookup(&mut commit, repo, &oid));
    show_commit(commit);
    raw::git_commit_free(commit);
}
```

The safe wrapper encapsulates this:

```rust
// Safe wrapper - unsafe hidden
let commit = repo.find_commit(&commit_oid)
    .expect("looking up commit");
let author = commit.author();
println!("{} <{}>",
         author.name().unwrap_or("(none)"),
         author.email().unwrap_or("none"));
// Commit freed automatically via Drop
```

The unsafe code still exists, but it's concentrated in the wrapper implementation. Users of the API don't need to think about pointers, manual memory management, or safety invariants. The wrapper maintains these invariants internally.

This is the proper role of `unsafe`: building safe abstractions over unsafe primitives. The unsafe code is audited once, in the wrapper implementation, rather than verified at every call site.

## FFI as Architectural Boundary

Treating FFI as an architectural boundary means designing your system with clear separation between the safe Rust world and the unsafe C world.

**Minimize the Boundary Surface**: Keep the FFI layer thin. Don't expose raw C types throughout your codebase. Instead, convert at the boundary:

```rust
// In libgit2-rs-safe
impl Repository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Repository> {
        ensure_initialized();

        // Convert Rust Path to C string at the boundary
        let path = path_to_cstring(path.as_ref())?;
        let mut repo = ptr::null_mut();
        unsafe {
            check(raw::git_repository_open(&mut repo, path.as_ptr()))?;
        }
        Ok(Repository { raw: repo })
    }
}
```

The `Path` type is Rust's safe path representation. We convert it to `CString` (a C-compatible string) right at the boundary, call the C function, then wrap the result in a safe `Repository` type. Callers never see pointers or C types.

**Enforce Invariants at the Boundary**: The FFI boundary is where you enforce safety invariants:

```rust
impl Drop for Repository {
    fn drop(&mut self) {
        unsafe {
            raw::git_repository_free(self.raw);
        }
    }
}
```

This `Drop` implementation ensures repositories are always freed, even if the user forgets. The invariant "opened repositories must be closed" is enforced automatically.

**Clear Ownership Models**: Decide who owns what at the boundary. Does Rust own the C data, or does C own it? In `libgit2-rs-safe`, Rust owns the repository pointer—when the Rust `Repository` is dropped, we free the C resource. But the `Signature` returned from a commit is owned by the commit—we don't free it ourselves:

```rust
pub struct Signature<'text> {
    raw: *const raw::git_signature,
    _marker: PhantomData<&'text str>
}
```

The lifetime parameter `'text` ties the signature to the commit that owns it. When the commit is freed, all signatures derived from it become invalid, enforced by the borrow checker.

## Strategic Design Decisions

When designing FFI boundaries, make these strategic decisions explicit:

1. **Error Handling Strategy**: Will you panic on C errors, return Results, or use another approach? The safe wrapper uses Results for recoverable errors.

2. **Memory Management Model**: RAII (Resource Acquisition Is Initialization) via Drop is typically best. Resources are freed automatically when values go out of scope.

3. **Thread Safety**: Are C functions thread-safe? If not, how will you ensure safe concurrent access? The safe wrapper uses `std::sync::Once` to ensure initialization happens exactly once.

4. **Lifetime Management**: How do borrowed C pointers relate to their owners? Use PhantomData and lifetime parameters to express these relationships.

5. **Type Safety**: Create newtype wrappers around C types to prevent misuse. An `Oid` is not just raw bytes—it's a specific Git object identifier.

## Conclusion

FFI is not just about calling C functions—it's about carefully managing the boundary between safe and unsafe code. Treat this boundary as an architectural element that deserves careful design. Minimize the surface area, encapsulate unsafe operations, enforce invariants automatically, and present a safe, idiomatic Rust API to consumers. When done well, FFI allows you to leverage the C ecosystem while maintaining Rust's safety guarantees.
