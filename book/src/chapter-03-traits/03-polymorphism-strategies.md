# Polymorphism Strategies

Rust provides two fundamentally different polymorphism strategies: static dispatch through monomorphization and dynamic dispatch through trait objects. This architectural decision significantly impacts performance, binary size, and design flexibility. Understanding when and how to use each approach is critical for building efficient, maintainable Rust systems.

## Static Dispatch: Monomorphization for Performance

Static dispatch resolves trait method calls at compile time through a process called monomorphization. The compiler generates specialized code for each concrete type used with a generic function, eliminating runtime overhead entirely.

### How Monomorphization Works

Consider this generic function from the `complex` project:

```rust
fn add_complex<T>(a: Complex<T>, b: Complex<T>) -> Complex<T>
where
    T: Add<Output = T>,
{
    a + b  // Calls the trait method
}

// Usage with different types
let int_result = add_complex(
    Complex { re: 1, im: 2 },
    Complex { re: 3, im: 4 }
);  // T = i32

let float_result = add_complex(
    Complex { re: 1.0, im: 2.0 },
    Complex { re: 3.0, im: 4.0 }
);  // T = f64
```

The compiler generates two separate functions:

```rust
// Compiler-generated specialization for i32
fn add_complex_i32(a: Complex<i32>, b: Complex<i32>) -> Complex<i32> {
    Complex {
        re: a.re + b.re,  // Direct i32 addition
        im: a.im + b.im,
    }
}

// Compiler-generated specialization for f64
fn add_complex_f64(a: Complex<f64>, b: Complex<f64>) -> Complex<f64> {
    Complex {
        re: a.re + b.re,  // Direct f64 addition
        im: a.im + b.im,
    }
}
```

Each specialized version uses direct method calls with no indirection, enabling aggressive compiler optimizations like inlining, constant propagation, and loop unrolling.

### Performance Characteristics

Static dispatch provides:

1. **Zero-cost abstraction**: Generic code performs identically to hand-written specialized code
2. **Inlining opportunities**: The compiler knows the exact function being called
3. **CPU optimization**: Branch prediction works better with direct calls
4. **No vtable overhead**: No runtime lookup required

Benchmark comparisons show static dispatch often matches or exceeds C++ template performance:

```rust
// Static dispatch - compiler generates optimal code
fn process_numbers<T: Numeric>(nums: &[T]) -> T
where
    T: Copy + Default + Add<Output = T>
{
    nums.iter().copied().fold(T::default(), |acc, x| acc + x)
}

// This compiles to the same assembly as:
fn process_numbers_i32(nums: &[i32]) -> i32 {
    nums.iter().copied().fold(0, |acc, x| acc + x)
}
```

### Trade-offs: Binary Size

The primary cost of monomorphization is code bloat. Each generic instantiation produces separate machine code:

```rust
// This generic function
fn complex_operation<T: Numeric>(x: T, y: T) -> T { /* ... */ }

// Used with 5 types generates 5 copies of the function's machine code
complex_operation(1i32, 2i32);
complex_operation(1.0f32, 2.0f32);
complex_operation(1.0f64, 2.0f64);
complex_operation(1i64, 2i64);
complex_operation(1u32, 2u32);
```

For heavily generic libraries, this can significantly increase binary size. The standard library's `Vec<T>` generates dozens of specialized implementations across a typical program.

## Dynamic Dispatch: Trait Objects for Flexibility

Dynamic dispatch uses trait objects to enable runtime polymorphism. Instead of generating specialized code, the compiler creates a virtual method table (vtable) that dispatches method calls at runtime.

### Trait Objects in Practice

The `basic-router` project demonstrates trait objects effectively:

```rust
type BoxedCallback = Box<dyn Fn(&Request) -> Response>;

struct BasicRouter {
    routes: HashMap<String, BoxedCallback>
}

impl BasicRouter {
    fn add_route<C>(&mut self, url: &str, callback: C)
    where
        C: Fn(&Request) -> Response + 'static
    {
        self.routes.insert(url.to_string(), Box::new(callback));
    }

    fn handle_request(&self, request: &Request) -> Response {
        match self.routes.get(&request.url) {
            None => not_found_response(),
            Some(callback) => callback(request)  // Dynamic dispatch
        }
    }
}
```

