# Integration Across Systems

Real-world systems rarely consist of a single architectural pattern. Production applications combine web services with databases, text processing with network I/O, and async operations with data structures. This final section synthesizes the patterns from previous case studies, revealing how Rust's type system enables clean integration across architectural boundaries.

## Multiple Architectural Layers in Practice

Consider building a real-world service: a web-based grep tool that searches uploaded files. This combines:

- **Web layer**: HTTP request/response (actix-web pattern)
- **Processing layer**: Text search (grep pattern)
- **Storage layer**: File I/O (stream processing pattern)
- **Concurrency layer**: Async execution (many-requests pattern)

The architecture looks like this:

```
┌─────────────────────────────────┐
│   HTTP Handler (async)          │  ← actix-web
├─────────────────────────────────┤
│   Business Logic (pure)         │  ← grep algorithm
├─────────────────────────────────┤
│   Storage Access (async I/O)    │  ← stream processing
├─────────────────────────────────┤
│   Data Structures (owned)       │  ← Queue, BinaryTree
└─────────────────────────────────┘
```

Each layer has distinct responsibilities and architectural constraints.

## Clear Separation of Concerns

The actix-gcd example demonstrates this separation:

```rust
// LAYER 1: Pure business logic (no I/O, no HTTP)
fn gcd(mut n: u64, mut m: u64) -> u64 {
    assert!(n != 0 && m != 0);
    while m != 0 {
        if m < n {
            let t = m; m = n; n = t;
        }
        m = m % n;
    }
    n
}

// LAYER 2: HTTP handling (uses business logic)
async fn post_gcd(form: web::Form<GcdParameters>) -> HttpResponse {
    if form.n == 0 || form.m == 0 {
        return HttpResponse::BadRequest()
            .content_type("text/html")
            .body("Computing the GCD with zero is boring.");
    }

    let response = format!(
        "The greatest common divisor of the numbers {} and {} is <b>{}</b>\n",
        form.n, form.m, gcd(form.n, form.m)  // Calls pure logic
    );

    HttpResponse::Ok()
        .content_type("text/html")
        .body(response)
}
```

**Separation Benefits**:

1. **Testability**: The `gcd` function can be tested without web framework:
   ```rust
   #[test]
   fn test_gcd() {
       assert_eq!(gcd(42, 56), 14);
   }
   ```

2. **Reusability**: The same `gcd` function works in CLI tools, GUIs, or microservices.

3. **Clarity**: Each layer has a single responsibility. HTTP concerns don't pollute algorithm implementation.

This is **dependency inversion**—the core logic depends on nothing, and the HTTP layer depends on the core. Dependencies point inward.

## Error Propagation Through Layers

Errors must cross layer boundaries cleanly. Consider a file search service:

```rust
// Domain error type
#[derive(Debug)]
enum SearchError {
    FileNotFound(PathBuf),
    InvalidPattern(String),
    IoError(io::Error),
}

impl From<io::Error> for SearchError {
    fn from(err: io::Error) -> Self {
        SearchError::IoError(err)
    }
}

// LAYER 1: Storage (returns domain errors)
async fn read_file(path: &Path) -> Result<String, SearchError> {
    if !path.exists() {
        return Err(SearchError::FileNotFound(path.to_path_buf()));
    }

    let content = tokio::fs::read_to_string(path).await?;
    Ok(content)
}

// LAYER 2: Business logic (returns domain errors)
async fn search_file(path: &Path, pattern: &str) -> Result<Vec<String>, SearchError> {
    let content = read_file(path).await?;

    let regex = Regex::new(pattern)
        .map_err(|e| SearchError::InvalidPattern(e.to_string()))?;

    Ok(content
        .lines()
        .filter(|line| regex.is_match(line))
        .map(String::from)
        .collect())
}

// LAYER 3: HTTP handler (converts domain errors to HTTP errors)
async fn search_handler(
    path: web::Path<String>,
    query: web::Query<SearchParams>,
) -> Result<HttpResponse, actix_web::Error> {
    let results = search_file(Path::new(&path.0), &query.pattern)
        .await
        .map_err(|e| match e {
            SearchError::FileNotFound(_) =>
                actix_web::error::ErrorNotFound("File not found"),
            SearchError::InvalidPattern(_) =>
                actix_web::error::ErrorBadRequest("Invalid regex pattern"),
            SearchError::IoError(_) =>
                actix_web::error::ErrorInternalServerError("I/O error"),
        })?;

    Ok(HttpResponse::Ok().json(results))
}
```

