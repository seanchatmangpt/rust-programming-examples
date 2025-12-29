# HTML Templating

## Context

Your web service needs to generate HTML responses dynamically, incorporating data from requests, databases, or computations. You want responses that are type-safe, free from injection vulnerabilities, and maintainable as your application grows. The HTML may include user input that must be properly escaped.

## Problem

**How do you generate HTML responses that are safe from injection attacks, maintainable, and integrate naturally with Rust's type system?**

Generating HTML in web services involves several challenges:
- Concatenating strings is tedious and error-prone
- User input must be HTML-escaped to prevent XSS attacks
- Templates should be validated (no missing closing tags)
- Logic and presentation should be separated
- Performance matters (HTML generation happens on every request)

Traditional approaches have weaknesses:
- **String concatenation**: Error-prone, no structure validation, easy to forget escaping
- **External template engines** (Handlebars, Jinja): Runtime parsing, unclear errors
- **JSX/React**: Requires JavaScript ecosystem, build tools

## Forces

- **Safety vs Simplicity**: Automatic escaping prevents XSS; manual control allows flexibility
- **Compile-Time vs Runtime**: Static checks catch errors early; dynamic templates are flexible
- **Embedded vs External**: Code-embedded templates are simple; file-based templates separate concerns
- **Performance**: Template compilation overhead vs rendering speed
- **Maintainability**: Small inline templates are convenient; large templates need structure

## Solution

**Use raw string literals for small templates, with explicit HTML escaping for user input; migrate to template engines (Tera, Askama) as complexity grows.**

The pattern has three levels of sophistication:

### Level 1: Static HTML with Raw Strings

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

**Why this works:**
- `r#"..."#` is a **raw string literal**: no escape sequences, what you see is what you get
- No HTML escaping needed (no user input)
- Content-type `text/html` tells browser to render as HTML
- Indentation is preserved (will appear in HTML source)

**When to use:**
- Static HTML with no dynamic content
- Simple forms, landing pages, error pages
- Prototyping before adding a template engine

### Level 2: Dynamic HTML with format!

```rust
async fn post_gcd(form: web::Form<GcdParameters>) -> HttpResponse {
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

**Why this works:**
- `format!` is Rust's string interpolation macro
- `{}` placeholders are replaced with values
- `form.n`, `form.m`, and `gcd()` result are all **numeric types** (u64)
- **Numbers are safe to embed** (cannot contain HTML tags or scripts)

**Critical safety note:**
This is safe because `u64` values can only be numbers. If embedding strings:

```rust
// UNSAFE: User input could contain <script> tags
let unsafe_html = format!("<p>Hello, {}</p>", user_name);

// SAFE: Escape HTML entities
use html_escape::encode_text;
let safe_html = format!("<p>Hello, {}</p>", encode_text(&user_name));
```

**When to use:**
- Simple dynamic content (numbers, enums, trusted data)
- Error messages with context
- Redirects with success messages
- Debugging/development responses

### Level 3: Template Engines (Recommended for Production)

For larger applications, use a template engine:

**Tera Example** (Jinja2-like syntax):
```rust
use tera::{Tera, Context};

async fn get_index(tmpl: web::Data<Tera>) -> HttpResponse {
    let mut context = Context::new();
    context.insert("title", "GCD Calculator");

    let html = tmpl.render("index.html", &context).unwrap();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(html)
}
```

`templates/index.html`:
```html
<!DOCTYPE html>
<html>
<head><title>{{ title }}</title></head>
<body>
    <form action="/gcd" method="post">
        <input type="text" name="n"/>
        <input type="text" name="m"/>
        <button type="submit">Compute GCD</button>
    </form>
</body>
</html>
```

**Askama Example** (compile-time templates):
```rust
use askama::Template;

#[derive(Template)]
#[template(path = "result.html")]
struct ResultTemplate {
    n: u64,
    m: u64,
    gcd: u64,
}