The `Box<dyn Fn(&Request) -> Response>` is a trait object. At runtime, calling `callback(request)` looks up the actual function pointer in the vtable and invokes it.

### Why Trait Objects?

The router needs to store different callback types in a single HashMap:

```rust
router.add_route("/", |_| get_form_response());
router.add_route("/gcd", |req| get_gcd_response(req));
```

These closures have different types (different capture environments), making static dispatch impossible. The HashMap can't store `Callback1` and `Callback2` directly—it needs a uniform type. Trait objects provide this uniformity.

### Trait Object Mechanics

A trait object consists of two pointers:

```rust
// Conceptual representation
struct TraitObject {
    data: *mut (),      // Pointer to the actual data
    vtable: *const ()   // Pointer to the vtable
}
```

The vtable contains function pointers for all trait methods:

```rust
// Conceptual vtable for Fn(&Request) -> Response
struct Vtable {
    drop: fn(*mut ()),                          // Destructor
    size: usize,                                // Type size
    align: usize,                               // Type alignment
    call: fn(*mut (), &Request) -> Response,    // The trait method
}
```

When you call a method on a trait object, the runtime:
1. Dereferences the vtable pointer
2. Looks up the method's function pointer
3. Calls the function with the data pointer

This indirection costs approximately 1-2 nanoseconds per call on modern CPUs—negligible for I/O operations but significant in tight loops.

### Object Safety Requirements

Not all traits can become trait objects. A trait is object-safe only if:

1. **All methods return concrete types or `Self` by reference**: Methods can't return `Self` by value
2. **No generic methods**: Generic methods can't be dispatched dynamically
3. **No associated constants**: Only associated types are allowed

Example of object-unsafe traits:

```rust
// NOT object-safe - returns Self by value
trait Clone {
    fn clone(&self) -> Self;  // Can't know size at runtime
}

// NOT object-safe - generic method
trait Convert {
    fn convert<T>(&self) -> T;  // Can't generate vtable for infinite T
}

// Object-safe version
trait Drawable {
    fn draw(&self);  // Returns concrete type, no generics
}
```

The `basic-router`'s `Fn` trait is object-safe because:
- It returns a concrete type `Response`
- It has no generic methods (the signature is fixed)
- It works with references

## Decision Framework: Static vs Dynamic Dispatch

Use this decision tree to choose the appropriate strategy:

```
┌─────────────────────────────────────┐
│ Do you need to store heterogeneous  │
│ types in the same collection?       │
└─────────┬───────────────────────────┘
          │
    Yes ──┤
          │      ┌─────────────────────────────────────┐
          └────→ │ Use DYNAMIC DISPATCH (trait objects)│
                 └─────────────────────────────────────┘
                 Examples:
                 - BasicRouter storing different closures
                 - Plugin system with various implementations
                 - GUI elements of different types

    No ───┐
          │      ┌─────────────────────────┐
          │      │ Is the trait object-safe?│
          │      └──────────┬───────────────┘
          │                 │
          │           Yes ──┤
          │                 │      ┌──────────────────────────────┐
          │                 │      │ Do you need runtime selection │
          │                 │      │ of implementation?            │
          │                 │      └──────┬────────────────────────┘
          │                 │             │
          │                 │       Yes ──┤
          │                 │             │    ┌─────────────────────┐
          │                 │             └──→ │ Use DYNAMIC DISPATCH│
          │                 │                  └─────────────────────┘
          │                 │
          │                 │       No ───┐
          │                 │             │    ┌─────────────────────┐
          │                 │             └──→ │ Use STATIC DISPATCH │
          │                 │                  └─────────────────────┘
          │                 │
          │           No ───┘
          │                       ┌─────────────────────┐
          └─────────────────────→ │ Use STATIC DISPATCH │
                                  └─────────────────────┘
```

### Concrete Decision Criteria

