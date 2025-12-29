# Compiler Intrinsics & LLVM Integration

**Target Audience**: AI agents optimizing performance-critical code, working with low-level operations
**Prerequisites**: Understanding of Rust unsafe code, LLVM basics
**Scope**: Intrinsics, SIMD, inline assembly, code generation

---

## 1. Compiler Intrinsics Overview

**Compiler intrinsics** are special functions that provide direct access to low-level CPU instructions and compiler operations. Unlike regular functions, intrinsics are recognized by the compiler and translated directly to specific instruction sequences.

### What Are Intrinsics?

Intrinsics bridge the gap between high-level Rust code and low-level hardware capabilities:

```rust
use std::intrinsics;

// Regular function: has call overhead
fn regular_sqrt(x: f64) -> f64 {
    x.sqrt()
}

// Intrinsic: compiles to single CPU instruction (sqrtsd on x86)
unsafe fn intrinsic_sqrt(x: f64) -> f64 {
    std::intrinsics::sqrtf64(x)
}
```

### When to Use Intrinsics vs Regular Functions

**Use intrinsics when**:
- Performance is critical and profiling shows bottlenecks
- You need specific hardware instructions not exposed otherwise
- Working with hardware registers or memory-mapped I/O
- Implementing low-level primitives (atomics, SIMD)

**Use regular functions when**:
- Portability across platforms matters
- Code clarity is a priority
- The performance difference is negligible

## 2. Common Intrinsics

### Math Intrinsics

Standard mathematical operations compiled to single instructions:

```rust
use std::intrinsics::*;

unsafe fn math_intrinsics_demo() {
    let x = 16.0_f64;

    // Square root: sqrtsd instruction
    let sqrt_result = sqrtf64(x);  // 4.0

    // Floating-point manipulation
    let abs_val = fabsf64(-3.14);  // 3.14
    let floor_val = floorf64(3.7); // 3.0
    let ceil_val = ceilf64(3.2);   // 4.0

    // Fused multiply-add: single instruction
    let fma = fmaf64(2.0, 3.0, 4.0);  // 10.0
}
```

### Bit Manipulation Intrinsics

Efficient bit-level operations:

```rust
use std::intrinsics::*;

unsafe fn bit_intrinsics_demo() {
    let value: u64 = 0b1010_1100_0000_0001;

    // Count trailing zeros: tzcnt instruction
    let trailing_zeros = cttz(value);  // 0

    // Count leading zeros: lzcnt instruction
    let leading_zeros = ctlz(value);   // 48

    // Population count (number of 1 bits): popcnt instruction
    let ones = ctpop(value);           // 5

    // Byte swap: bswap instruction
    let swapped = bswap(0x1234_5678_u32);  // 0x7856_3412
}
```

### Memory Operation Intrinsics

Direct memory manipulation:

```rust
use std::intrinsics::*;

unsafe fn memory_intrinsics_demo() {
    let src = [1u8, 2, 3, 4];
    let mut dst = [0u8; 4];

    // Non-overlapping copy
    copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), 4);

    // Volatile read (prevents compiler optimization)
    let hardware_register = 0x40000000 as *const u32;
    let value = volatile_load(hardware_register);

    // Volatile write
    let output_register = 0x40000004 as *mut u32;
    volatile_store(output_register, 0xFF);
}
```

## 3. SIMD Intrinsics

### Vector Types and Operations

SIMD (Single Instruction, Multiple Data) processes multiple values simultaneously:

```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "x86_64")]
unsafe fn simd_addition_demo() {
    // Process 4 f32 values at once with SSE
    let a = _mm_set_ps(1.0, 2.0, 3.0, 4.0);
    let b = _mm_set_ps(5.0, 6.0, 7.0, 8.0);
    let result = _mm_add_ps(a, b);  // [6.0, 8.0, 10.0, 12.0]

    // Process 8 f32 values at once with AVX
    if is_x86_feature_detected!("avx") {
        let a = _mm256_set_ps(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
        let b = _mm256_set_ps(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
        let result = _mm256_add_ps(a, b);
    }
}
```

