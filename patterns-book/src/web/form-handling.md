# Form Handling

## Context

You have an HTTP service that needs to accept user input from HTML forms or URL-encoded POST requests. Users submit data that must be parsed, validated, and converted to strongly-typed Rust structures before processing. The data may be invalid, incomplete, or maliciously crafted.

## Problem

**How do you safely extract and validate form data from HTTP requests while maintaining type safety and preventing common web vulnerabilities?**

Form handling involves several challenges:
- Form data arrives as strings that must be converted to typed values
- Fields may be missing, duplicated, or contain invalid data
- Type conversion can fail (e.g., "abc" is not a valid number)
- Manual parsing is tedious and error-prone
- Validation logic should be centralized, not scattered across handlers

Traditional approaches have weaknesses:
- **Untyped dictionaries** (Python, PHP): No compile-time validation, easy to misspell keys
- **Manual parsing**: Verbose, repetitive, easy to forget edge cases
- **Reflection-based validation**: Runtime overhead, unclear error messages

## Forces

- **Type Safety vs Flexibility**: Rigid schemas catch errors early; flexible parsing handles varied input
- **Validation Timing**: Early validation (at parsing) vs late validation (in business logic)
- **Error Reporting**: Detailed errors help debugging but may leak information
- **Performance**: Parsing and validation add latency to every request
- **Reusability**: Validation rules should be reusable across endpoints

## Solution

**Use Serde's `Deserialize` trait with actix-web's `web::Form` extractor to automatically parse and validate form data into typed structs.**

The pattern has four key components:

### 1. Define a Request Schema with Serde

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct GcdParameters {
    n: u64,
    m: u64,
}
```

**Why this works:**
- `#[derive(Deserialize)]` generates parsing code at compile time
- Field names match HTML form input names exactly
- Field types determine parsing behavior (u64 parses decimal numbers)
- Missing fields or invalid types cause automatic 400 Bad Request responses

### 2. Extract Form Data with web::Form

```rust
async fn post_gcd(form: web::Form<GcdParameters>) -> HttpResponse {
    // form.n and form.m are guaranteed to be valid u64 values
    if form.n == 0 || form.m == 0 {
        return HttpResponse::BadRequest()
            .content_type("text/html")
            .body("Computing the GCD with zero is boring.");
    }

    let response = format!(
        "The greatest common divisor of the numbers {} and {} is <b>{}</b>\n",
        form.n, form.m, gcd(form.n, form.m)
    );

    HttpResponse::Ok()
        .content_type("text/html")
        .body(response)
}
```

**Type safety guarantees:**
- If parsing fails, the handler **never runs** (actix returns 400 automatically)
- No need for `.parse()` or error checking in the handler
- `form.n` and `form.m` are guaranteed valid `u64` values
- Business logic validation (non-zero check) is separate from parsing

### 3. HTML Form Structure

```rust
async fn get_index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(
            r#"
                <title>GCD Calculator</title>
                <form action="/gcd" method="post">
                <input type="text" name="n"/>
                <input type="text" name="m"/>
                <button type="submit">Compute GCD</button>
                </form>
            "#,
        )
}
```

**Critical details:**
- `action="/gcd"` must match the route path
- `method="post"` must match the route's HTTP method guard
- `name="n"` and `name="m"` **must exactly match** struct field names
- No `type="number"` on inputs (client-side validation is optional; server always validates)

### 4. Automatic Error Responses

When form parsing fails, actix-web automatically returns:

```http
HTTP/1.1 400 Bad Request
Content-Type: text/plain

Failed to deserialize form: invalid digit found in string
```

**No handler code needed** for invalid input errors. Only business logic validation (like zero checks) is manual.

## Resulting Context

### Benefits

1. **Compile-Time Schema Validation**: Struct fields are checked by the compiler
2. **Automatic Parsing**: No manual `.parse()` calls or error handling boilerplate
3. **Type-Driven Security**: Invalid input never reaches business logic
4. **Centralized Validation**: Schema is co-located with field definitions
5. **Consistent Errors**: Parsing failures produce uniform 400 responses
6. **Performance**: Parsing is zero-allocation in many cases (lifetime-based deserialization)

### Liabilities

