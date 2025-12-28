# Unsafe Rust and FFI

## The Safe/Unsafe Boundary

Rust's safety guarantees are famous: no null pointers, no data races, no use-after-free. But these guarantees come with restrictions. Sometimes you need to break the rules - to interface with C libraries, implement low-level optimizations, or build safe abstractions that require unsafe internals.

This is where `unsafe` comes in. It's not "unsafe code" - it's code where you, the programmer, are responsible for maintaining invariants that the compiler can't verify.

## What Unsafe Actually Means

In Python, everything is "unsafe" in Rust's sense - there's no compile-time memory safety. Python can crash with segfaults if you misuse C extensions or ctypes. Rust makes the boundary explicit.

### Safe Rust

```rust
fn process_data(data: &[u8]) -> u8 {
    data[0]  // Bounds-checked at runtime
}
```

The compiler guarantees:
- No null pointers
- No dangling references
- No data races
- No buffer overflows (with panic on bounds check failure)

### Unsafe Rust

```rust
unsafe fn process_data_unchecked(data: &[u8]) -> u8 {
    *data.get_unchecked(0)  // No bounds check - YOUR responsibility
}
```

You can now:
- Dereference raw pointers
- Call unsafe functions
- Access mutable statics
- Implement unsafe traits
- Access union fields

But you must manually maintain safety invariants.

## What Unsafe Doesn't Mean

Common misconceptions:

1. **NOT "turn off safety checks"** - Most checks still apply (borrowing, ownership, types)
2. **NOT "undefined behavior is OK"** - UB is still a bug, just harder to catch
3. **NOT "abandon all hope"** - You create safe abstractions using unsafe primitives

Think of `unsafe` like Python's `ctypes` - a way to interoperate with the unsafe world while maintaining safe interfaces.

## The Five Unsafe Superpowers

### 1. Dereferencing Raw Pointers

The example from `/home/user/rust-programming-examples/ref-with-flag/src/lib.rs` shows this:

```rust
pub struct RefWithFlag<'a, T> {
    ptr_and_bit: usize,
    behaves_like: PhantomData<&'a T>
}

impl<'a, T: 'a> RefWithFlag<'a, T> {
    pub fn new(ptr: &'a T, flag: bool) -> RefWithFlag<T> {
        assert!(align_of::<T>() % 2 == 0);
        RefWithFlag {
            ptr_and_bit: ptr as *const T as usize | flag as usize,
            behaves_like: PhantomData
        }
    }

    pub fn get_ref(&self) -> &'a T {
        unsafe {
            let ptr = (self.ptr_and_bit & !1) as *const T;
            &*ptr  // Dereference raw pointer
        }
    }

    pub fn get_flag(&self) -> bool {
        self.ptr_and_bit & 1 != 0
    }
}
```

This type packs a reference and a boolean into a single `usize` by stealing the lowest bit (which is always 0 for aligned pointers).

Why unsafe is needed:
- Converting reference to integer (`as usize`) and back (`as *const T`)
- Dereferencing the raw pointer (`&*ptr`)

Safety invariants you must maintain:
- The alignment check (`align_of::<T>() % 2 == 0`)
- The pointer remains valid for lifetime `'a`
- No one else mutates through this pointer

Compare to Python:

```python
import ctypes

class RefWithFlag:
    def __init__(self, obj, flag):
        # Python can't do this safely!
        # Would need ctypes tricks and is extremely fragile
        self.obj = obj
        self.flag = flag  # Stored separately, not packed
```

Python doesn't expose raw memory addresses safely. Rust gives you the power but requires `unsafe`.

### 2. Calling Unsafe Functions

From the libgit2 example (`/home/user/rust-programming-examples/libgit2-rs/src/main.rs`):

