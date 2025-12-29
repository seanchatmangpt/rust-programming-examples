# 29. DERIVE DESERIALIZE FOR FORMS

*A struct that automatically extracts itself from incoming data, like a key that knows how to unlock its own lock.*

...within a **WEB SERVER HANDLER** or **DATA PARSING CONTEXT**, when you need to convert incoming data (form posts, JSON, query parameters) into structured types...

◆ ◆ ◆

**How can you extract structured data from HTTP requests without writing tedious parsing code for every field?**

When a web form submits data, it arrives as a flat collection of name-value pairs. To use this data, you must extract each field, parse it to the correct type, handle missing or invalid values, and assemble a struct. For a form with five fields, this means fifteen lines of error-prone boilerplate:

```rust
let n = form.get("n").ok_or("missing n")?.parse::<u64>()?;
let m = form.get("m").ok_or("missing m")?.parse::<u64>()?;
// Repeat for every field...
```

Serde's `Deserialize` trait automates this entirely. Define a struct whose fields match the form field names, derive Deserialize, and the framework automatically parses the request into your struct. Missing fields, type errors, and validation all happen before your handler runs. Your code receives a valid, typed struct.

This pattern works for HTML forms, JSON APIs, query parameters, configuration files—anywhere data enters your program from the outside world. The web framework uses Serde to deserialize directly into your type, converting `&str` to `u64`, handling errors, and enforcing your structure.

**Therefore:**

**Define a struct matching your input schema and derive Deserialize to automatically parse incoming data.**

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct GcdParameters {
    n: u64,
    m: u64,
}

async fn post_gcd(form: web::Form<GcdParameters>) -> HttpResponse {
    if form.n == 0 || form.m == 0 {
        return HttpResponse::BadRequest()
            .content_type("text/html")
            .body("Computing the GCD with zero is boring.");
    }

    let response = format!(
        "The greatest common divisor of {} and {} is <b>{}</b>",
        form.n, form.m, gcd(form.n, form.m)
    );

    HttpResponse::Ok()
        .content_type("text/html")
        .body(response)
}
```

*Deserialize transforms raw form data into a validated struct automatically, like a customs inspector who checks and processes documents before letting them through.*

◆ ◆ ◆

This pattern often appears with **DERIVE DEBUG FOR TESTING** (28) for debugging request data, and uses the same derive mechanism as **DERIVE COPY FOR STACK TYPES** (20).
