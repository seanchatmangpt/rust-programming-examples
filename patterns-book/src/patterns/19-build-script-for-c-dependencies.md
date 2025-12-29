# 19. BUILD SCRIPT FOR C DEPENDENCIES

*Before Rust code can be compiled, before binaries can link, a small script runs—configuring paths, finding libraries, bridging the gap between Cargo and the C toolchain.*

...within a **SAFE WRAPPER AROUND C LIBRARY (22)** or **RAW BINDINGS MODULE (16)**, when your Rust crate depends on a C library that must be linked at compile time...

◆ ◆ ◆

**How do you tell the Rust compiler where to find C libraries on the user's system without hardcoding paths or requiring manual configuration?**

Rust's `extern` declarations can reference C functions, but the linker must actually find the compiled C library to link against. Different systems install libraries in different locations: `/usr/lib`, `/usr/local/lib`, custom directories from source builds. If you hardcode a path in your code, the crate won't compile on most systems.

Cargo provides no built-in mechanism for finding system libraries. It focuses on Rust dependencies, not native ones. Yet linking requires precise information: library names, search paths, sometimes additional flags. You need to communicate this information to the compiler, but you can't know it until the user's system is examined.

The solution must run before compilation, must be able to probe the system, and must speak to the compiler in a language it understands: compiler flags. It must also be transparent to users—they shouldn't need to manually configure anything for common library installations.

**Therefore:**

**Create a `build.rs` file at your crate root that runs before compilation. Use `println!("cargo:rustc-link-search=native=PATH")` to specify library search paths and `println!("cargo:rustc-link-lib=NAME")` to specify libraries to link.**

```rust
// build.rs
fn main() {
    // Tell the linker where to find libgit2
    println!("cargo:rustc-link-search=native=/usr/local/lib");
    println!("cargo:rustc-link-search=native=/home/user/libgit2-0.25.1/build");

    // Tell the linker to link against libgit2
    println!("cargo:rustc-link-lib=git2");
}
```

For more sophisticated cases, probe the system at build time:

```rust
// build.rs with pkg-config
fn main() {
    // Use pkg-config to find library details
    pkg_config::Config::new()
        .atleast_version("0.25.0")
        .probe("libgit2")
        .unwrap();
}
```

Or compile C code directly:

```rust
// build.rs compiling C sources
fn main() {
    cc::Build::new()
        .file("src/wrapper.c")
        .compile("wrapper");
}
```

*The build script is Cargo's extension point. It runs in a constrained environment before compilation, printing instructions that Cargo interprets and passes to the compiler.*

◆ ◆ ◆

Build scripts transform the build process from static to dynamic. Instead of assuming where libraries live, the script can search, probe, or even compile C code on the fly. This flexibility makes crates portable across different systems and configurations.

The `cargo:` prefix is Cargo's instruction protocol. When your build script prints these specially formatted lines, Cargo captures them and converts them to compiler flags. Common instructions include:

- `cargo:rustc-link-search=native=PATH` — Add library search path
- `cargo:rustc-link-lib=NAME` — Link against library
- `cargo:rerun-if-changed=PATH` — Rebuild only if file changes
- `cargo:rustc-cfg=FEATURE` — Enable conditional compilation

Build scripts have limited capabilities intentionally. They cannot access the network (for security and reproducibility), and they should be deterministic. They focus on one task: preparing the build environment.

For complex C dependencies, use helper crates: `pkg-config` to query system package metadata, `cc` to compile C/C++ code, `bindgen` to auto-generate FFI bindings from C headers. These crates handle platform differences and corner cases, letting you focus on library-specific logic.

The build script is also where you can generate code: create Rust source files from templates, parse data files into constants, or generate bindings from C headers. Any generated `.rs` files should be written to the directory specified by the `OUT_DIR` environment variable.

Use **RAW BINDINGS MODULE (16)** for the FFI declarations that the linked library provides, and consider **CONDITIONAL COMPILATION BY OS (20)** if library locations or linking requirements differ by platform.