### Portable SIMD

The `std::simd` module provides portable SIMD (requires nightly):

```rust
#![feature(portable_simd)]
use std::simd::*;

fn portable_simd_demo() {
    // Works on any platform with SIMD support
    let a = f32x8::from_array([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
    let b = f32x8::splat(1.0);
    let result = a + b;

    // Horizontal operations
    let sum = result.reduce_sum();
    let max = result.reduce_max();
}
```

## 4. Inline Assembly

### asm! Macro Syntax

Direct assembly code embedded in Rust:

```rust
use std::arch::asm;

unsafe fn inline_asm_demo() {
    let x: u64 = 10;
    let y: u64;

    // Intel syntax (default)
    asm!(
        "mov {0}, {1}",
        "add {0}, 5",
        out(reg) y,
        in(reg) x,
    );
    assert_eq!(y, 15);
}
```

### Register Allocation Hints

```rust
use std::arch::asm;

unsafe fn register_hints_demo() {
    let a: u64 = 10;
    let b: u64 = 20;
    let result: u64;

    // Suggest specific registers for optimal performance
    asm!(
        "add {result}, {a}",
        result = inout("rax") b => result,  // Use rax
        a = in("rcx") a,                     // Prefer rcx
    );
}
```

## 5. LLVM IR & Code Generation

### LLVM Intermediate Representation

Rust compiles to LLVM IR before machine code:

```rust
// Rust source
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Emitted LLVM IR (simplified):
// define i32 @add(i32 %a, i32 %b) {
//   %result = add nsw i32 %a, %b
//   ret i32 %result
// }
```

View IR with:
```bash
cargo rustc -- --emit=llvm-ir
```

### Optimization Passes

LLVM applies ~100+ optimization passes:
- **Dead Code Elimination**: Removes unused code
- **Constant Propagation**: Evaluates constants at compile time
- **Inlining**: Replaces function calls with function body
- **Loop Unrolling**: Duplicates loop body for performance
- **Vectorization**: Converts scalar to SIMD operations

## 6. Optimization Hints for LLVM

### #[inline] Attributes

```rust
// Always inline (small, hot functions)
#[inline(always)]
pub fn always_inlined(x: i32) -> i32 {
    x * 2
}

// Hint to inline (LLVM decides)
#[inline]
pub fn maybe_inlined(x: i32) -> i32 {
    x * x
}

// Never inline (debugging, large functions)
#[inline(never)]
pub fn never_inlined(x: i32) -> i32 {
    // Complex logic...
    x
}
```

### #[cold] for Cold Code Paths

```rust
#[cold]
fn handle_error(msg: &str) -> ! {
    eprintln!("Error: {}", msg);
    std::process::exit(1);
}

pub fn process(value: i32) -> i32 {
    if value < 0 {
        handle_error("Negative value");  // Marked cold
    }
    value * 2
}
```

## 7. Memory Ordering & Atomics

### Memory Ordering in Atomic Operations

```rust
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn ordering_examples() {
    // Relaxed: No synchronization
    COUNTER.store(1, Ordering::Relaxed);

    // Release: Prevents prior writes from moving after
    COUNTER.store(2, Ordering::Release);

    // Acquire: Prevents subsequent reads from moving before
    let value = COUNTER.load(Ordering::Acquire);

    // SeqCst: Total ordering
    COUNTER.fetch_add(1, Ordering::SeqCst);
}
```

## 8. Platform-Specific Intrinsics

### Using Feature Gates

```rust
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn dispatch_function(data: &[f32]) -> f32 {
    #[cfg(target_feature = "avx2")]
    {
        return unsafe { avx2_implementation(data) };
    }

    #[cfg(target_feature = "sse2")]
    {
        return unsafe { sse2_implementation(data) };
    }

    fallback_implementation(data)
}
```

