# Raw Pointers & Pointer Arithmetic Internals

**Target Audience**: AI agents working with unsafe code, systems programming, and memory manipulation
**Prerequisites**: Understanding of ownership, unsafe code, and memory layout
**Scope**: Pointer creation, arithmetic, dereferencing, common bugs, and optimization implications

---

## 1. Raw Pointer Fundamentals

### Creating Pointers from References

```rust
let value = 42i32;

// Immutable pointer from reference
let const_ptr: *const i32 = &value;

// Mutable pointer from mutable reference
let mut value_mut = 42i32;
let mut_ptr: *mut i32 = &mut value_mut;

// Dangling pointer (DO NOT USE)
let dangling: *const i32 = std::ptr::null();
```

### Creating Pointers from Integers

```rust
// Hardware registers or memory-mapped I/O
const GPIO_ADDRESS: usize = 0x40000000;
let gpio_ptr = GPIO_ADDRESS as *const u32;

// Dereferencing requires safety verification
unsafe {
    let value = *gpio_ptr;
}
```

### Null Pointer Handling

```rust
let ptr: *const i32 = std::ptr::null();
let ptr_int = std::ptr::null_mut::<i32>();

// Safe null check
if ptr.is_null() {
    println!("Pointer is null");
}
```

## 2. Pointer Arithmetic & Address Calculation

### add/offset Operations

```rust
unsafe fn pointer_arithmetic_demo() {
    let arr = [1i32, 2, 3, 4, 5];
    let ptr = arr.as_ptr();

    // Element-wise offset
    let elem2 = *ptr.add(2);  // arr[2] = 3

    // Wrapping arithmetic
    let wrapped = ptr.wrapping_add(1000);  // Wraps without panic
}
```

### Byte-Level vs Element-Level Arithmetic

```rust
unsafe fn mixed_arithmetic() {
    let bytes = [1u8, 2, 3, 4];
    let byte_ptr = bytes.as_ptr();

    // Element-level: moves by size_of::<u8>() = 1
    let elem1 = *byte_ptr.add(1);  // bytes[1]

    // For larger types:
    let i32_arr = [100i32, 200, 300];
    let i32_ptr = i32_arr.as_ptr();

    // Element-level: moves by size_of::<i32>() = 4
    let elem2 = *i32_ptr.add(1);  // i32_arr[1] = 200
    // Address moved by 4 bytes (assuming 32-bit i32)
}
```

### Alignment-Preserving Arithmetic

```rust
unsafe fn alignment_aware() {
    let arr = [1u32, 2u32, 3u32];
    let ptr = arr.as_ptr();

    // Safe: add() preserves alignment for Sized types
    let offset_ptr = ptr.add(1);

    // Verify alignment before dereferencing
    let alignment = std::mem::align_of::<u32>();
    if (offset_ptr as usize) % alignment == 0 {
        let value = *offset_ptr;  // Safe to dereference
    }
}
```

## 3. Pointer Casting & Type Punning

### as Casts on Pointers

```rust
unsafe fn pointer_casts() {
    let value = 0x12345678_u32;
    let ptr: *const u32 = &value;

    // Widening cast
    let wider: *const u64 = ptr as *const u64;  // UNSAFE: may access beyond bounds

    // Narrowing cast
    let narrower: *const u8 = ptr as *const u8;  // OK: points within bounds
}
```

### Transmute vs Cast for Pointers

```rust
unsafe fn transmute_vs_cast() {
    // Using as cast (simpler, type-checks alignment)
    let ptr = 0x40000000_usize as *const u32;

    // Using transmute (explicit, no implicit conversions)
    let ptr2 = std::mem::transmute::<usize, *const u32>(0x40000000);

    // Transmute is rarely needed for pointers
}
```

## 4. Pointer-to-Pointer Conversions

### Valid Conversions

