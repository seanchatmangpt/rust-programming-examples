# Phantom Types and Compile-Time Constraints

Phantom types are one of Rust's most sophisticated architectural patterns—types that exist solely at compile time, carrying information without occupying runtime memory. They enable the type-state pattern, where invalid state transitions become compile errors rather than runtime panics. This is type-driven architecture at its most elegant: impossible to misuse because the compiler won't let you.

## Understanding Phantom Types

A phantom type is a type parameter that appears in a struct's definition but not in its fields:

```rust
use std::marker::PhantomData;

struct Buffer<State> {
    data: Vec<u8>,
    _state: PhantomData<State>,  // Zero-sized, compile-time only
}

// State markers - no data, just types
struct Empty;
struct Filled;
struct Sealed;
```

The `PhantomData<State>` field exists only for the type system. In compiled code, it vanishes—it's truly zero-cost. Yet at compile time, `Buffer<Empty>` and `Buffer<Filled>` are distinct types with different capabilities.

## The Type-State Pattern: Making Invalid States Impossible

Traditional state machines use runtime checks:

```rust
// Runtime state machine - can fail at runtime
struct Connection {
    state: ConnectionState,
    socket: TcpStream,
}

enum ConnectionState {
    Disconnected,
    Connected,
    Authenticated,
}

impl Connection {
    pub fn send(&mut self, data: &[u8]) -> Result<()> {
        if self.state != ConnectionState::Authenticated {
            return Err("Not authenticated".into());  // Runtime error!
        }
        self.socket.write_all(data)
    }
}
```

This design has critical flaws:
- **Runtime Overhead**: Every operation checks state
- **Partial Correctness**: Easy to forget checks
- **API Ambiguity**: Signature doesn't show which states are valid

The type-state pattern eliminates these issues by encoding states as types:

```rust
use std::marker::PhantomData;

// State markers
struct Disconnected;
struct Connected;
struct Authenticated;

// Connection parameterized by state
struct Connection<State> {
    socket: TcpStream,
    _state: PhantomData<State>,
}

// Only authenticated connections can send
impl Connection<Authenticated> {
    pub fn send(&mut self, data: &[u8]) -> Result<()> {
        self.socket.write_all(data)  // No runtime check needed!
    }
}

// State transitions consume and return new states
impl Connection<Disconnected> {
    pub fn connect(self, addr: &str) -> Result<Connection<Connected>> {
        let socket = TcpStream::connect(addr)?;
        Ok(Connection {
            socket,
            _state: PhantomData,
        })
    }
}

impl Connection<Connected> {
    pub fn authenticate(self, credentials: &Credentials)
        -> Result<Connection<Authenticated>>
    {
        // Perform authentication...
        Ok(Connection {
            socket: self.socket,
            _state: PhantomData,
        })
    }
}
```

Now invalid state transitions are **compile errors**:

```rust
let conn = Connection::new();  // Connection<Disconnected>
conn.send(data);  // ❌ Compile error: no method `send` for Connection<Disconnected>

// Must follow correct sequence:
let conn = conn.connect("127.0.0.1:8080")?;  // Connection<Connected>
let conn = conn.authenticate(&creds)?;       // Connection<Authenticated>
conn.send(data)?;  // ✅ Compiles—guaranteed authenticated
```

This is architectural correctness enforced by the compiler.

## Compile-Time Properties Without Runtime Cost

The beauty of phantom types is their complete erasure at runtime. Consider this builder pattern with type-state:

```rust
struct Builder<HasName, HasEmail> {
    name: Option<String>,
    email: Option<String>,
    _has_name: PhantomData<HasName>,
    _has_email: PhantomData<HasEmail>,
}

// Type-level booleans
struct Yes;
struct No;

impl Builder<No, No> {
    pub fn new() -> Self {
        Builder {
            name: None,
            email: None,
            _has_name: PhantomData,
            _has_email: PhantomData,
        }
    }
}

impl<E> Builder<No, E> {
    pub fn name(self, name: String) -> Builder<Yes, E> {
        Builder {
            name: Some(name),
            email: self.email,
            _has_name: PhantomData,
            _has_email: PhantomData,
        }
    }
}

impl<N> Builder<N, No> {
    pub fn email(self, email: String) -> Builder<N, Yes> {
        Builder {
            name: self.name,
            email: Some(email),
            _has_name: PhantomData,
            _has_email: PhantomData,
        }
    }
}

// Only complete builders can build
impl Builder<Yes, Yes> {
    pub fn build(self) -> User {
        User {
            name: self.name.unwrap(),  // Safe—type guarantees Some
            email: self.email.unwrap(),
        }
    }
}
```

Usage enforces completeness:

```rust
// ❌ Compile error—missing email
let user = Builder::new()
    .name("Alice".into())
    .build();

// ✅ Compiles—all required fields provided
let user = Builder::new()
    .name("Alice".into())
    .email("alice@example.com".into())
    .build();
```

The compiler tracks which fields have been set. At runtime, there's zero overhead—no `Option` checks, no state flags, just the raw data.

## Phantom Types for Units and Dimensions

Phantom types excel at encoding physical units:

```rust
struct Quantity<Unit> {
    value: f64,
    _unit: PhantomData<Unit>,
}

// Unit markers
struct Meters;
struct Seconds;
struct MetersPerSecond;

// Type-safe operations
impl Quantity<Meters> {
    pub fn new(value: f64) -> Self {
        Quantity { value, _unit: PhantomData }
    }
}

impl Quantity<Seconds> {
    pub fn new(value: f64) -> Self {
        Quantity { value, _unit: PhantomData }
    }
}

// Division creates new unit type
impl std::ops::Div<Quantity<Seconds>> for Quantity<Meters> {
    type Output = Quantity<MetersPerSecond>;

    fn div(self, rhs: Quantity<Seconds>) -> Self::Output {
        Quantity {
            value: self.value / rhs.value,
            _unit: PhantomData,
        }
    }
}
```

Now unit errors are compile errors:

```rust
let distance = Quantity::<Meters>::new(100.0);
let time = Quantity::<Seconds>::new(10.0);
let velocity = distance / time;  // Quantity<MetersPerSecond>

let wrong = distance / distance;  // ❌ Compile error:
                                   // no impl for Div<Quantity<Meters>> for Quantity<Meters>
```

NASA famously lost the Mars Climate Orbiter due to unit confusion ($327 million mistake). Phantom types prevent such disasters at compile time.

## The Builder Pattern with Type-State

The builder pattern becomes bulletproof with type-state. Traditional builders allow incomplete objects:

```rust
// Traditional builder - can forget required fields
let config = ConfigBuilder::new()
    .timeout(30)
    // Oops, forgot to set endpoint!
    .build();  // Runtime panic or undefined behavior
```

Type-state builders make this impossible:

```rust
struct ConfigBuilder<HasEndpoint, HasTimeout> {
    endpoint: Option<String>,
    timeout: Option<u64>,
    _has_endpoint: PhantomData<HasEndpoint>,
    _has_timeout: PhantomData<HasTimeout>,
}

impl ConfigBuilder<No, No> {
    pub fn new() -> Self { /* ... */ }
}

impl<T> ConfigBuilder<No, T> {
    pub fn endpoint(self, url: String) -> ConfigBuilder<Yes, T> { /* ... */ }
}

impl<E> ConfigBuilder<E, No> {
    pub fn timeout(self, secs: u64) -> ConfigBuilder<E, Yes> { /* ... */ }
}

impl ConfigBuilder<Yes, Yes> {
    pub fn build(self) -> Config {
        Config {
            endpoint: self.endpoint.unwrap(),  // Safe!
            timeout: self.timeout.unwrap(),
        }
    }
}
```

The compiler enforces completeness:

```rust
let config = ConfigBuilder::new()
    .timeout(30)
    .build();  // ❌ Compile error: no method `build` for ConfigBuilder<No, Yes>

let config = ConfigBuilder::new()
    .endpoint("https://api.example.com".into())
    .timeout(30)
    .build();  // ✅ Compiles
```

## Practical Example: File Handle Type-State

File handles are a perfect type-state use case. Files can be opened for reading, writing, or both, with different operations valid in each mode:

```rust
use std::marker::PhantomData;
use std::fs::File;

struct Read;
struct Write;

struct FileHandle<Mode> {
    file: File,
    _mode: PhantomData<Mode>,
}

impl FileHandle<Read> {
    pub fn open_read(path: &str) -> std::io::Result<Self> {
        Ok(FileHandle {
            file: File::open(path)?,
            _mode: PhantomData,
        })
    }

    pub fn read_line(&mut self, buf: &mut String) -> std::io::Result<usize> {
        use std::io::BufRead;
        let mut reader = std::io::BufReader::new(&self.file);
        reader.read_line(buf)
    }
}

impl FileHandle<Write> {
    pub fn create(path: &str) -> std::io::Result<Self> {
        Ok(FileHandle {
            file: File::create(path)?,
            _mode: PhantomData,
        })
    }

    pub fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        use std::io::Write;
        self.file.write_all(buf)
    }
}
```

Usage is type-safe:

```rust
let mut f = FileHandle::<Read>::open_read("input.txt")?;
f.read_line(&mut buffer)?;  // ✅ OK
f.write_all(b"data")?;      // ❌ Compile error: no method `write_all`

let mut f = FileHandle::<Write>::create("output.txt")?;
f.write_all(b"data")?;  // ✅ OK
f.read_line(&mut buffer)?;  // ❌ Compile error: no method `read_line`
```

No runtime checks, no possibility of using a file handle incorrectly.

## Variance and Phantom Types

Phantom types interact subtly with Rust's variance system. The `PhantomData<T>` type has special variance properties:

```rust
// Invariant in T
struct Invariant<T> {
    _marker: PhantomData<fn(T) -> T>,
}

// Covariant in T
struct Covariant<T> {
    _marker: PhantomData<fn() -> T>,
}

// Contravariant in T
struct Contravariant<T> {
    _marker: PhantomData<fn(T)>,
}
```

For most type-state applications, you want invariance to prevent unexpected subtyping:

```rust
struct Buffer<State> {
    _state: PhantomData<fn(State) -> State>,  // Invariant
}
```

This prevents a `Buffer<Authenticated>` from being used where `Buffer<Connected>` is expected, even if there were some subtyping relationship.

## When to Use Phantom Types: Decision Framework

### Use Phantom Types When:

1. **State Machines**: Object has distinct states with different valid operations
2. **Builder Patterns**: Some fields must be set before building
3. **Type-Level Tracking**: Need to track properties without runtime data
4. **Unit Safety**: Preventing unit confusion in scientific/financial code

### Avoid Phantom Types When:

1. **Dynamic State**: State depends on runtime data (use enums)
2. **Simple Validation**: A single check suffices (use newtype pattern)
3. **Performance Critical**: Compile-time cost matters (rare)

### Red Flags:

- State transitions aren't linear (use enum state machines)
- Many possible states (combinatorial explosion in impl blocks)
- State depends on external factors (can't encode in types)

## Integration with Existing Types

Phantom types compose well with Rust's existing abstractions. The standard library's `std::marker::PhantomData` is designed for this:

```rust
use std::marker::PhantomData;

// Phantom types work with lifetimes
struct Ref<'a, T, State> {
    ptr: &'a T,
    _state: PhantomData<State>,
}

// And with Send/Sync bounds
struct Channel<T, State> {
    _data: PhantomData<fn() -> T>,
    _state: PhantomData<State>,
}

unsafe impl<T: Send, State> Send for Channel<T, State> {}
```

## Cross-References

Phantom types build on concepts from:

- **Chapter 1: Newtypes** - Foundation for type-driven design
- **Chapter 3: Traits** - Implementing state-specific behavior
- **Chapter 8: Lifetimes** - Phantom types with lifetime parameters

## Conclusion

Phantom types represent the pinnacle of compile-time verification. By encoding state and properties in the type system, you create APIs that are impossible to misuse. The compiler becomes your co-architect, enforcing correct usage through type checking rather than runtime validation.

This is the essence of type-driven architecture: leveraging the type system not just for correctness, but as a primary design tool. Invalid states become unrepresentable, state transition errors become compile errors, and runtime checks vanish—replaced by compile-time guarantees.

The cost? Zero. Phantom types are completely erased at runtime. The benefit? Entire categories of bugs eliminated before your code ever runs.
