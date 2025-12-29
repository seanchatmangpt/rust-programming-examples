# 6. ASYNC RUNTIME WITH ATTRIBUTE MAIN

*A server application starting with a simple attribute macro, transforming the entry point into an async context*

...within a **WEB SERVICE SKELETON**, when you need to handle concurrent network requests with async/await but the main function must be synchronous by default...

◆ ◆ ◆

**The fundamental tension: Rust's main function cannot be async, yet web servers must run on an async runtime to efficiently handle concurrent connections. Manual runtime initialization clutters your code with boilerplate that obscures the actual application logic.**

Every web service needs an event loop—a runtime that can wake tasks when I/O completes, schedule concurrent operations, and manage thousands of connections efficiently. But Rust's `fn main()` is synchronous by design. You could write `fn main()` and manually initialize an async runtime, calling `block_on` to execute async code, but this forces you to expose runtime details at the application boundary.

Look at what happens without the attribute: you must create a runtime, configure it, then block on your application's future. The ceremony of runtime initialization becomes the first thing developers see, even though it's infrastructural noise—the same in nearly every async application. The essence of your program (starting a server, binding to a port) drowns in mechanical setup.

The actix-web project demonstrates this pattern. The `#[actix_web::main]` attribute transforms `async fn main()` into valid Rust, generating the runtime setup code automatically. The macro expands to create a runtime, execute the async main body, and cleanly shut down when the application exits.

**Therefore:**

**Annotate your main function with the runtime's attribute macro (like `#[actix_web::main]` or `#[tokio::main]`), allowing you to write `async fn main()` directly. Let the macro generate the runtime initialization boilerplate.**

```rust
use actix_web::{web, App, HttpResponse, HttpServer};

#[actix_web::main]
async fn main() {
    let server = HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(get_index))
            .route("/gcd", web::post().to(post_gcd))
    });

    println!("Serving on http://localhost:3000...");
    server
        .bind("127.0.0.1:3000").expect("error binding server to address")
        .run()
        .await
        .expect("error running server");
}

async fn get_index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(/* ... */)
}
```

*The attribute macro wraps the function, expanding to runtime initialization code that remains invisible to the developer, creating a clean application entry point*

◆ ◆ ◆

This pattern naturally leads to **ROUTE HANDLERS** and **ASYNC HTTP REQUEST**, where the established async context enables concurrent request processing throughout your application. The runtime initialized here becomes the foundation for all async operations in your web service.