async fn post_gcd(form: web::Form<GcdParameters>) -> HttpResponse {
    let result = ResultTemplate {
        n: form.n,
        m: form.m,
        gcd: gcd(form.n, form.m),
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(result.render().unwrap())
}
```

`templates/result.html`:
```html
<p>The GCD of {{ n }} and {{ m }} is <b>{{ gcd }}</b></p>
```

**When to use:**
- Multiple pages with shared layouts
- Complex conditional logic in templates
- Iteration over collections
- Automatic HTML escaping
- Team with separate frontend/backend developers

## Resulting Context

### Benefits of Raw String Approach (actix-gcd style)

1. **Zero Dependencies**: No template engine needed
2. **Compile-Time Safety**: Syntax errors caught by Rust compiler (unclosed strings)
3. **Performance**: No template parsing at runtime (~100ns to construct response)
4. **Simplicity**: No new syntax to learn
5. **Immediate Visibility**: HTML structure visible in handler code

### Liabilities of Raw String Approach

1. **No Auto-Escaping**: Must manually escape user input (easy to forget)
2. **No Validation**: Malformed HTML compiles fine
3. **Code Clutter**: Large HTML blocks make handlers hard to read
4. **No Reusability**: Can't share layouts or components
5. **Editor Support**: Most editors don't syntax-highlight HTML in strings

### Benefits of Template Engines

1. **Automatic Escaping**: User input is escaped by default (prevents XSS)
2. **Syntax Validation**: Template errors caught at parse time (Tera) or compile time (Askama)
3. **Reusability**: Layouts, includes, and macros for DRY code
4. **Separation of Concerns**: Designers can edit HTML without touching Rust
5. **Editor Support**: Full HTML syntax highlighting and validation

### Liabilities of Template Engines

1. **Runtime Overhead**: Tera parses templates at startup (~1-10ms per template)
2. **Dependency Weight**: Adds crate dependencies (tera: ~500KB, askama: ~200KB)
3. **Learning Curve**: New template syntax to learn
4. **Error Messages**: Runtime template errors can be cryptic
5. **Indirection**: Handler code doesn't show HTML structure

## Related Patterns

- **Route Handlers**: Handlers return `HttpResponse` with templated bodies
- **Form Handling**: Forms are defined in HTML templates, parsed in handlers
- **Application State**: Template engines are stored in `web::Data<Tera>` for sharing
- **Response Builders**: Templates generate bodies for response builders

## Known Uses

From **actix-gcd** (`/home/user/rust-programming-examples/actix-gcd/src/main.rs`):

### Example 1: Static HTML Form

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

**Characteristics:**
- No `<!DOCTYPE html>` or `<html>` tags (minimal HTML)
- Form `action` points to `/gcd` endpoint
- Form `method="post"` matches route's POST guard
- Input `name` attributes match struct field names
- Self-contained (no layout, no navigation)

**Production improvements:**
```rust
async fn get_index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>GCD Calculator</title>
    <style>
        body { font-family: sans-serif; max-width: 600px; margin: 2em auto; }
        input { margin: 0.5em; padding: 0.5em; }
    </style>
</head>
<body>
    <h1>GCD Calculator</h1>
    <form action="/gcd" method="post">
        <label>First number: <input type="text" name="n" required/></label>
        <label>Second number: <input type="text" name="m" required/></label>
        <button type="submit">Compute GCD</button>
    </form>
</body>
</html>"#,
        )
}
```

### Example 2: Dynamic HTML with Safe Interpolation

```rust
async fn post_gcd(form: web::Form<GcdParameters>) -> HttpResponse {
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

**Safety analysis:**
- `form.n` and `form.m` are `u64` (cannot contain HTML)
- `gcd(form.n, form.m)` returns `u64` (cannot contain HTML)
- No user-controlled strings are embedded
- **This is safe** because all interpolated values are numbers

**If embedding strings:**
```rust
// Example with user input (UNSAFE without escaping)
#[derive(Deserialize)]
struct Comment {
    author: String,  // User input!
    text: String,    // User input!
}

async fn unsafe_comment(form: web::Form<Comment>) -> HttpResponse {
    // DANGEROUS: XSS vulnerability
    let html = format!("<p><b>{}</b>: {}</p>", form.author, form.text);
    // If author = "<script>alert('XSS')</script>", script executes!

    HttpResponse::Ok().body(html)
}

// SAFE version with escaping
async fn safe_comment(form: web::Form<Comment>) -> HttpResponse {
    use html_escape::encode_text;
    let html = format!(
        "<p><b>{}</b>: {}</p>",
        encode_text(&form.author),
        encode_text(&form.text)
    );
    HttpResponse::Ok().body(html)
}
```

### Example 3: Error Responses

```rust
if form.n == 0 || form.m == 0 {
    return HttpResponse::BadRequest()
        .content_type("text/html")
        .body("Computing the GCD with zero is boring.");
}
```

**Error response pattern:**
- 400 Bad Request status (client error)
- HTML content type
- Human-readable message
- No stack traces or internal details

**Production improvement:**
```rust
if form.n == 0 || form.m == 0 {
    return HttpResponse::BadRequest()
        .content_type("text/html")
        .body(r#"
<!DOCTYPE html>
<html>
<head><title>Error</title></head>
<body>
    <h1>Invalid Input</h1>
    <p>Computing the GCD with zero is boring.</p>
    <a href="/">Try again</a>
</body>
</html>
        "#);
}
```

## Implementation Notes

### HTML Escaping Library

Add to `Cargo.toml`:
```toml
[dependencies]
html-escape = "0.2"
```

Usage:
```rust
use html_escape::{encode_text, encode_quoted_attribute};

let safe_text = encode_text("<script>alert('xss')</script>");
// Result: "&lt;script&gt;alert('xss')&lt;/script&gt;"

let safe_attr = encode_quoted_attribute("onclick=\"alert('xss')\"");
// Result: "onclick=&quot;alert('xss')&quot;"
```

### Template Engine Setup (Tera)

`Cargo.toml`:
```toml
[dependencies]
tera = "1.17"
```

`main.rs`:
```rust
use tera::Tera;

#[actix_web::main]
async fn main() {
    let tera = Tera::new("templates/**/*.html").unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:3000")
    .unwrap()
    .run()
    .await
    .unwrap();
}

async fn index(tmpl: web::Data<Tera>) -> HttpResponse {
    let ctx = tera::Context::new();
    let html = tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(html)
}
```

### Template Engine Setup (Askama)

`Cargo.toml`:
```toml
[dependencies]
askama = "0.12"
```

`templates/index.html`:
```html
<!DOCTYPE html>
<html>
<head><title>{{ title }}</title></head>
<body><h1>{{ heading }}</h1></body>
</html>
```

`main.rs`:
```rust
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    title: String,
    heading: String,
}

async fn index() -> impl Responder {
    IndexTemplate {
        title: "Home".to_string(),
        heading: "Welcome!".to_string(),
    }
}
```

### Performance Characteristics

**Raw Strings**:
- Template "compilation": 0ns (compile-time string)
- Rendering: ~100-500ns (string copy)
- Memory: Exact size of output string

**format! Macro**:
- Compilation: 0ns (compile-time macro)
- Rendering: ~1-10μs (depends on interpolation count)
- Memory: Allocates output string

**Tera (runtime)**:
- Startup: ~1-10ms to parse all templates
- Rendering: ~10-100μs (depends on complexity)
- Memory: Parsed AST cached, output string allocated

**Askama (compile-time)**:
- Compile time: +1-5s to compile time
- Rendering: ~1-10μs (compiled Rust code)
- Memory: Output string allocated

### Testing Strategies

**Unit Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcd_response_format() {
        let form = GcdParameters { n: 42, m: 56 };
        let response = format!(
            "The greatest common divisor of the numbers {} and {} is <b>{}</b>\n",
            form.n, form.m, gcd(form.n, form.m)
        );
        assert!(response.contains("42"));
        assert!(response.contains("56"));
        assert!(response.contains("<b>14</b>"));
    }
}
```

**Integration Tests**:
```rust
#[actix_web::test]
async fn test_html_response() {
    let app = test::init_service(
        App::new().route("/", web::get().to(get_index))
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let html = std::str::from_utf8(&body).unwrap();
    assert!(html.contains("<form"));
    assert!(html.contains("action=\"/gcd\""));
}
```

**HTML Validation**:
```bash
# Use external tool to validate generated HTML
curl http://localhost:3000/ | tidy -errors
```

## Security Considerations

### XSS Prevention Checklist

- [ ] **Never** embed user input directly in HTML
- [ ] **Always** escape user input with `html_escape::encode_text()`
- [ ] **Use** template engines with auto-escaping (Tera, Askama)
- [ ] **Validate** user input before rendering (reject malicious patterns)
- [ ] **Set** Content-Security-Policy header to mitigate XSS impact

### Content Security Policy

```rust
HttpResponse::Ok()
    .content_type("text/html")
    .insert_header((
        "Content-Security-Policy",
        "default-src 'self'; script-src 'none';"
    ))
    .body(html)
```

### MIME Type Security

Always set correct content type:
```rust
// CORRECT: Browser treats as HTML
.content_type("text/html; charset=utf-8")

// DANGEROUS: Browser guesses type (can lead to XSS)
.content_type("text/plain")  // With HTML content
```

## Migration Path: Raw Strings → Template Engine

**Stage 1: Extract string constants**
```rust
const INDEX_HTML: &str = r#"<form>...</form>"#;

async fn get_index() -> HttpResponse {
    HttpResponse::Ok().body(INDEX_HTML)
}
```

**Stage 2: Move to separate files**
```rust
async fn get_index() -> HttpResponse {
    let html = include_str!("../templates/index.html");
    HttpResponse::Ok().body(html)
}
```

**Stage 3: Add template engine**
```rust
async fn get_index(tmpl: web::Data<Tera>) -> HttpResponse {
    let html = tmpl.render("index.html", &Context::new()).unwrap();
    HttpResponse::Ok().body(html)
}
```

## References

- **HTML Escaping**: https://docs.rs/html-escape/
- **Tera Templates**: https://tera.netlify.app/
- **Askama**: https://djc.github.io/askama/
- **XSS Prevention**: OWASP XSS Prevention Cheat Sheet
- **Content Security Policy**: https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP
