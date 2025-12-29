# Composition Patterns with Traits

Rust's trait system enables powerful composition patterns that build complex behaviors from simple, reusable components. Unlike inheritance-based systems, trait composition maintains flexibility without creating rigid hierarchies. This chapter explores three key composition strategies: trait stacking, default implementations, and blanket implementations, demonstrating how these patterns create maintainable, extensible architectures.

## Trait Stacking and Layering

Trait stacking allows types to implement multiple independent traits, building sophisticated capabilities through composition rather than inheritance. This architectural pattern provides maximum flexibility while maintaining clear separation of concerns.

### Multiple Independent Capabilities

The `complex` project demonstrates trait stacking clearly:

```rust
#[derive(Clone, Copy, Debug)]
struct Complex<T> {
    re: T,
    im: T,
}

impl<T: Add<Output = T>> Add for Complex<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Complex {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl<T: Sub<Output = T>> Sub for Complex<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Complex {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}

impl<T> Mul for Complex<T>
where
    T: Clone + Add<Output = T> + Sub<Output = T> + Mul<Output = T>,
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Complex {
            re: self.re.clone() * rhs.re.clone()
                - (self.im.clone() * rhs.im.clone()),
            im: self.im * rhs.re + self.re * rhs.im,
        }
    }
}
```

Each arithmetic operation is an independent trait implementation. A `Complex<i32>` automatically supports all three operations because `i32` satisfies their requirements. However, a hypothetical `Complex<OnlyAddable>` would only support addition—the architecture adapts to the component type's capabilities.

### Compositional Dependencies

Some trait implementations depend on others, creating compositional layers:

```rust
// From complex project: AddAssign builds on Add
impl<T> AddAssign for Complex<T>
where
    T: AddAssign<T>,
{
    fn add_assign(&mut self, rhs: Complex<T>) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}
```

Notice this doesn't require `Add`—`AddAssign` is independent. However, conventionally, types implementing `AddAssign` also implement `Add`, creating a logical composition of capabilities.

### Trait Bounds as Composition Requirements

Complex trait bounds specify compositional requirements explicitly:

```rust
// Multiplication requires four composed capabilities
impl<T> Mul for Complex<T>
where
    T: Clone + Add<Output = T> + Sub<Output = T> + Mul<Output = T>,
{
    // Implementation uses all four traits
}
```

This pattern composes simple traits (`Clone`, `Add`, `Sub`, `Mul`) into a complex requirement, demonstrating how trait bounds enable compositional architecture.

## Default Implementations as Provided Behavior

Default trait implementations provide reusable behavior that implementors can override when needed. This pattern balances code reuse with customization flexibility.

### Standard Library Examples

The `PartialEq` trait demonstrates this pattern:

```rust
trait PartialEq<Rhs = Self> {
    fn eq(&self, other: &Rhs) -> bool;

    // Default implementation provided
    fn ne(&self, other: &Rhs) -> bool {
        !self.eq(other)
    }
}
```

Implementors only need to define `eq`—the `ne` method comes for free. However, if there's a more efficient implementation of inequality, types can override it.

### Custom Traits with Defaults

When designing your own traits, default implementations reduce boilerplate:

```rust
trait Drawable {
    // Required method
    fn draw(&self);

    // Default implementation
    fn draw_with_border(&self) {
        println!("===Border Start===");
        self.draw();
        println!("===Border End===");
    }
}

struct Circle { radius: f64 }

impl Drawable for Circle {
    fn draw(&self) {
        println!("Circle: radius {}", self.radius);
    }

    // draw_with_border comes for free
}
```

This pattern creates extensible architectures where common functionality lives in the trait, reducing duplication across implementations.

### Architectural Benefits

Default implementations provide several architectural advantages:

1. **DRY Principle**: Common logic lives in one place
2. **Consistent Behavior**: Default implementations ensure uniform behavior across types
3. **Customization Points**: Critical cases can override defaults for optimization
4. **Evolution-Friendly**: Adding new default methods doesn't break existing implementations

## Blanket Implementations for Broad Abstraction

Blanket implementations provide trait implementations for entire categories of types, creating powerful abstractions with minimal code. This advanced pattern demonstrates the full power of Rust's trait system.

### Automatic Capability Propagation

The standard library uses blanket implementations extensively:

```rust
// From std library: any T that implements Deref<Target=U>
// automatically implements all of U's methods
impl<T: Deref<Target = U>, U> SomeTrait for T
where
    U: SomeTrait
{
    // Forward implementation to deref target
}
```

This pattern enables smart pointers like `Box<T>` and `Rc<T>` to automatically support any trait that `T` supports.

### Conditional Trait Implementation

Blanket implementations can add capabilities conditionally:

