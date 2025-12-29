# Unsafe Rust & Memory Layout Deep Dive

**Target Audience**: AI agents reviewing, writing, or analyzing unsafe Rust code
**Prerequisites**: Understanding of Rust ownership, borrowing, and lifetimes
**Scope**: Advanced unsafe patterns, memory representation, verification techniques

---

## 1. The Unsafe Boundary

### 1.1 Why Unsafe Exists

Unsafe Rust is not "turning off the borrow checker" — it's establishing a **contract** between the programmer and the compiler. The borrow checker operates on conservative assumptions. Sometimes these assumptions are too restrictive for valid operations that humans (or AI agents) can verify manually.

```rust
// Safe Rust: Conservative but correct
fn get_two_mut<T>(slice: &mut [T], i: usize, j: usize) -> Option<(&mut T, &mut T)> {
    // Borrow checker rejects: can't have two mutable references to slice
    // Some((&mut slice[i], &mut slice[j]))  // ❌ Won't compile
    None  // We're forced to give up
}

// Unsafe Rust: Precise invariants with human verification
fn get_two_mut_unsafe<T>(slice: &mut [T], i: usize, j: usize) -> Option<(&mut T, &mut T)> {
    if i == j || i >= slice.len() || j >= slice.len() {
        return None;
    }

    unsafe {
        // SAFETY: We have verified:
        // 1. i and j are distinct (i != j)
        // 2. Both indices are in bounds (i < len, j < len)
        // 3. Therefore, the mutable references don't alias
        let ptr = slice.as_mut_ptr();
        Some((
            &mut *ptr.add(i),
            &mut *ptr.add(j),
        ))
    }
}
```

**Key Insight for AI Agents**: The `unsafe` keyword does not disable safety checks. It enables operations that require **manual verification** of invariants the compiler cannot prove automatically.

### 1.2 The Five Unsafe Superpowers

Unsafe Rust grants exactly five additional capabilities:

1. **Dereference raw pointers** (`*const T`, `*mut T`)
2. **Call unsafe functions** (including FFI functions)
3. **Implement unsafe traits** (e.g., `Send`, `Sync`)
4. **Access mutable statics**
5. **Access fields of unions**

Everything else remains checked. For example, you **cannot**:
- Violate lifetime rules (references still must be valid)
- Bypass type checking (transmute still requires size equality)
- Create undefined behavior "safely" (UB is still UB)

### 1.3 SAFETY Comments: The Contract

Every unsafe block must document its safety contract in a `// SAFETY:` comment.

```rust
pub struct Ascii([u8]);

impl Ascii {
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Ascii {
        // SAFETY: Caller must ensure all bytes are ASCII (0x00..=0x7F).
        // Ascii is repr(transparent) over [u8], so the transmute is layout-safe.
        std::mem::transmute(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<&Ascii> {
        if bytes.iter().all(|&b| b <= 0x7F) {
            // SAFETY: We just verified all bytes are valid ASCII.
            Some(unsafe { Self::from_bytes_unchecked(bytes) })
        } else {
            None
        }
    }
}
```

**AI Agent Checklist for SAFETY Comments**:
- [ ] Does the comment state **what** invariants must hold?
- [ ] Does the comment state **why** those invariants make the code safe?
- [ ] Can the invariants be verified by examining the calling context?
- [ ] Are there any implicit assumptions (alignment, initialization, etc.)?

### 1.4 Invariants AI Agents Must Verify

When reviewing unsafe code, verify these invariants:

**Memory Safety**:
- Pointers are valid (non-null, aligned, pointing to allocated memory)
- Pointers are initialized before dereferencing
- Lifetimes are respected (data outlives all references)
- No aliasing violations (mutable references are exclusive)

**Type Safety**:
- Transmutes preserve size and alignment
- Bit patterns are valid for the target type
- No invalid enum discriminants
- No dangling references

**Thread Safety**:
- Types sent across threads are `Send`
- Types shared across threads are `Sync`
- No data races (conflicting access without synchronization)

---

## 2. Memory Layout & Representation

### 2.1 Default Layout: Unspecified and Optimized

Rust **does not guarantee** struct field order or padding by default. The compiler may reorder fields for optimization.

