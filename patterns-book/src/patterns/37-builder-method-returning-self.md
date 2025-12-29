# 37. BUILDER METHOD RETURNING SELF

*A potter at the wheel, each motion shaping the clay further—spinning, pulling, smoothing—each action returning the evolving form to your hands for the next transformation*

...within a **CONSTRUCTOR FUNCTION NAMED NEW (36)** or **STRUCT WITH NAMED FIELDS (15)**, when you need to configure a type through multiple optional steps rather than a single overwhelming initialization...

◆ ◆ ◆

**How do you configure complex types when initialization has many optional parameters, without forcing users to pass dozens of arguments or create invalid intermediate states?**

Complex types often have many configuration options. If you put them all in the constructor, you get unwieldy signatures: `new(a, b, c, d, e, f, g, h, i, j)`. Default values help, but Rust doesn't have keyword arguments or function overloading. Creating the struct directly exposes all fields, including those that should be private.

The builder pattern solves this by consuming `self`, modifying it, and returning it. Each method call advances the configuration. The user chains calls together in a fluid syntax: `Type::new().with_x(1).with_y(2).build()`. The methods take ownership using `mut self`, ensuring you can't accidentally use an intermediate state.

This ownership transfer is central to Rust's design. Unlike other languages where builders might use setters that mutate in place, Rust builders move the value through each method. This prevents you from holding references to partially-built objects and eliminates a whole class of errors.

The pattern appears throughout the standard library (Option's and/or, Result's map) and in web frameworks like Actix where request builders chain dozens of configuration methods.

**Therefore:**

**Provide methods that take `mut self`, modify internal state, and return `self`, enabling fluent chaining that terminates in a consuming method or direct use.**

```rust
// From actix-gcd/src/main.rs
use actix_web::{web, App, HttpServer};

let server = HttpServer::new(|| {
    App::new()
        .route("/", web::get().to(get_index))
        .route("/gcd", web::post().to(post_gcd))
});

// Conceptual builder pattern:
impl Builder {
    pub fn with_capacity(mut self, cap: usize) -> Self {
        self.capacity = cap;
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn build(self) -> Result<FinalType, Error> {
        // Consume self, produce final type
        FinalType::from_builder(self)
    }
}

// Usage:
let result = Builder::new()
    .with_capacity(100)
    .with_name("example".into())
    .build()?;
```

*Each method holds the form briefly, shapes it, then passes it forward—a chain of transformations that flows like water*

◆ ◆ ◆

This works with **CONSTRUCTOR FUNCTION NAMED NEW (36)** as the starting point, connects to **FUNCTION RETURNING RESULT (38)** when the final `build()` might fail, and supports **METHOD TAKING SELF (22)** for the consuming transformation pattern.