```rust
// Any type implementing Debug and Display also implements ShowBoth
impl<T: Debug + Display> ShowBoth for T {
    fn show_both(&self) {
        println!("Debug: {:?}", self);
        println!("Display: {}", self);
    }
}
```

This creates emergent capabilities—any type that implements both `Debug` and `Display` automatically gains `ShowBoth` without explicit implementation.

### Architectural Pattern: Trait Extension

A powerful pattern uses blanket implementations to extend trait capabilities:

```rust
trait Numeric: Add + Sub + Mul + Div {
    // Marker trait requiring arithmetic
}

// Automatic implementation for any type with all operations
impl<T> Numeric for T
where
    T: Add + Sub + Mul + Div
{}

// Now extend with more capabilities
trait NumericExt: Numeric {
    fn square(self) -> Self
    where
        Self: Sized + Copy + Mul<Output = Self>,
    {
        self * self
    }

    fn double(self) -> Self
    where
        Self: Sized + Copy + Add<Output = Self>,
    {
        self + self
    }
}

// Blanket implementation: all Numeric types get NumericExt
impl<T: Numeric> NumericExt for T {}
```

Any type implementing the four basic arithmetic traits automatically implements both `Numeric` and `NumericExt`, gaining `square` and `double` methods. This creates hierarchical capability layers through composition.

## Real Composition Examples from Projects

### Example 1: Complex Number Layers

The `complex` project's architecture layers traits progressively:

```rust
// Layer 1: Basic structure (any T)
#[derive(Clone, Copy, Debug)]
struct Complex<T> { re: T, im: T }

// Layer 2: Equality (requires T: PartialEq)
impl<T: PartialEq> PartialEq for Complex<T> { /* ... */ }
impl<T: Eq> Eq for Complex<T> {}

// Layer 3: Arithmetic (various T requirements)
impl<T: Add<Output = T>> Add for Complex<T> { /* ... */ }
impl<T: Neg<Output = T>> Neg for Complex<T> { /* ... */ }

// Layer 4: Complex arithmetic (multiple trait requirements)
impl<T: Clone + Add + Sub + Mul> Mul for Complex<T> { /* ... */ }

// Layer 5: Display (requires specific formatting)
impl Display for Complex<f64> { /* ... */ }
```

Each layer builds on previous capabilities, creating a progression from simple to complex without rigid hierarchies.

### Example 2: Router Composition

The `basic-router` project demonstrates closure trait composition:

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
}
```

This composes several traits:
- `Fn(&Request) -> Response`: Function capability
- `'static`: Lifetime requirement for storage
- Boxing enables trait object polymorphism (see Chapter 3.3)

The architecture accepts any callable matching the signature, enabling flexible request handling.

### Example 3: Interval Ordering

The `interval` project shows partial trait implementation:

```rust
impl<T: PartialOrd> PartialOrd<Interval<T>> for Interval<T> {
    fn partial_cmp(&self, other: &Interval<T>) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self.lower >= other.upper {
            Some(Ordering::Greater)
        } else if self.upper <= other.lower {
            Some(Ordering::Less)
        } else {
            None  // Overlapping intervals have no ordering
        }
    }
}
```

This implements `PartialOrd` but not `Ord`, demonstrating that trait composition allows selective capability implementation. Overlapping intervals can't be totally ordered, so the architecture correctly implements only partial ordering.

## Composition Strategy Decision Matrix

| Pattern | Use When | Example |
|---------|----------|---------|
| **Trait Stacking** | Multiple independent capabilities | `Complex<T>`: separate `Add`, `Sub`, `Mul` implementations |
| **Default Implementations** | Common behavior with override option | `PartialEq::ne` provided automatically |
| **Blanket Implementations** | Automatic capability for trait combinations | `T: Debug + Display` gains `ShowBoth` |
| **Conditional Methods** | Capabilities only for constrained types | `Complex<T: Ord>` has sorting methods |
| **Trait Extension** | Hierarchical capability layers | `NumericExt` builds on `Numeric` |

## Architectural Principles

1. **Favor Composition Over Inheritance**: Stack traits instead of building type hierarchies
2. **Provide Defaults Judiciously**: Only when behavior is universally correct
3. **Use Blanket Impls for Abstraction**: Automatically provide capabilities for trait combinations
4. **Keep Traits Focused**: Small, single-purpose traits compose better than monolithic ones
5. **Document Compositional Requirements**: Trait bounds are architectural contracts

These composition patterns, demonstrated across the repository's projects, enable flexible, maintainable architectures that adapt to changing requirements without extensive refactoring.

## Cross-References

- **Section 3.1**: Traits as interfaces provide the foundation for these composition patterns
- **Section 3.3**: Polymorphism strategies determine when trait objects enable composition
- **Chapter 2**: Generic types form the base for trait composition