```rust
use std::mem::{size_of, align_of};

#[derive(Debug)]
struct Example {
    a: u8,   // 1 byte, 1-byte aligned
    b: u32,  // 4 bytes, 4-byte aligned
    c: u16,  // 2 bytes, 2-byte aligned
}

fn main() {
    println!("Size: {}", size_of::<Example>());      // Likely 8, NOT 7
    println!("Align: {}", align_of::<Example>());    // Likely 4

    // Field order in memory is UNSPECIFIED
    // Compiler might reorder to: b, c, a (then pad to alignment)
}
```

**Why This Matters**: If you write unsafe code that assumes field order, it will break when the compiler changes optimizations.

### 2.2 repr(C): C-Compatible Layout

Use `#[repr(C)]` for FFI or when layout must be predictable.

```rust
#[repr(C)]
struct CStruct {
    a: u8,   // Offset 0
    // 3 bytes padding
    b: u32,  // Offset 4
    c: u16,  // Offset 8
    // 2 bytes padding to align to 4-byte boundary
}

// Size: 12 (not 7)
// Alignment: 4 (alignment of largest field)
```

**Field Ordering with repr(C)**:
- Fields appear in declaration order
- Padding added to satisfy alignment
- Struct's alignment = max alignment of fields
- Size rounded up to multiple of alignment

### 2.3 repr(packed): Remove Padding

`#[repr(packed)]` removes all padding, aligning fields to 1-byte boundaries.

```rust
#[repr(packed)]
struct Packed {
    a: u8,   // Offset 0
    b: u32,  // Offset 1 (UNALIGNED!)
    c: u16,  // Offset 5
}
// Size: 7 (exactly sum of field sizes)
```

**⚠️ DANGER**: Accessing unaligned fields is **undefined behavior** on many architectures.

```rust
#[repr(packed)]
struct Packed {
    a: u8,
    b: u32,
}

fn bad_access(p: &Packed) {
    let x = p.b;  // ❌ UB: Creates reference to unaligned u32
}

fn safe_access(p: &Packed) {
    let x = unsafe { std::ptr::addr_of!(p.b).read_unaligned() };  // ✅ OK
}
```

**AI Agent Rule**: When you see `#[repr(packed)]`, verify that all field accesses use `addr_of!` or `read_unaligned`, never direct field references.

### 2.4 repr(align(N)): Increase Alignment

Force a type to have minimum alignment:

```rust
#[repr(align(64))]  // Cache line alignment
struct CacheLine {
    data: [u8; 64],
}

// Ensures this struct starts on a 64-byte boundary
// Prevents false sharing in multi-threaded code
```

### 2.5 Zero-Copy Serialization Implications

Unsafe code often aims for zero-copy deserialization. This requires:

1. **Known layout** (`#[repr(C)]`)
2. **No invalid bit patterns** (all bit patterns valid for type)
3. **No references** (pointers are not portable across processes)

```rust
#[repr(C)]
struct Header {
    magic: u32,
    version: u16,
    flags: u16,
}

unsafe fn parse_header(bytes: &[u8]) -> Option<&Header> {
    if bytes.len() < std::mem::size_of::<Header>() {
        return None;
    }

    // SAFETY:
    // 1. Header is repr(C), so layout is known
    // 2. All bit patterns are valid for u32/u16 fields
    // 3. bytes.len() >= size_of::<Header>()
    // 4. Alignment: we must verify bytes.as_ptr() is aligned
    if (bytes.as_ptr() as usize) % std::mem::align_of::<Header>() != 0 {
        return None;  // Not aligned, would be UB
    }

    Some(&*(bytes.as_ptr() as *const Header))
}
```

---

## 3. Transmute Semantics & Type Safety

### 3.1 What Transmute Does

`std::mem::transmute<T, U>(value: T) -> U` reinterprets the bits of `T` as type `U`. It is the ultimate escape hatch and extremely dangerous.

```rust
let x: u32 = 0x3f800000;
let f: f32 = unsafe { std::mem::transmute(x) };
println!("{}", f);  // 1.0 (IEEE 754 representation of 1.0)
```

**Compiler Guarantees**:
- `size_of::<T>() == size_of::<U>()` (enforced at compile time)

**Programmer Responsibilities**:
- Bit pattern of `T` must be valid for `U`
- Alignment requirements must be compatible
- Lifetimes must be preserved

### 3.2 Safe Transmute Patterns

**Pattern 1: Primitive Casts**
```rust
// ✅ SAFE: All bit patterns of u32 are valid f32 (NaN is valid)
let bits: u32 = 0x3f800000;
let float: f32 = unsafe { std::mem::transmute(bits) };

// ✅ SAFE: Pointer to usize (same size on target platform)
let ptr = &value as *const i32;
let addr: usize = unsafe { std::mem::transmute(ptr) };
```