```rust
fn check(activity: &'static str, status: c_int) -> c_int {
    if status < 0 {
        unsafe {
            let error = &*raw::giterr_last();  // Unsafe: call C function
            println!("error while {}: {} ({})",
                     activity,
                     CStr::from_ptr(error.message).to_string_lossy(),
                     error.klass);
            std::process::exit(1);
        }
    }
    status
}

unsafe fn show_commit(commit: *const raw::git_commit) {
    let author = raw::git_commit_author(commit);  // C function call

    let name = CStr::from_ptr((*author).name).to_string_lossy();
    let email = CStr::from_ptr((*author).email).to_string_lossy();
    println!("{} <{}>\n", name, email);

    let message = raw::git_commit_message(commit);
    println!("{}", CStr::from_ptr(message).to_string_lossy());
}

fn main() {
    let path = std::env::args().skip(1).next()
        .expect("usage: git-toy PATH");
    let path = CString::new(path)
        .expect("path contains null characters");

    unsafe {
        check("initializing library", raw::git_libgit2_init());

        let mut repo = ptr::null_mut();
        check("opening repository",
              raw::git_repository_open(&mut repo, path.as_ptr()));

        let c_name = b"HEAD\0".as_ptr() as *const c_char;
        let oid = {
            let mut oid = mem::MaybeUninit::uninit();
            check("looking up HEAD",
                  raw::git_reference_name_to_id(oid.as_mut_ptr(), repo, c_name));
            oid.assume_init()
        };

        let mut commit = ptr::null_mut();
        check("looking up commit",
              raw::git_commit_lookup(&mut commit, repo, &oid));

        show_commit(commit);

        raw::git_commit_free(commit);
        raw::git_repository_free(repo);
        check("shutting down library", raw::git_libgit2_shutdown());
    }
}
```

This calls C functions from libgit2. Every C call is unsafe because:
- C doesn't have Rust's safety guarantees
- You must manually ensure pointers are valid
- You must manually manage memory (free objects)
- You must handle null pointers

Safety invariants:
- Initialize library before using it
- Free resources in the correct order
- Don't use pointers after freeing them
- Pass valid, non-null pointers to C functions

Compare to Python's ctypes:

```python
import ctypes

libgit2 = ctypes.CDLL("libgit2.so")

# Also unsafe, but Python hides it
repo = ctypes.c_void_p()
result = libgit2.git_repository_open(
    ctypes.byref(repo),
    b"/path/to/repo"
)
```

Python's ctypes is implicitly unsafe. Mistakes cause segfaults. Rust makes the unsafe boundary explicit.

### 3. Accessing Mutable Statics

```rust
static mut COUNTER: i32 = 0;

fn increment_counter() {
    unsafe {
        COUNTER += 1;  // Unsafe: data race possible
    }
}
```

Mutable statics are unsafe because multiple threads could access them simultaneously, causing data races.

Safe alternative: use `Mutex` or atomic types:

```rust
use std::sync::atomic::{AtomicI32, Ordering};

static COUNTER: AtomicI32 = AtomicI32::new(0);

fn increment_counter() {
    COUNTER.fetch_add(1, Ordering::SeqCst);  // Safe!
}
```

Python equivalent:

```python
counter = 0

def increment_counter():
    global counter
    counter += 1  # Race condition! Not thread-safe!
```

Python doesn't warn you. Rust requires `unsafe` or safe synchronization primitives.

### 4. Implementing Unsafe Traits

Some traits are `unsafe` to implement because you must maintain invariants:

```rust
unsafe trait TrustedLen: Iterator {
    // Contract: size_hint() must be exact
}

unsafe impl TrustedLen for MyIterator {
    // You guarantee size_hint() is always correct
}
```

The `Send` and `Sync` traits are also unsafe:

```rust
struct MyType {
    ptr: *mut u8,
}

// You must manually implement Send if your type is thread-safe
unsafe impl Send for MyType {}

// You must manually implement Sync if &MyType can be shared between threads
unsafe impl Sync for MyType {}
```

By default, the compiler conservatively assumes types with raw pointers aren't thread-safe. You use `unsafe impl` to override this.

Python has no equivalent - there's no compile-time thread safety checking.

### 5. Accessing Union Fields

```rust
union Value {
    int: i32,
    float: f32,
}

let v = Value { int: 42 };
unsafe {
    println!("As float: {}", v.float);  // Reinterpret bits
}
```

Accessing union fields is unsafe because you might read the wrong variant. You must track which variant is active.