| Criterion | Static Dispatch | Dynamic Dispatch |
|-----------|----------------|------------------|
| **Performance-critical inner loops** | ✅ Preferred | ❌ Avoid |
| **Collections of different types** | ❌ Impossible | ✅ Required |
| **Binary size concerns** | ⚠️ May bloat | ✅ Smaller |
| **Trait has generic methods** | ✅ Works | ❌ Impossible |
| **Compile time known types** | ✅ Optimal | ⚠️ Less optimal |
| **Plugin/extensibility systems** | ❌ Limited | ✅ Flexible |
| **Method returns Self by value** | ✅ Works | ❌ Impossible |

## Performance Implications and Trade-offs

### Benchmarking Static vs Dynamic

Consider this comparison:

```rust
// Static dispatch
fn process_static<T: Numeric>(values: &[T]) -> T
where
    T: Copy + Default + Add<Output = T>
{
    values.iter().copied().sum()  // Inlined, optimized
}

// Dynamic dispatch
fn process_dynamic(values: &[Box<dyn Numeric>]) -> i32 {
    values.iter().map(|v| v.as_i32()).sum()  // Vtable calls
}
```

Benchmark results (1 million element array):
- **Static dispatch**: ~1.2ms (fully vectorized by compiler)
- **Dynamic dispatch**: ~8.5ms (vtable overhead prevents vectorization)

The 7x difference stems from:
1. Indirect function calls preventing inlining
2. Reduced CPU branch prediction accuracy
3. Missed optimization opportunities (vectorization, loop unrolling)

### When Dynamic Dispatch Doesn't Matter

For I/O-bound operations, vtable overhead is negligible:

```rust
// Router example - request handling dominates
fn handle_request(&self, request: &Request) -> Response {
    match self.routes.get(&request.url) {
        Some(callback) => callback(request)  // <1ns vtable lookup
                                             // ~1000ns request processing
    }
}
```

Network latency (microseconds to milliseconds) dwarfs the nanosecond vtable cost, making dynamic dispatch perfectly acceptable.

### Hybrid Approaches

Combine both strategies for optimal results:

```rust
// Static dispatch for computation
impl<T: Add<Output = T>> Complex<T> {
    fn add_static(self, other: Self) -> Self {
        self + other  // Static dispatch, optimized
    }
}

// Dynamic dispatch for storage
struct ComplexContainer {
    items: Vec<Box<dyn ComplexOps>>  // Heterogeneous storage
}
```

This uses static dispatch for performance-critical operations and dynamic dispatch only where type erasure is necessary.

## Architectural Patterns

### Pattern 1: Type Erasure for Extensibility

```rust
trait Plugin {
    fn execute(&self, input: &str) -> String;
}

struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>
}

impl PluginManager {
    fn run_all(&self, input: &str) -> Vec<String> {
        self.plugins.iter()
            .map(|p| p.execute(input))
            .collect()
    }
}
```

This pattern enables runtime-loaded plugins without recompilation.

### Pattern 2: Static Dispatch with Enum

Avoid dynamic dispatch for closed type sets:

```rust
enum Callback {
    Form(fn() -> Response),
    Gcd(fn(&Request) -> Response),
    // All variants known at compile time
}

impl Callback {
    fn call(&self, req: &Request) -> Response {
        match self {
            Callback::Form(f) => f(),
            Callback::Gcd(f) => f(req),
        }
    }
}
```

This provides static dispatch performance with heterogeneous storage.

## Key Takeaways

1. **Static dispatch is the default**: Use it unless you need dynamic behavior
2. **Trait objects enable heterogeneous collections**: The only way to store different types together
3. **Object safety matters**: Check trait compatibility with `dyn` usage
4. **Performance differences are real**: Profile before choosing dynamic dispatch in hot paths
5. **Hybrid approaches work well**: Use each strategy where it excels

Understanding these polymorphism strategies enables architectural decisions that balance performance, flexibility, and maintainability—core concerns in systems programming.

## Cross-References

- **Section 3.2**: Composition patterns often use static dispatch
- **Section 3.5**: Case studies demonstrate both strategies in practice
- **Chapter 5**: Error handling uses trait objects for `Box<dyn Error>`