**Pattern 2: Array to Array**
```rust
// ✅ SAFE: [u8; 4] and u32 have same size, all bit patterns valid
let bytes: [u8; 4] = [0x00, 0x00, 0x80, 0x3f];
let value: u32 = unsafe { std::mem::transmute(bytes) };
```

**Pattern 3: repr(transparent) Wrappers**
```rust
#[repr(transparent)]
struct Ascii([u8]);

// ✅ SAFE: Ascii is repr(transparent), identical layout to [u8]
unsafe fn cast_to_ascii(bytes: &[u8]) -> &Ascii {
    std::mem::transmute(bytes)
}
```

### 3.3 Dangerous Transmute Patterns

**Anti-Pattern 1: Lifetime Extension**
```rust
// ❌ UB: Extends lifetime artificially
fn evil_lifetime<'a>(s: &str) -> &'a str {
    unsafe { std::mem::transmute(s) }
}
// This creates a dangling reference when the original data is freed
```

**Anti-Pattern 2: Invalid Enum Discriminants**
```rust
enum Bool {
    False = 0,
    True = 1,
}

let x: u8 = 2;
let b: Bool = unsafe { std::mem::transmute(x) };  // ❌ UB: 2 is not a valid Bool
```

**Anti-Pattern 3: Uninitialized Memory**
```rust
// ❌ UB: Transmuting uninitialized memory
let x: i32 = unsafe { std::mem::transmute([0u8; 4]) };  // OK (0 is valid)
let x: bool = unsafe { std::mem::transmute(2u8) };      // ❌ UB (only 0 or 1 valid)
```

### 3.4 Size and Alignment Verification

Always verify size equality before transmute:

```rust
fn safe_transmute<T, U>(value: T) -> Result<U, T> {
    if std::mem::size_of::<T>() != std::mem::size_of::<U>() {
        return Err(value);
    }

    // SAFETY: Sizes are equal. Caller must verify bit patterns are valid.
    Ok(unsafe { std::mem::transmute_copy(&value) })
}
```

**Note**: Use `transmute_copy` to avoid move semantics issues.

### 3.5 Endianness Considerations

```rust
#[repr(C)]
struct NetworkHeader {
    value: u32,
}

fn parse_network(bytes: &[u8; 4]) -> u32 {
    // ❌ WRONG: Endianness-dependent
    let header: NetworkHeader = unsafe { std::mem::transmute(*bytes) };
    header.value

    // ✅ CORRECT: Explicit endianness handling
    u32::from_be_bytes(*bytes)
}
```

---

## 4. Pointer Semantics & Aliasing Rules

### 4.1 Rust's Aliasing Model

Rust enforces **XOR aliasing**: at any given time, you can have **either**:
- **One mutable reference** (`&mut T`), **OR**
- **Any number of immutable references** (`&T`)

This prevents data races and enables optimizations.

```rust
fn aliasing_violation() {
    let mut x = 42;
    let r1 = &x;
    let r2 = &x;        // OK: Multiple immutable
    let r3 = &mut x;    // ❌ ERROR: Can't have &mut while & exists
}
```

### 4.2 Stacked Borrows Model

Rust's memory model (formalized as **Stacked Borrows**) treats references like a stack:

```rust
let mut x = 10;
let r1 = &mut x;     // Push mutable tag
*r1 += 1;            // Use r1 (valid)
let r2 = &*r1;       // Push immutable tag (r1 becomes inactive)
println!("{}", r2);  // Use r2 (valid)
// *r1 += 1;         // ❌ UB: r1 was invalidated by creating r2
```

**Stacked Borrows Rules**:
1. Creating a reference **pushes** a tag onto the stack
2. Using a reference **validates** its tag is still on the stack
3. Creating a conflicting reference **pops** older conflicting tags

**AI Agent Insight**: When reviewing unsafe code, trace the "borrow stack" to ensure no invalidated references are used.

### 4.3 Raw Pointers and Aliasing

Raw pointers (`*const T`, `*mut T`) **opt out** of aliasing rules:

```rust
let mut x = 42;
let p1: *mut i32 = &mut x;
let p2: *mut i32 = &mut x;

unsafe {
    *p1 += 1;  // ✅ OK with raw pointers (but risky!)
    *p2 += 1;  // ✅ OK with raw pointers
}
// x is now 44
```