1. **Rigid Schema**: All fields must be present and correctly typed (use `Option<T>` for optional fields)
2. **Generic Error Messages**: Default errors may not be user-friendly
3. **No Partial Parsing**: All-or-nothing deserialization (can't process valid fields if one fails)
4. **Debugging Difficulty**: Serde errors can be cryptic for complex types

### Consequences

**Field Type Choices Matter**:
- `u64`: Requires non-negative integer, rejects decimals and negative numbers
- `i64`: Accepts negative integers
- `f64`: Accepts decimals
- `String`: Accepts any text
- `Option<T>`: Makes field optional (missing field = `None`)
- `Vec<T>`: Accepts multiple values with same name

**Example with Optional Field**:
```rust
#[derive(Deserialize)]
struct SearchQuery {
    q: String,           // Required
    limit: Option<u32>,  // Optional, defaults to None
}
```

**Validation Separation**:
- **Parsing validation**: Handled by Serde (type safety, presence)
- **Business logic validation**: Handled in handler (non-zero, range checks, cross-field validation)

## Related Patterns

- **Route Handlers**: Form extraction is one type of extractor; others include JSON, path params, and query strings
- **Error Handlers**: Customize 400 responses for better user experience
- **Request-Response Cycle**: Understanding when and how extractors are invoked
- **Application State**: Validate against shared state (e.g., check database constraints)

## Known Uses

From **actix-gcd** (`/home/user/rust-programming-examples/actix-gcd/src/main.rs`):

### Complete Form Handling Example

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct GcdParameters {
    n: u64,
    m: u64,
}

async fn post_gcd(form: web::Form<GcdParameters>) -> HttpResponse {
    // At this point, form.n and form.m are guaranteed valid u64 values
    if form.n == 0 || form.m == 0 {
        return HttpResponse::BadRequest()
            .content_type("text/html")
            .body("Computing the GCD with zero is boring.");
    }

    let response =
        format!("The greatest common divisor of the numbers {} and {} \
                 is <b>{}</b>\n",
                form.n, form.m, gcd(form.n, form.m));

    HttpResponse::Ok()
        .content_type("text/html")
        .body(response)
}
```

**What happens with different inputs:**

| Form Input | Parsing Result | Handler Behavior |
|------------|----------------|------------------|
| `n=42&m=56` | ✅ `GcdParameters { n: 42, m: 56 }` | Computes GCD, returns 14 |
| `n=0&m=56` | ✅ Parses, but `n=0` | Returns 400 with "boring" message |
| `n=abc&m=56` | ❌ Parse error | Handler never runs; 400 automatic |
| `n=42` (missing m) | ❌ Missing field | Handler never runs; 400 automatic |
| `n=-5&m=10` | ❌ Negative number | Handler never runs; 400 automatic |

### Common Extensions

**Optional Fields**:
```rust
#[derive(Deserialize)]
struct SearchParams {
    query: String,
    page: Option<u32>,  // Defaults to None if omitted
    limit: Option<u32>,
}

async fn search(params: web::Form<SearchParams>) -> HttpResponse {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);
    // ...
}
```

**Custom Validation**:
```rust
#[derive(Deserialize)]
struct CreateUser {
    username: String,
    age: u8,
}

async fn create_user(form: web::Form<CreateUser>) -> HttpResponse {
    // Parsing succeeded, now validate business rules
    if form.username.len() < 3 {
        return HttpResponse::BadRequest()
            .body("Username must be at least 3 characters");
    }
    if form.age < 18 {
        return HttpResponse::BadRequest()
            .body("Must be 18 or older");
    }
    // Proceed with creation...
}
```

**Multiple Values**:
```rust
#[derive(Deserialize)]
struct MultiSelect {
    ids: Vec<u64>,  // ?ids=1&ids=2&ids=3
}
```

## Implementation Notes

### Serde Configuration

From `Cargo.toml`:
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
```

The `derive` feature enables `#[derive(Deserialize)]` macros.

### Content-Type Header

Actix-web automatically handles:
- `application/x-www-form-urlencoded` (standard HTML forms)
- `multipart/form-data` (file uploads, use `MultipartForm` instead)

### Error Customization

To customize parsing error responses:

```rust
use actix_web::{error, http::StatusCode};
use serde::Deserialize;

#[derive(Deserialize)]
struct Params {
    value: u32,
}

async fn handler(form: web::Form<Params>) -> Result<HttpResponse, actix_web::Error> {
    // Use ? to propagate errors with custom messages
    Ok(HttpResponse::Ok().body(format!("Value: {}", form.value)))
}

// Or implement custom error handler
impl ResponseError for MyFormError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .content_type("text/html")
            .body(format!("<h1>Error</h1><p>{}</p>", self))
    }
}
```

### Performance Characteristics

- **Parsing Time**: ~1-10μs for simple forms (n fields)
- **Memory**: Zero-copy deserialization for `&str` fields (when using lifetimes)
- **Validation**: Happens once at extraction time, not repeated

### Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn test_valid_form() {
        let app = test::init_service(
            App::new().route("/gcd", web::post().to(post_gcd))
        ).await;

        let req = test::TestRequest::post()
            .uri("/gcd")
            .set_form(&GcdParameters { n: 42, m: 56 })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_invalid_form() {
        let app = test::init_service(
            App::new().route("/gcd", web::post().to(post_gcd))
        ).await;

        // Send malformed data
        let req = test::TestRequest::post()
            .uri("/gcd")
            .set_payload("n=abc&m=56")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
```

## Security Considerations

1. **Input Validation is Not Optional**: Always validate business rules even after parsing
2. **Type System Prevents Injection**: u64 fields cannot contain SQL or script injection
3. **String Fields Need Validation**: Never trust string content; sanitize for HTML/SQL/etc.
4. **Size Limits**: Actix limits request body size (default 256KB); configure with `.app_data(PayloadConfig::new(...))`
5. **Rate Limiting**: Parsing is cheap, but still apply rate limits to prevent DoS

### Example: Preventing HTML Injection

```rust
async fn post_comment(form: web::Form<Comment>) -> HttpResponse {
    // BAD: Directly embedding user input in HTML
    // let html = format!("<p>{}</p>", form.text);

    // GOOD: Escape HTML entities
    let escaped = html_escape::encode_text(&form.text);
    let html = format!("<p>{}</p>", escaped);

    HttpResponse::Ok().body(html)
}
```

## References

- **Serde Documentation**: https://serde.rs/
- **actix-web Extractors**: https://actix.rs/docs/extractors/
- **HTML Forms Specification**: HTML 5.2 Section 4.10
- **URL Encoding**: RFC 3986