Python doesn't have unions. You'd use ctypes:

```python
class Value(ctypes.Union):
    _fields_ = [("int", ctypes.c_int32), ("float", ctypes.c_float)]

v = Value(int=42)
print(v.float)  # Reinterprets bits, implicitly unsafe
```

## Safety Invariants

When you write `unsafe`, you're making a promise to maintain invariants:

### Example: RefWithFlag

```rust
impl<'a, T: 'a> RefWithFlag<'a, T> {
    pub fn new(ptr: &'a T, flag: bool) -> RefWithFlag<T> {
        // INVARIANT: T must be at least 2-byte aligned
        assert!(align_of::<T>() % 2 == 0);

        RefWithFlag {
            // INVARIANT: Lowest bit stores flag, rest stores pointer
            ptr_and_bit: ptr as *const T as usize | flag as usize,
            behaves_like: PhantomData
        }
    }

    pub fn get_ref(&self) -> &'a T {
        unsafe {
            // INVARIANT: We can clear lowest bit and dereference
            let ptr = (self.ptr_and_bit & !1) as *const T;

            // INVARIANT: The pointer is still valid
            &*ptr
        }
    }
}
```

The invariants:
1. `T` is at least 2-byte aligned (checked at runtime)
2. The lowest bit is always 0 in the original pointer
3. The pointer remains valid for lifetime `'a`
4. No mutable aliasing occurs

Break any of these and you get undefined behavior.

## FFI: Foreign Function Interface

FFI lets Rust call C code and vice versa. This is inherently unsafe because C doesn't have Rust's guarantees.

### Declaring C Functions

```rust
extern "C" {
    fn strlen(s: *const c_char) -> size_t;
}

let s = b"Hello\0";
let len = unsafe { strlen(s.as_ptr() as *const c_char) };
```

You must ensure:
- The string is null-terminated
- The pointer is valid
- The string outlives the call

### Calling Rust from C

```rust
#[no_mangle]
pub extern "C" fn rust_function(x: i32) -> i32 {
    x * 2
}
```

`#[no_mangle]` prevents name mangling so C can find the function. `extern "C"` uses C calling convention.

### Real Example: libgit2

The libgit2 example shows real-world FFI:

```rust
use std::ffi::CStr;
use std::os::raw::c_int;

fn check(activity: &'static str, status: c_int) -> c_int {
    if status < 0 {
        unsafe {
            let error = &*raw::giterr_last();
            println!("error while {}: {} ({})",
                     activity,
                     CStr::from_ptr(error.message).to_string_lossy(),
                     error.klass);
            std::process::exit(1);
        }
    }
    status
}
```

This wraps C error handling in a safe Rust function. The `unsafe` block:
1. Calls the C function `giterr_last()`
2. Dereferences the returned pointer
3. Converts C strings (`*const c_char`) to Rust strings

Safety invariants:
- libgit2 guarantees `giterr_last()` returns a valid pointer after an error
- The error pointer is valid until the next libgit2 call
- We don't modify the error (only read it)

Compare to Python's ctypes:

```python
class GitError(ctypes.Structure):
    _fields_ = [
        ("message", ctypes.c_char_p),
        ("klass", ctypes.c_int),
    ]

libgit2 = ctypes.CDLL("libgit2.so")
libgit2.giterr_last.restype = ctypes.POINTER(GitError)

error = libgit2.giterr_last()
if error:
    print(f"Error: {error.contents.message.decode()}")
```

Both are unsafe, but Rust makes it explicit.

## Building Safe Abstractions

The goal of `unsafe` is to build safe abstractions. `RefWithFlag` exposes a safe API despite using unsafe internally:

```rust
// Safe to use!
let vec = vec![10, 20, 30];
let flagged = RefWithFlag::new(&vec, true);
assert_eq!(flagged.get_ref()[1], 20);
assert_eq!(flagged.get_flag(), true);
```

Users can't:
- Create invalid `RefWithFlag` instances
- Violate the invariants
- Access the raw pointer

The unsafe code is encapsulated in the implementation. The public API is safe.

### PhantomData: Teaching the Compiler

```rust
use std::marker::PhantomData;

pub struct RefWithFlag<'a, T> {
    ptr_and_bit: usize,
    behaves_like: PhantomData<&'a T>  // Zero-size marker
}
```

`PhantomData<&'a T>` tells the compiler:
- This type behaves like it owns a `&'a T`
- Apply lifetime and variance rules accordingly
- Even though we only store `usize`

Without `PhantomData`, the compiler wouldn't enforce the lifetime correctly. This is an unsafe superpower used to create safe abstractions.

## When to Use Unsafe

Use unsafe when:

1. **Interfacing with C code** (FFI)
2. **Performance-critical code** (after profiling!)
3. **Low-level data structures** (implementing collections)
4. **Building safe abstractions** (encapsulating unsafe operations)

Avoid unsafe when:
1. **Safe alternatives exist** (use `Vec` instead of raw pointers)
2. **Performance difference is negligible** (profile first!)
3. **You're not sure** (ask for help)

## Unsafe Guidelines

When writing unsafe code:

1. **Document invariants** - Comment what must remain true
2. **Minimize unsafe blocks** - Keep them small and focused
3. **Encapsulate in safe APIs** - Don't expose unsafe to users
4. **Test extensively** - Unsafe bugs are hard to find
5. **Use Miri** - Rust's interpreter detects undefined behavior
6. **Audit carefully** - Have others review your unsafe code

### Example: Documenting Invariants

```rust
/// A reference and a boolean packed into one word.
///
/// # Safety Invariants
///
/// - T must be at least 2-byte aligned (checked at construction)
/// - The stored pointer must remain valid for lifetime 'a
/// - The pointer must not be mutated while this exists
pub struct RefWithFlag<'a, T> {
    ptr_and_bit: usize,
    behaves_like: PhantomData<&'a T>
}
```

Make invariants explicit.

## Comparison to Python

| Aspect | Python | Rust |
|--------|--------|------|
| C interop | ctypes (implicitly unsafe) | FFI (explicitly unsafe) |
| Raw pointers | Not exposed | Exposed with `unsafe` |
| Segfault protection | None (can crash) | Safe code can't segfault |
| Unsafe boundary | No distinction | Explicit `unsafe` blocks |
| Thread safety | Runtime (hope for best) | Compile-time (`Send`/`Sync`) |
| Memory safety | GC + runtime checks | Compile-time + unsafe escapes |

Python is "unsafe all the way down" - any C extension can crash your program. Rust isolates unsafety to explicit blocks, allowing you to audit and verify them.

## Real-World Example: libgit2

The libgit2 wrapper demonstrates:

1. **Calling C functions** (unsafe)
2. **Managing C resources** (manual free)
3. **Converting between C and Rust types** (CString, CStr)
4. **Handling C errors** (status codes)
5. **Wrapping in safe functions** (check helper)

```rust
unsafe {
    check("initializing library", raw::git_libgit2_init());

    let mut repo = ptr::null_mut();
    check("opening repository",
          raw::git_repository_open(&mut repo, path.as_ptr()));

    // ... use repo ...

    raw::git_repository_free(repo);
    check("shutting down library", raw::git_libgit2_shutdown());
}
```

The `unsafe` block makes it clear: this code requires manual verification. Everything outside the block is guaranteed safe by the compiler.

## Key Takeaways

1. **Unsafe is an escape hatch** - Not "unsafe code", but "unchecked code"
2. **Most of Rust is still checked** - Unsafe doesn't disable all safety
3. **Build safe abstractions** - Encapsulate unsafe internals with safe APIs
4. **Document invariants** - Make requirements explicit
5. **FFI is always unsafe** - C doesn't have Rust's guarantees
6. **Explicit is better** - `unsafe` blocks show where to audit
7. **Safe code can't segfault** - Only bugs in unsafe code cause UB

Unsafe Rust is like Python's ctypes: a way to interoperate with unsafe code while maintaining safety at the boundaries. The difference is Rust makes the boundary explicit and verifiable, while Python leaves it implicit and easy to misuse. When used correctly, unsafe enables zero-cost abstractions that are safe to use and fast to run.