**⚠️ WARNING**: While raw pointers allow aliasing, **dereferencing aliased mutable pointers** in a way that causes conflicting access is still UB.

```rust
unsafe {
    let p1 = &mut x as *mut i32;
    let p2 = p1;

    *p1 = 1;   // ✅ OK
    *p2 = 2;   // ✅ OK

    let val = *p1 + *p2;  // ❓ Potentially UB depending on compiler optimizations
}
```

### 4.4 Creation, Use, and Invalidation of Pointers

**Creating Pointers** (always safe):
```rust
let x = 42;
let ptr: *const i32 = &x;  // ✅ Creating pointer is always safe
```

**Dereferencing Pointers** (requires unsafe):
```rust
unsafe {
    let value = *ptr;  // Must verify: non-null, aligned, initialized, valid lifetime
}
```

**Pointer Invalidation**:
```rust
fn dangling_pointer() -> *const i32 {
    let x = 42;
    &x as *const i32  // ✅ Creating pointer is safe
}  // x is dropped here

fn use_dangling() {
    let ptr = dangling_pointer();
    unsafe {
        let value = *ptr;  // ❌ UB: Pointer is dangling
    }
}
```

### 4.5 Provenance and Pointer-to-Integer Casts

Recent Rust models introduce **provenance**: pointers carry metadata about what memory they're allowed to access.

```rust
let x = 42;
let y = 42;

let px = &x as *const i32 as usize;
let py = &y as *const i32 as usize;

// Even if px == py (same address by coincidence), the pointers
// have different provenance and can't be used interchangeably
```

**AI Agent Rule**: Assume pointer-to-integer-to-pointer casts lose provenance. Use `std::ptr::addr_of!` and keep pointers as pointers.

---

## 5. Data Races & Race Conditions

### 5.1 Data Race vs Race Condition

**Data Race** (undefined behavior in Rust):
- Two threads access the same memory
- At least one access is a write
- Accesses are not synchronized

**Race Condition** (not UB, but often a bug):
- Program behavior depends on timing
- Results are non-deterministic but all outcomes are defined

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

// ❌ DATA RACE: Undefined behavior
fn data_race() {
    let mut x = 0;
    let x_ref = &mut x as *mut i32;

    std::thread::scope(|s| {
        s.spawn(|| unsafe { *x_ref = 1; });  // Write without synchronization
        s.spawn(|| unsafe { *x_ref = 2; });  // Concurrent write = DATA RACE
    });
}

// ✅ RACE CONDITION: Defined behavior, non-deterministic result
fn race_condition() {
    let counter = Arc::new(AtomicUsize::new(0));
    let c1 = counter.clone();
    let c2 = counter.clone();

    std::thread::scope(|s| {
        s.spawn(move || { c1.fetch_add(1, Ordering::SeqCst); });
        s.spawn(move || { c2.fetch_add(1, Ordering::SeqCst); });
    });

    // counter is now 2, but order of increments is non-deterministic
    // This is a RACE CONDITION but NOT a data race
}
```

### 5.2 Send and Sync

**Send**: Type can be transferred across thread boundaries
```rust
// T: Send means ownership of T can move to another thread
fn send_example<T: Send>(value: T) {
    std::thread::spawn(move || {
        drop(value);  // value used in another thread
    });
}
```

**Sync**: Type can be shared across threads via `&T`
```rust
// T: Sync means &T can be shared across threads
// Equivalent to: T: Sync ⟺ &T: Send
fn sync_example<T: Sync>(value: &'static T) {
    std::thread::spawn(move || {
        let _ = value;  // Immutable reference used in another thread
    });
}
```

**Key Relationships**:
- `T: Sync` ⟺ `&T: Send`
- `&mut T: Send` if `T: Send`
- Most types are `Send + Sync` (primitives, `Vec`, `String`, etc.)
- `Rc<T>` is **not** `Send` or `Sync` (use `Arc<T>` instead)
- `Cell<T>` is `Send` but **not** `Sync` (interior mutability without synchronization)

### 5.3 Atomic Operations and Memory Ordering

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn increment() {
    COUNTER.fetch_add(1, Ordering::SeqCst);  // Sequentially consistent
}

fn relaxed_increment() {
    COUNTER.fetch_add(1, Ordering::Relaxed);  // No ordering guarantees
}
```

**Memory Orderings** (from strongest to weakest):
- **SeqCst**: Sequential consistency (total order across all threads)
- **AcqRel**: Acquire on load, Release on store
- **Acquire**: Synchronizes with Release stores
- **Release**: Synchronizes with Acquire loads
- **Relaxed**: No ordering guarantees (atomicity only)

**AI Agent Rule**: Unless you have a specific reason, use `Ordering::SeqCst`. Weaker orderings are optimization opportunities but require expert understanding.

### 5.4 When to Use Arc<Mutex<T>> vs Arc<T>

```rust
use std::sync::{Arc, Mutex};

// Arc<Mutex<T>>: Shared mutable state
let data = Arc::new(Mutex::new(vec![1, 2, 3]));
let data_clone = data.clone();

std::thread::spawn(move || {
    data_clone.lock().unwrap().push(4);  // Mutable access
});

// Arc<T> where T: Sync: Shared immutable state
let data = Arc::new(vec![1, 2, 3]);
let data_clone = data.clone();

std::thread::spawn(move || {
    println!("{:?}", data_clone);  // Immutable access only
});
```

**Decision Tree**:
- Need to mutate? → `Arc<Mutex<T>>`
- Only read? → `Arc<T>` (if `T: Sync`)
- Frequent writes, rare reads? → `Arc<RwLock<T>>`
- Lock-free required? → Atomic types or custom concurrent data structures

---

## 6. Common Unsafe Patterns & Pitfalls

### 6.1 Self-Referential Structs (The Pin Problem)

Self-referential structs are a major pitfall:

```rust
struct SelfRef {
    data: String,
    ptr: *const String,  // Points to self.data
}

impl SelfRef {
    fn new(s: String) -> Self {
        let mut sr = SelfRef {
            data: s,
            ptr: std::ptr::null(),
        };
        sr.ptr = &sr.data;  // ❌ PROBLEM: Moving sr invalidates this pointer!
        sr
    }
}

fn break_it() {
    let sr = SelfRef::new("hello".into());
    let sr2 = sr;  // ❌ sr.ptr now points to freed memory (sr.data moved)
}
```

**Why This Fails**: Moving `sr` to `sr2` moves `sr.data` to a new memory location, but `sr.ptr` still points to the old location.

### 6.2 Pin and Unpin

`Pin<P>` prevents moving the pointed-to value:

```rust
use std::pin::Pin;
use std::marker::PhantomPinned;

struct SelfRefSafe {
    data: String,
    ptr: *const String,
    _pin: PhantomPinned,  // Marks type as !Unpin
}

impl SelfRefSafe {
    fn new(s: String) -> Pin<Box<Self>> {
        let mut boxed = Box::pin(SelfRefSafe {
            data: s,
            ptr: std::ptr::null(),
            _pin: PhantomPinned,
        });

        // SAFETY: We never move this after pinning
        let ptr = &boxed.data as *const String;
        unsafe {
            let mut_ref = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).ptr = ptr;
        }

        boxed
    }

    fn get_data(self: Pin<&Self>) -> &str {
        &self.data
    }

    fn get_via_ptr(self: Pin<&Self>) -> &str {
        // SAFETY: Pin guarantees the data hasn't moved
        unsafe { &*self.ptr }
    }
}
```

**Pin Guarantees**:
- `Pin<P>` where `P: Deref<Target = T>` and `T: !Unpin` prevents moving `T`
- Most types implement `Unpin` (can be moved even when pinned)
- `PhantomPinned` opts out of `Unpin`

### 6.3 Unsafe Trait Implementations

Implementing `Send` and `Sync` is unsafe because you're making a contract:

```rust
use std::cell::UnsafeCell;

struct MyCell {
    value: UnsafeCell<i32>,
}

// SAFETY: MyCell can be sent across threads because i32 is Send
unsafe impl Send for MyCell {}

// NOT SAFE: MyCell uses interior mutability without synchronization
// unsafe impl Sync for MyCell {}  // ❌ Would be UB
```

**AI Agent Checklist for Unsafe Trait Impls**:
- [ ] `Send`: Can all fields be safely sent across threads?
- [ ] `Sync`: Can `&Self` be safely shared across threads?
- [ ] Are there any raw pointers or interior mutability that break assumptions?

### 6.4 Custom Allocators and Drop Implementations

```rust
use std::alloc::{alloc, dealloc, Layout};
use std::ptr;

struct RawVec<T> {
    ptr: *mut T,
    cap: usize,
}

impl<T> RawVec<T> {
    fn new() -> Self {
        RawVec {
            ptr: std::ptr::NonNull::dangling().as_ptr(),
            cap: 0,
        }
    }

    fn grow(&mut self) {
        let new_cap = if self.cap == 0 { 1 } else { self.cap * 2 };
        let new_layout = Layout::array::<T>(new_cap).unwrap();

        let new_ptr = if self.cap == 0 {
            unsafe { alloc(new_layout) as *mut T }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                std::alloc::realloc(self.ptr as *mut u8, old_layout, new_layout.size()) as *mut T
            }
        };

        self.ptr = new_ptr;
        self.cap = new_cap;
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        if self.cap > 0 {
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}
```

**Drop Safety Rules**:
- Drop must not panic (will abort if it does during unwinding)
- Drop must not access moved-from fields
- Drop must handle partial initialization

### 6.5 Pattern Matching in Unsafe Code

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
}

fn unsafe_match(msg_ptr: *const Message) {
    unsafe {
        match *msg_ptr {
            Message::Quit => {
                // SAFETY: Just read the discriminant, no issues
            }
            Message::Move { x, y } => {
                // SAFETY: Copies x and y, which are Copy types
                println!("{}, {}", x, y);
            }
        }
    }
}
```

**Pitfall**: Matching on `!Copy` types moves them:
```rust
enum Container {
    Data(String),
}

fn unsafe_match_move(ptr: *const Container) {
    unsafe {
        match *ptr {
            Container::Data(s) => {
                // ❌ MOVES the String out of *ptr, leaving it uninitialized!
                println!("{}", s);
            }
        }
    }
}
```

**Fix**: Use `ref` or match on references:
```rust
unsafe {
    match &*ptr {
        Container::Data(s) => {
            println!("{}", s);  // ✅ Borrows, doesn't move
        }
    }
}
```

---

## 7. Verification Techniques for AI Agents

### 7.1 Miri: The Interpreter That Catches UB

Miri is a Rust interpreter that detects undefined behavior at runtime:

```bash
# Install Miri
rustup +nightly component add miri

# Run tests under Miri
cargo +nightly miri test

# Run specific binary
cargo +nightly miri run
```

**What Miri Catches**:
- Out-of-bounds memory access
- Use-after-free
- Using uninitialized memory
- Violating aliasing rules (Stacked Borrows violations)
- Data races (with `-Zmiri-disable-isolation`)

**Example**:
```rust
#[test]
fn test_undefined_behavior() {
    let x = 5;
    let ptr = &x as *const i32;

    // Miri will catch this use-after-free
    drop(x);
    unsafe {
        let value = *ptr;  // ❌ Miri error: pointer is dangling
    }
}
```

### 7.2 AddressSanitizer (ASAN) and LeakSanitizer (LSAN)

For native code and FFI, use sanitizers:

```bash
# Build with AddressSanitizer
RUSTFLAGS="-Z sanitizer=address" cargo +nightly build

# Run with AddressSanitizer
RUSTFLAGS="-Z sanitizer=address" cargo +nightly test

# LeakSanitizer (memory leaks)
RUSTFLAGS="-Z sanitizer=leak" cargo +nightly test
```

**What ASAN Catches**:
- Heap buffer overflow
- Stack buffer overflow
- Use-after-free
- Use-after-return
- Double-free

### 7.3 Proof Techniques for Unsafe Blocks

**Technique 1: Invariant Documentation**
```rust
/// Invariant: `len <= cap` always holds
/// Invariant: `ptr` points to an allocation of `cap * size_of::<T>()` bytes
/// Invariant: Elements `[0..len)` are initialized, `[len..cap)` are uninitialized
struct Vec<T> {
    ptr: *mut T,
    len: usize,
    cap: usize,
}
```

**Technique 2: State Machine Proof**
```rust
enum State {
    Empty,           // len == 0, cap == 0
    Reserved,        // len == 0, cap > 0
    PartiallyFilled, // 0 < len < cap
    Full,            // len == cap
}

// Prove each operation maintains valid transitions
impl<T> Vec<T> {
    fn push(&mut self, value: T) {
        // Pre-condition: len <= cap
        // Post-condition: len <= cap (grows capacity if needed)
    }
}
```

**Technique 3: Lifetime Proofs**
```rust
fn split_at_mut<T>(slice: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    // Proof:
    // 1. Input lifetime: 'a (lifetime of slice)
    // 2. Output lifetimes: both 'a (tied to input)
    // 3. No aliasing: [0..mid) and [mid..len) don't overlap
    // 4. Soundness: Both outputs have lifetime 'a and don't alias

    assert!(mid <= slice.len());
    unsafe {
        (
            std::slice::from_raw_parts_mut(slice.as_mut_ptr(), mid),
            std::slice::from_raw_parts_mut(slice.as_mut_ptr().add(mid), slice.len() - mid),
        )
    }
}
```

### 7.4 Code Structure for Auditing

**Pattern: Unsafe Core, Safe Shell**
```rust
mod unsafe_core {
    /// SAFETY: Caller must ensure idx < len
    pub unsafe fn get_unchecked<T>(slice: &[T], idx: usize) -> &T {
        &*slice.as_ptr().add(idx)
    }
}

pub mod safe_api {
    use super::unsafe_core;

    pub fn get<T>(slice: &[T], idx: usize) -> Option<&T> {
        if idx < slice.len() {
            // SAFETY: We just verified idx < len
            Some(unsafe { unsafe_core::get_unchecked(slice, idx) })
        } else {
            None
        }
    }
}
```

**Pattern: Capability-Based Safety**
```rust
/// Token proving that index is in bounds
struct ValidIndex {
    idx: usize,
}

impl ValidIndex {
    fn new(idx: usize, len: usize) -> Option<Self> {
        if idx < len {
            Some(ValidIndex { idx })
        } else {
            None
        }
    }
}

fn get_unchecked<T>(slice: &[T], valid_idx: ValidIndex) -> &T {
    // SAFETY: ValidIndex can only be constructed if idx < len
    unsafe { &*slice.as_ptr().add(valid_idx.idx) }
}
```

### 7.5 Documentation Patterns for Auditable Unsafe Code

**Complete SAFETY Comment Template**:
```rust
/// # Safety
///
/// This function is unsafe because [reason].
///
/// ## Requirements
/// The caller must ensure:
/// - [Requirement 1]
/// - [Requirement 2]
///
/// ## Guarantees
/// If the requirements are met, this function:
/// - [Guarantee 1]
/// - [Guarantee 2]
///
/// ## Examples
/// ```
/// # use crate::example;
/// let x = 42;
/// // SAFETY: [explanation why this call is safe]
/// let result = unsafe { example(&x) };
/// ```
pub unsafe fn example<T>(ptr: *const T) -> T {
    std::ptr::read(ptr)
}
```

---

## Summary: AI Agent Unsafe Code Review Checklist

When reviewing or writing unsafe code, verify:

**Memory Safety**:
- [ ] All pointers are valid (non-null, aligned, within allocation)
- [ ] No use-after-free (lifetimes respected)
- [ ] No buffer overflows (bounds checked)
- [ ] No uninitialized memory reads

**Aliasing**:
- [ ] No mutable aliasing (unique access via `&mut`)
- [ ] Stacked Borrows not violated (no use-after-invalidation)
- [ ] Raw pointers don't create illegal references

**Type Safety**:
- [ ] Transmutes preserve size and alignment
- [ ] Bit patterns are valid for target type
- [ ] No invalid enum discriminants

**Thread Safety**:
- [ ] Send/Sync bounds are correct
- [ ] No data races (atomics or locks for shared mutable state)
- [ ] Memory ordering is appropriate

**Documentation**:
- [ ] Every unsafe block has a SAFETY comment
- [ ] Invariants are documented
- [ ] API contracts are clear

**Testing**:
- [ ] Tests run under Miri without errors
- [ ] Edge cases are tested (null, alignment, bounds)
- [ ] Concurrent tests for thread-safety

---

## Further Reading

- **The Rustonomicon**: https://doc.rust-lang.org/nomicon/
- **Unsafe Code Guidelines**: https://rust-lang.github.io/unsafe-code-guidelines/
- **Stacked Borrows**: https://github.com/rust-lang/unsafe-code-guidelines/blob/master/wip/stacked-borrows.md
- **Miri Documentation**: https://github.com/rust-lang/miri

---

**End of Chapter: Unsafe Rust & Memory Layout Deep Dive**

*This chapter is part of the comprehensive Rust programming guide for AI agents working with the rust-programming-examples repository.*