**Error Propagation Patterns**:

**Domain-Specific Error Types**: `SearchError` captures all failure modes of the search domain. It's not an HTTP error (like 404) or an I/O error—it's domain-specific.

**From Trait for Conversion**: Implementing `From<io::Error>` allows the `?` operator to automatically convert I/O errors to domain errors. This is **error wrapping**.

**Layer-Specific Translation**: The HTTP handler converts domain errors to HTTP errors. This keeps HTTP concerns out of the business logic layer.

**Error Context Enrichment**: Each layer adds context:
```
Layer 1: io::Error { kind: NotFound }
Layer 2: SearchError::FileNotFound("/path/to/file")
Layer 3: 404 Not Found with body "File not found"
```

This is **progressive error enrichment**—errors gain context as they bubble up.

## Ownership Boundaries Across Layers

Ownership transfers clarify responsibility:

```rust
// Database-backed queue (combines storage and data structure)
pub struct PersistentQueue<T> {
    items: VecDeque<T>,
    storage: File,
}

impl<T: Serialize + DeserializeOwned> PersistentQueue<T> {
    // Takes ownership: caller gives up the file
    pub fn new(storage: File) -> io::Result<Self> {
        let items = VecDeque::new();
        Ok(PersistentQueue { items, storage })
    }

    // Returns owned value: caller gets ownership
    pub fn pop(&mut self) -> Option<T> {
        let item = self.items.pop_front()?;
        self.sync().ok()?;
        Some(item)
    }

    // Borrows: caller retains ownership
    pub fn peek(&self) -> Option<&T> {
        self.items.front()
    }

    // Takes ownership: item is moved into queue
    pub fn push(&mut self, item: T) -> io::Result<()> {
        self.items.push_back(item);
        self.sync()
    }

    fn sync(&mut self) -> io::Result<()> {
        self.storage.seek(SeekFrom::Start(0))?;
        self.storage.set_len(0)?;
        serde_json::to_writer(&self.storage, &self.items)?;
        self.storage.flush()
    }
}
```

**Ownership Design Decisions**:

**Constructor Takes Ownership**: `new(storage: File)` takes ownership of the file. This prevents external code from closing the file while the queue uses it. The file's lifetime is tied to the queue's lifetime.

**Pop Returns Owned Value**: `pop` returns `T`, not `&T`. The caller gets full ownership and can modify or move the value. This is appropriate for a queue—once dequeued, items leave the queue permanently.

**Peek Returns Reference**: `peek` returns `&T` because the item remains in the queue. The reference's lifetime is tied to the borrow of `&self`, preventing modification during observation.

**Push Takes Ownership**: `push(item: T)` takes ownership. The item is moved into the queue, preventing the caller from accidentally using it after enqueueing.

These ownership patterns create a **clear ownership contract**: the queue owns its items, and ownership transfers happen at explicit boundaries (push/pop).

## Trait-Based Integration Points

Traits enable different architectural layers to interact without tight coupling:

```rust
// Generic search trait
trait Searchable {
    type Item;

    fn search(&self, query: &str) -> Vec<Self::Item>;
}

// Implementation for in-memory data (uses BinaryTree)
impl Searchable for BinaryTree<String> {
    type Item = String;

    fn search(&self, query: &str) -> Vec<String> {
        self.iter()
            .filter(|s| s.contains(query))
            .cloned()
            .collect()
    }
}

// Implementation for file-based data (uses grep pattern)
struct FileIndex {
    path: PathBuf,
}

impl Searchable for FileIndex {
    type Item = String;

    fn search(&self, query: &str) -> Vec<String> {
        let file = File::open(&self.path).unwrap();
        let reader = BufReader::new(file);

        reader
            .lines()
            .filter_map(Result::ok)
            .filter(|line| line.contains(query))
            .collect()
    }
}

// Generic search handler (works with any Searchable)
async fn search_handler<S: Searchable>(
    searchable: web::Data<S>,
    query: web::Query<SearchParams>,
) -> HttpResponse {
    let results = searchable.search(&query.pattern);
    HttpResponse::Ok().json(results)
}
```

**Trait-Based Architecture Benefits**:

**Polymorphism Without Virtual Dispatch**: The `search_handler` is generic over `S: Searchable`. The compiler generates specialized versions for each concrete type (monomorphization). This is **zero-cost abstraction**.

**Swappable Implementations**: You can swap `BinaryTree` for `FileIndex` without changing handler code. This enables testing with in-memory implementations and production with file-based ones.

**Type Safety**: The compiler ensures all `Searchable` implementations have the `search` method with the correct signature. You can't accidentally forget to implement it.

## Integrating Async Across Layers

Async operations compose naturally in layered architectures:

```rust
// LAYER 1: Data access (async I/O)
async fn fetch_user(id: u64) -> Result<User, DbError> {
    let query = format!("SELECT * FROM users WHERE id = {}", id);
    DATABASE.query(&query).await
}

// LAYER 2: Business logic (async operations)
async fn calculate_user_score(id: u64) -> Result<f64, AppError> {
    let user = fetch_user(id).await?;
    let activities = fetch_activities(user.id).await?;

    Ok(activities.iter().map(|a| a.points).sum())
}

// LAYER 3: HTTP handler (async composition)
async fn user_score_handler(
    path: web::Path<u64>,
) -> Result<HttpResponse, actix_web::Error> {
    let score = calculate_user_score(path.into_inner())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(json!({ "score": score })))
}
```

**Async Composition**:

Each layer is async, using `.await` to call the layer below. The `?` operator propagates errors through async boundaries just like sync code. This is **transparent async**—the architecture doesn't fundamentally change whether operations are sync or async.

**Async Boundaries**: Importantly, only I/O operations are async. Pure computation (like summing points) remains synchronous:

```rust
async fn calculate_user_score(id: u64) -> Result<f64, AppError> {
    let user = fetch_user(id).await?;         // Async I/O
    let activities = fetch_activities(user.id).await?;  // Async I/O

    // Pure computation (sync)
    let score = activities.iter().map(|a| a.points).sum();
    Ok(score)
}
```

This keeps the async footprint minimal. Only operations that truly need async (I/O) use it.

## Architectural Pattern: The Onion Architecture

All case studies follow the **onion architecture**:

```
        ┌─────────────────────┐
        │   HTTP/CLI Layer    │  ← Framework-specific code
        ├─────────────────────┤
        │  Application Layer  │  ← Use cases, business logic
        ├─────────────────────┤
        │   Domain Layer      │  ← Pure types and algorithms
        └─────────────────────┘
```

**Domain Layer** (innermost):
- Pure functions and data structures
- No dependencies on frameworks, I/O, or infrastructure
- Example: `gcd` function, `BinaryTree` type, `Queue` implementation

**Application Layer** (middle):
- Orchestrates domain objects
- Handles I/O, but delegates logic to domain
- Example: `grep` function (calls `contains` on strings), `apply_sunlight` (modifies ferns)

**Interface Layer** (outermost):
- Handles external interactions (HTTP, CLI, etc.)
- Converts between domain types and external formats
- Example: `post_gcd` handler, `grep_main` CLI parser

Dependencies flow **inward**: outer layers depend on inner layers, never the reverse. This enables testing inner layers without outer layers' complexity.

## Synthesizing All Patterns: A Complete Service

Let's design a complete service using all patterns:

**Service**: A text search API with persistent storage

```rust
// ===== DOMAIN LAYER (pure logic) =====

#[derive(Serialize, Deserialize, Clone)]
struct Document {
    id: u64,
    content: String,
}

// Pure search algorithm (from grep pattern)
fn search_documents(docs: &[Document], query: &str) -> Vec<Document> {
    docs.iter()
        .filter(|doc| doc.content.contains(query))
        .cloned()
        .collect()
}

// ===== APPLICATION LAYER (orchestration) =====

// Uses queue pattern for persistence
struct DocumentStore {
    documents: Vec<Document>,
    storage: File,
}

impl DocumentStore {
    async fn load(path: &Path) -> Result<Self, io::Error> {
        let mut file = File::open(path).await?;
        let mut content = String::new();
        file.read_to_string(&mut content).await?;

        let documents: Vec<Document> = serde_json::from_str(&content)
            .unwrap_or_default();

        Ok(DocumentStore {
            documents,
            storage: file,
        })
    }

    async fn add_document(&mut self, doc: Document) -> Result<(), io::Error> {
        self.documents.push(doc);
        self.persist().await
    }

    fn search(&self, query: &str) -> Vec<Document> {
        search_documents(&self.documents, query)
    }

    async fn persist(&mut self) -> Result<(), io::Error> {
        let content = serde_json::to_string(&self.documents)?;
        self.storage.seek(SeekFrom::Start(0)).await?;
        self.storage.set_len(0).await?;
        self.storage.write_all(content.as_bytes()).await?;
        Ok(())
    }
}

// ===== INTERFACE LAYER (HTTP handlers) =====

async fn search_handler(
    store: web::Data<Arc<RwLock<DocumentStore>>>,
    query: web::Query<SearchQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    let store = store.read().await;
    let results = store.search(&query.q);

    Ok(HttpResponse::Ok().json(results))
}

async fn add_handler(
    store: web::Data<Arc<RwLock<DocumentStore>>>,
    doc: web::Json<Document>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut store = store.write().await;
    store.add_document(doc.into_inner()).await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Created().finish())
}

// ===== MAIN (wiring everything together) =====

#[actix_web::main]
async fn main() -> io::Result<()> {
    let store = DocumentStore::load(Path::new("documents.json"))
        .await?;
    let store = web::Data::new(Arc::new(RwLock::new(store)));

    HttpServer::new(move || {
        App::new()
            .app_data(store.clone())
            .route("/search", web::get().to(search_handler))
            .route("/documents", web::post().to(add_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

**Architecture Analysis**:

**Layers Are Clear**:
- Domain: `search_documents` (pure function)
- Application: `DocumentStore` (manages persistence)
- Interface: `search_handler`, `add_handler` (HTTP specifics)

**Patterns Combined**:
- **Web service** pattern (actix-web)
- **Text processing** pattern (search algorithm)
- **Storage** pattern (async file I/O)
- **Data structure** pattern (Vec used like Queue)
- **Concurrency** pattern (RwLock for thread-safe access)

**Ownership Boundaries**:
- `DocumentStore` owns documents
- Handlers borrow via `Arc<RwLock<>>`
- Each layer has clear ownership semantics

**Error Handling**:
- I/O errors propagate with `?`
- Conversions at layer boundaries (`.map_err`)
- Domain layer is infallible (search can't fail)

## Lessons from Integration

Synthesizing these patterns teaches several lessons:

1. **Start with Pure Core**: Build domain logic first, without I/O or frameworks. This makes it testable and portable.

2. **Use Traits for Abstraction**: Traits enable swapping implementations without changing dependent code.

3. **Ownership Clarifies Responsibility**: Explicit ownership transfer makes responsibilities clear. Who owns this data? The types tell you.

4. **Errors Should Carry Context**: As errors propagate up layers, add context. Don't lose information.

5. **Async Composes Naturally**: Async functions compose just like sync functions. Don't fear mixing them.

6. **Architecture Emerges from Constraints**: The onion architecture emerges naturally when you separate pure logic from I/O, domain from infrastructure.

## Cross-References: Complete Pattern Map

This chapter synthesizes patterns from the entire book:

- **Chapter 1 (Ownership)**: Ownership boundaries enable clear layer separation
- **Chapter 2 (Types)**: Rich type systems model domains accurately
- **Chapter 3 (Modules)**: Hierarchical modules organize large systems
- **Chapter 4 (Traits)**: Traits provide abstraction between layers
- **Chapter 5 (Error Handling)**: Errors propagate cleanly through layers
- **Chapter 6 (Collections)**: Data structures underpin storage layers
- **Chapter 7 (Concurrency)**: Thread safety enables concurrent access
- **Chapter 8 (Async)**: Async I/O integrates seamlessly across layers

Each pattern, studied individually in earlier chapters, combines here to create complete, production-ready architectures.

## Conclusion

Real-world Rust architectures leverage multiple patterns simultaneously. Web services integrate with databases, text processors handle network data, and async operations span architectural layers. The case studies in this chapter demonstrate how Rust's type system, ownership model, and trait system enable clean integration across these boundaries.

The key insight: **architecture in Rust emerges from types**. Ownership clarifies responsibilities, lifetimes prevent resource leaks, traits enable abstraction, and the compiler enforces correctness. You don't fight the architecture—you express it through types, and the compiler ensures it holds.

These patterns scale from small tools (grep, echo server) to large systems (web frameworks, databases, distributed systems). The architectural principles remain constant: clear ownership, strong typing, and fearless composition.