**Runtime feature detection**:
```rust
pub fn runtime_dispatch(data: &[f32]) -> f32 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            return unsafe { avx2_implementation(data) };
        }
    }

    fallback_implementation(data)
}
```

## 9. Volatile Operations

### Volatile Reads and Writes

```rust
use std::ptr::{read_volatile, write_volatile};

// Memory-mapped hardware register
const GPIO_OUTPUT: *mut u32 = 0x4000_1000 as *mut u32;

unsafe fn control_hardware() {
    // Write to output register
    write_volatile(GPIO_OUTPUT, 0xFF);

    // Read from input register (must be volatile)
    let sensor_value = read_volatile(GPIO_OUTPUT);
}
```

### Volatile NOT a Replacement for Synchronization

**Key difference**:
- **Volatile**: Prevents compiler optimization (reordering, caching)
- **Atomic**: Provides hardware-level synchronization between threads

```rust
// ❌ WRONG: Volatile doesn't prevent data races
static mut COUNTER: u32 = 0;

unsafe fn wrong_increment() {
    let value = std::ptr::read_volatile(&COUNTER);
    std::ptr::write_volatile(&mut COUNTER, value + 1);
    // Race condition: read-modify-write not atomic
}

// ✅ CORRECT: Use atomic operations
use std::sync::atomic::{AtomicU32, Ordering};
static COUNTER: AtomicU32 = AtomicU32::new(0);

fn correct_increment() {
    COUNTER.fetch_add(1, Ordering::SeqCst);
}
```

## 10. AI Agent Intrinsic Analysis

### Verifying Intrinsic Preconditions

AI agents should check:

```rust
// Analysis checklist for unsafe intrinsics:

// ✅ 1. Pointer validity
unsafe fn check_pointer_validity(ptr: *const u8, len: usize) -> bool {
    !ptr.is_null() && ptr.is_aligned()
}

// ✅ 2. Alignment requirements
unsafe fn check_alignment<T>(ptr: *const T) -> bool {
    ptr.align_offset(std::mem::align_of::<T>()) == 0
}

// ✅ 3. Memory overlap
unsafe fn check_no_overlap(
    src: *const u8, src_len: usize,
    dst: *const u8, dst_len: usize
) -> bool {
    let src_end = src.add(src_len);
    let dst_end = dst.add(dst_len);
    src_end <= dst || dst_end <= src
}

// ✅ 4. Platform feature availability
fn check_feature_available() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        is_x86_feature_detected!("avx2")
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}
```

### Understanding Performance Implications

AI agents should recognize:

```rust
// Anti-pattern: Unnecessary intrinsic
fn bad_example(x: i32) -> i32 {
    unsafe { std::intrinsics::add_with_overflow(x, 1).0 }
}
// Better: Use regular addition
fn good_example(x: i32) -> i32 {
    x.wrapping_add(1)  // Clearer, equally efficient
}

// Good use case: Performance-critical SIMD
#[target_feature(enable = "avx2")]
unsafe fn justified_intrinsic(data: &[f32]) -> f32 {
    // 8x parallelism justifies complexity
    sum_simd_avx(data)
}
```

## Conclusion

Compiler intrinsics and LLVM integration provide powerful tools for performance-critical code. For AI agents analyzing intrinsic usage:

1. **Verify preconditions** rigorously (alignment, overlap, validity)
2. **Prefer safe alternatives** when performance difference is negligible
3. **Provide fallback implementations** for platform portability
4. **Benchmark systematically** to validate performance claims
5. **Test with Miri and sanitizers** to catch undefined behavior

**Key Takeaways:**
- Intrinsics compile to single or few CPU instructions
- SIMD provides 4-16x parallelism for data-parallel operations
- Inline assembly offers direct hardware control
- Memory ordering is critical for correctness
- Platform-specific optimizations require feature gates and fallbacks