```rust
unsafe fn valid_conversions() {
    let value = 42i32;

    // Const to const (same type)
    let const_ptr: *const i32 = &value;
    let const_ptr2: *const i32 = const_ptr;  // OK

    // Mut to const (valid, loses mutability)
    let mut value_mut = 42i32;
    let mut_ptr: *mut i32 = &mut value_mut;
    let const_ptr3: *const i32 = mut_ptr as *const i32;  // OK

    // Mut to mut (same type)
    let mut_ptr2: *mut i32 = mut_ptr;  // OK
}
```

### Invalid Conversions

```rust
unsafe fn invalid_conversions() {
    let value = 42i32;
    let const_ptr: *const i32 = &value;

    // ERROR: const to mut (compiler prevents)
    // let mut_ptr: *mut i32 = const_ptr as *mut i32;

    // Workaround (only safe if value is actually mutable):
    let mut mut_value = 42i32;
    let const_ptr2: *const i32 = &mut_value;
    let mut_ptr: *mut i32 = const_ptr2 as *mut i32;  // UNSAFE but possible
}
```

## 5. Unsafe Dereferencing & Validity

### What "Valid for Reads" Means

```rust
unsafe fn valid_for_reads() {
    let arr = [1i32, 2, 3, 4];
    let ptr = arr.as_ptr();

    // Valid for reads at indices 0-3
    for i in 0..4 {
        let value = *ptr.add(i);  // Safe
    }

    // Invalid read (out of bounds)
    // let invalid = *ptr.add(4);  // UNDEFINED BEHAVIOR
}
```

### Alignment Requirements

```rust
unsafe fn alignment_requirements() {
    let bytes = [1u8, 2u8, 3u8, 4u8, 5u8, 6u8];
    let byte_ptr = bytes.as_ptr();

    // Offset 2: points to bytes[2]
    // i32 requires 4-byte alignment
    let misaligned: *const i32 = (byte_ptr.add(2)) as *const i32;

    // Verify alignment before dereferencing
    let alignment = std::mem::align_of::<i32>();
    if (misaligned as usize) % alignment == 0 {
        let _value = *misaligned;  // Safe
    } else {
        println!("Misaligned pointer!");
    }
}
```

### Initialization Requirements

```rust
unsafe fn uninitialized_check() {
    // WRONG: Reading uninitialized memory
    let uninitialized: *const u32 = std::mem::MaybeUninit::uninit().as_ptr() as *const u32;
    // let value = *uninitialized;  // UNDEFINED BEHAVIOR

    // CORRECT: Use MaybeUninit
    let mut maybe_uninit = std::mem::MaybeUninit::<u32>::uninit();
    maybe_uninit.write(42);
    let initialized_ptr = maybe_uninit.as_ptr();
    let value = *initialized_ptr;  // Safe
}
```

## 6. Advanced Pointer Patterns

### Intrusive Linked Lists

```rust
use std::ptr::NonNull;

struct LinkedList<T> {
    head: Option<NonNull<Node<T>>>,
}

struct Node<T> {
    value: T,
    next: Option<NonNull<Node<T>>>,
}

impl<T> LinkedList<T> {
    fn push_front(&mut self, value: T) {
        let mut node = Box::new(Node {
            value,
            next: self.head,
        });

        self.head = Some(NonNull::new(&mut *node).unwrap());
        std::mem::forget(node);  // Leak intentionally
    }
}
```

### Custom Vec-Like Implementations

```rust
struct SimpleVec<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
}

impl<T> SimpleVec<T> {
    fn new() -> Self {
        SimpleVec {
            ptr: std::ptr::null_mut(),
            len: 0,
            capacity: 0,
        }
    }

    fn push(&mut self, value: T) {
        if self.len == self.capacity {
            self.grow();
        }

        unsafe {
            let addr = self.ptr.add(self.len);
            std::ptr::write(addr, value);
        }

        self.len += 1;
    }

    fn grow(&mut self) {
        let new_cap = if self.capacity == 0 { 1 } else { self.capacity * 2 };
        let new_ptr = unsafe {
            if self.capacity == 0 {
                std::alloc::alloc(
                    std::alloc::Layout::array::<T>(new_cap).unwrap()
                ) as *mut T
            } else {
                std::alloc::realloc(
                    self.ptr as *mut u8,
                    std::alloc::Layout::array::<T>(self.capacity).unwrap(),
                    std::mem::size_of::<T>() * new_cap,
                ) as *mut T
            }
        };

        self.ptr = new_ptr;
        self.capacity = new_cap;
    }
}

impl<T> Drop for SimpleVec<T> {
    fn drop(&mut self) {
        if self.capacity != 0 {
            unsafe {
                std::alloc::dealloc(
                    self.ptr as *mut u8,
                    std::alloc::Layout::array::<T>(self.capacity).unwrap(),
                );
            }
        }
    }
}
```

## 7. Common Bugs & Detection

### Out-of-Bounds Pointer Arithmetic

```rust
unsafe fn out_of_bounds_bug() {
    let arr = [1i32, 2, 3];
    let ptr = arr.as_ptr();

    // BUG: Accessing beyond array bounds
    // let invalid = *ptr.add(10);  // UNDEFINED BEHAVIOR
}
```

### Use-After-Free

```rust
unsafe fn use_after_free_bug() {
    let ptr = {
        let value = Box::new(42);
        Box::into_raw(value)
    };  // value is dropped

    // BUG: Dereferencing after free
    // let value = *ptr;  // UNDEFINED BEHAVIOR
}
```

### Off-by-One Errors in Field Offset Calculations

```rust
unsafe fn field_offset_bug() {
    struct Pair {
        first: u32,
        second: u32,
    }

    let pair = Pair { first: 1, second: 2 };
    let base_ptr = &pair as *const Pair as *const u32;

    // Correct: offset by 1 element (4 bytes for u32)
    let second_ptr = base_ptr.add(1);
    let value = *second_ptr;  // 2
}
```

## 8. Performance & Optimization

### Provenance and Pointer Optimization

```rust
fn with_provenance(slice: &[i32]) -> i32 {
    let ptr = slice.as_ptr();
    unsafe {
        // Compiler knows ptr's provenance (from slice)
        *ptr.add(0) + *ptr.add(1)
    }
}

fn without_provenance(slice: &[i32]) -> i32 {
    let addr = slice.as_ptr() as usize;
    let ptr = addr as *const i32;  // Lost provenance
    unsafe {
        *ptr.add(0) + *ptr.add(1)
    }
}
```

### Dead Store Elimination

```rust
fn dead_store_example(ptr: *mut i32) {
    unsafe {
        *ptr = 10;
        *ptr = 20;  // First write may be eliminated
        *ptr = 30;
    }
}

fn volatile_prevents_elimination(ptr: *mut i32) {
    unsafe {
        std::ptr::write_volatile(ptr, 10);  // Won't be eliminated
        std::ptr::write_volatile(ptr, 20);
        std::ptr::write_volatile(ptr, 30);
    }
}
```

## AI Agent Analysis Checklist

When reviewing pointer arithmetic code:

1. ✅ **Bounds verification** - Ensure all offsets are within valid ranges
2. ✅ **Alignment checking** - Verify pointers maintain required alignment
3. ✅ **Overlap detection** - Ensure operations don't create overlapping regions
4. ✅ **Initialization** - Confirm all reads are from initialized memory
5. ✅ **Provenance** - Verify pointers are created validly
6. ✅ **Lifetime validity** - Ensure pointers don't outlive their data
7. ✅ **Use-after-free** - Detect potential dangling pointers

## Conclusion

Raw pointers are necessary for systems programming but require rigorous verification. AI agents analyzing pointer code should:

1. Verify preconditions before dereferencing
2. Check for alignment and bounds violations
3. Trace pointer creation and lifecycle
4. Detect common patterns (use-after-free, double-free, etc.)
5. Use tools like Miri to catch undefined behavior

**Key Takeaways:**
- Raw pointers lack compile-time safety checks
- Pointer arithmetic is frame-of-reference
- Alignment and bounds must be verified manually
- Provenance matters for optimization
- Tools like Miri help catch unsafe code bugs
