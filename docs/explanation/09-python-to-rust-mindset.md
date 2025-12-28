# Shifting from Python Thinking to Rust Thinking

Moving from Python to Rust isn't just learning new syntax - it's a fundamental shift in how you think about programs. This guide explores the mental models you need to let go of and those you need to embrace.

## Let Go Of: Garbage Collection

### Python: Automatic Memory Management

In Python, you never think about memory:

```python
def process():
    data = load_huge_file()  # Allocates
    result = transform(data)  # Allocates more
    return result
    # data is freed... eventually
```

The garbage collector cleans up. When? You don't know. You don't care. This is freedom - and also a limitation.

### Rust: Explicit Ownership

In Rust, every value has an owner:

```rust
fn process() -> String {
    let data = load_huge_file();  // data is owned here
    let result = transform(data);  // ownership moved to transform
    result  // ownership moved to caller
}  // Nothing to clean up - everything was moved
```

When `data` is moved into `transform`, it's no longer accessible in `process`. **This is not a bug - it's the design.**

The memory is freed **exactly** when the owner goes out of scope. No garbage collection pauses, no uncertainty.

### The Mental Shift

Python trains you to think: "Create objects freely, the GC will handle it."

Rust trains you to think: "Who owns this? How long does it live? Can I share it?"

**Initial reaction**: "This is so restrictive!"

**After a month**: "Wait, I know exactly when allocations happen and when memory is freed. I can reason about performance."

**After six months**: "How did I ever debug memory leaks in Python?"

## Let Go Of: Duck Typing

### Python: If It Quacks Like a Duck...

```python
def draw(shape):
    shape.draw()  # Anything with a draw() method works
```

This is flexible and expressive. It's also terrifying:

```python
draw(file_handle)  # Oops, files have draw() too (but it does something different)
```

You find this bug at runtime, in production, when a user hits that code path.

### Rust: Traits Are Contracts

```rust
trait Drawable {
    fn draw(&self);
}

fn draw(shape: &impl Drawable) {
    shape.draw();
}
```

Only types that explicitly implement `Drawable` can be passed. The compiler checks this at compile time.

```rust
draw(&file_handle);  // ERROR: File doesn't implement Drawable
```

You find this bug while writing the code, not when users run it.

### The Mental Shift

Python: "I'll pass anything and hope it has the right methods."

Rust: "I'll explicitly state what I need, and the compiler ensures I get it."

**Initial reaction**: "I have to implement traits for everything?!"

**After a month**: "Oh, the compiler is telling me exactly what's wrong and where."

**After six months**: "How did I ever trust Python's runtime to catch type errors?"

## Let Go Of: Runtime Flexibility

### Python: Change Anything, Anytime

```python
class Dog:
    def __init__(self):
        self.name = "Fido"

dog = Dog()
dog.age = 3  # Just add a new attribute!
dog.speak = lambda: "Woof"  # Add a method!
Dog.new_method = lambda self: "Class method!"  # Modify the class!
```

This is powerful for metaprogramming, testing, and monkey-patching. It's also how bugs hide in dark corners of your codebase.

### Rust: Fixed at Compile Time

```rust
struct Dog {
    name: String
}

let dog = Dog { name: "Fido".to_string() };
dog.age = 3;  // ERROR: no field `age` on struct `Dog`
```

You cannot add fields at runtime. You cannot modify methods. The structure is fixed.

**To add a field**, you modify the struct definition and recompile. Every place that constructs a `Dog` must be updated.

### The Mental Shift

Python: "I can change anything dynamically."

Rust: "The structure of my program is visible in the code and checked by the compiler."

**Initial reaction**: "This is so inflexible!"

**After a month**: "Wait, when I change a struct, the compiler finds every place I need to update. No silent bugs."

**After six months**: "How did I ever refactor Python without the compiler's help?"

## Embrace: Compile-Time Checks

### The Compiler Is Your Pair Programmer

In Python:
```python
def divide(a, b):
    return a / b

result = divide(10, 0)  # ZeroDivisionError at runtime
```

In Rust, the compiler can't prevent division by zero, but it catches many errors Python can't:

```rust
fn divide(a: &mut i32, b: i32) -> i32 {
    *a / b
}

let mut x = 10;
let result = divide(&mut x, x);  // ERROR: cannot borrow `x` as mutable and immutable
```

The compiler caught a potential data race at compile time.

### Types Are Guardrails

```rust
fn send_email(to: Email, subject: String, body: String)
```

vs.

```python
def send_email(to, subject, body):
    # Hope everything is a string!
```

With Rust, if you try:
```rust
send_email(subject, to, body);  // ERROR: wrong types
```

The compiler catches it. In Python, you'd send an email with swapped fields and not notice until someone complains.

### The Mental Shift

Python: "Write tests to catch bugs."

Rust: "Write types to prevent bugs, then write tests for logic."

**You're not testing that you passed the right types. The compiler does that for free.**

## Embrace: Explicit Types

### Type Inference, But Explicit Where It Matters

Rust has type inference:

```rust
let numbers = vec![1, 2, 3];  // Compiler infers Vec<i32>
```

But function signatures must be explicit:

```rust
fn process(data: &[u8]) -> Result<String, Error>
```

### Why Explicit?

1. **Documentation**: The signature tells you exactly what the function does
2. **Error messages**: When you call it wrong, the compiler can give precise errors
3. **Refactoring**: Change the signature, and the compiler finds all callers

### Python's Type Hints vs. Rust's Types

Python (optional):
```python
def process(data: bytes) -> str:
    return data.decode()

process("hello")  # Runs fine! Type hints aren't enforced
```

Rust (enforced):
```rust
fn process(data: &[u8]) -> String {
    String::from_utf8_lossy(data).to_string()
}

process("hello");  // ERROR: expected &[u8], found &str
```

### The Mental Shift

Python: "Types are documentation that might be wrong."

Rust: "Types are guarantees enforced by the compiler."

## Embrace: Ownership

### The Hardest Mental Shift

This is the big one. Python has references everywhere:

```python
a = [1, 2, 3]
b = a  # Both refer to the same list
b.append(4)
print(a)  # [1, 2, 3, 4] - a changed too!
```

This is convenient until you have a bug where something modified data you thought was immutable.

Rust has **ownership**:

```rust
let a = vec![1, 2, 3];
let b = a;  // Ownership moved to b
a.push(4);  // ERROR: use of moved value
```

After `let b = a`, you cannot use `a` anymore. The value has moved.

### Why This Matters

With ownership, Rust can:
- **Free memory immediately** when the owner goes out of scope
- **Prevent data races** (can't have two mutable references)
- **Guarantee no use-after-free** (compiler tracks lifetimes)

### The Mental Model

Every value has **one owner**. When the owner is destroyed, the value is freed.

You can:
- **Move** ownership: `let b = a;`
- **Borrow** immutably: `let b = &a;` (many readers)
- **Borrow** mutably: `let b = &mut a;` (one writer, no readers)

The compiler enforces these rules at compile time.

### The Mental Shift

Python: "Everything is a reference. Aliasing is everywhere. Good luck."

Rust: "One owner, clear borrowing rules, compiler enforces safety."

**Initial reaction**: "Why can't I just have two mutable references?!"

**After a month**: "Oh, this prevents the iterator invalidation bug I had in Python."

**After six months**: "I want ownership semantics in every language now."

## Common Mistakes Python Developers Make

### 1. Fighting the Borrow Checker

```rust
let mut data = vec![1, 2, 3];
let first = &data[0];
data.push(4);  // ERROR: can't mutate while borrowed
println!("{}", first);
```

**Why it fails**: `push` might reallocate the vector, making `first` point to freed memory.

**Python equivalent** that can cause bugs:
```python
data = [1, 2, 3]
first = data[0]  # This copies the integer
data.append(4)  # Fine
# But if data is a list of lists...
data = [[1], [2], [3]]
first = data[0]  # This is a reference!
data[0].append(4)  # Modifies what first points to
```

**Rust fix**:
```rust
let mut data = vec![1, 2, 3];
let first = data[0];  // Copy the value
data.push(4);  // Now fine
println!("{}", first);
```

### 2. Trying to Mutate Through Shared References

```rust
fn modify(list: &Vec<i32>) {
    list.push(1);  // ERROR: cannot borrow as mutable
}
```

**Fix**: Use `&mut Vec<i32>` if you need to modify.

**Python habit**: In Python, `list` parameters are mutable by default. In Rust, you must explicitly request mutability.

### 3. Returning References to Local Variables

```rust
fn create() -> &str {
    let s = String::from("hello");
    &s  // ERROR: s is dropped at the end of the function
}
```

**Python does this fine** (GC keeps it alive). **Rust forbids it** (would be use-after-free).

**Fix**: Return owned data:
```rust
fn create() -> String {
    String::from("hello")
}
```

### 4. Expecting Null

Python:
```python
def find(list, target):
    for item in list:
        if item == target:
            return item
    return None  # Not found
```

Rust:
```rust
fn find(list: &[i32], target: i32) -> Option<i32> {
    for &item in list {
        if item == target {
            return Some(item);
        }
    }
    None
}
```

**No `null` in Rust.** You use `Option<T>`, which forces you to handle the None case.

### 5. Ignoring Error Handling

Python:
```python
data = json.loads(text)  # Might throw exception
```

Rust:
```rust
let data = serde_json::from_str(text)?;  // Must handle error
```

The `?` operator makes error handling explicit but ergonomic. You can't ignore errors by accident.

## Tips for Successful Transition

### 1. Don't Fight the Compiler

When you get an error, **read it**. Rust's error messages are excellent:

```
error[E0502]: cannot borrow `data` as mutable because it is also borrowed as immutable
  --> src/main.rs:5:5
   |
 4 |     let first = &data[0];
   |                  ---- immutable borrow occurs here
 5 |     data.push(4);
   |     ^^^^^^^^^^^^ mutable borrow occurs here
 6 |     println!("{}", first);
   |                    ----- immutable borrow later used here
```

This tells you:
- What's wrong (conflicting borrows)
- Where it happens (line numbers)
- Why it's wrong (immutable borrow still in use)

### 2. Start with Owned Data

When learning, prefer owned types:

- `String` over `&str`
- `Vec<T>` over `&[T]`
- `.clone()` when you're not sure

This is less efficient but easier to learn. Optimize later when you understand lifetimes.

### 3. Use `cargo clippy`

Clippy is a linter that catches common mistakes and suggests idiomatic Rust:

```bash
cargo clippy
```

It will teach you Rust idioms as you code.

### 4. Read the Compiler Errors Carefully

Don't skim them. Rust's compiler is trying to teach you:

```
help: consider cloning the value if the performance cost is acceptable
  |
3 |     let b = a.clone();
  |              ++++++++
```

### 5. Embrace `Result` and `Option`

Instead of:
```rust
fn parse(s: &str) -> i32 {
    s.parse().unwrap()  // Panics on error
}
```

Write:
```rust
fn parse(s: &str) -> Result<i32, ParseIntError> {
    s.parse()
}
```

This makes errors explicit and handleable.

### 6. Use `match` for Control Flow

Python:
```python
if result is None:
    return default
else:
    return result.value
```

Rust:
```rust
match result {
    None => default,
    Some(value) => value,
}
```

`match` is exhaustive - the compiler ensures you handle all cases.

## Mental Model: Values vs. References

### Python Mental Model

Everything is a reference to an object on the heap:

```
x ----> [1, 2, 3] (heap)
y ----> [1, 2, 3] (heap, different object)
z ----> [1, 2, 3] (heap, same object as x)
```

### Rust Mental Model

Values can be on the stack or heap, and references are explicit:

```
x: Vec<i32>              Stack: [ptr, len, cap] ---> Heap: [1, 2, 3]
y: &Vec<i32>             Stack: [ptr] ---> x's stack location
z: &mut Vec<i32>         Stack: [ptr] ---> x's stack location (exclusive)
```

**Ownership is linear**: Each heap value has exactly one owner.

## Rewiring Your Instincts

### Python Instinct: "I'll figure it out at runtime"

```python
def process(data):
    if isinstance(data, list):
        return sum(data)
    elif isinstance(data, dict):
        return sum(data.values())
    else:
        raise TypeError("Unexpected type")
```

**Rust Rewiring**: "I'll use types to make illegal states unrepresentable"

```rust
enum Data {
    List(Vec<i32>),
    Dict(HashMap<String, i32>),
}

fn process(data: Data) -> i32 {
    match data {
        Data::List(list) => list.iter().sum(),
        Data::Dict(dict) => dict.values().sum(),
    }
}
```

The compiler ensures you handle all cases. No runtime type checking needed.

### Python Instinct: "I'll catch exceptions"

```python
try:
    result = risky_operation()
except Exception as e:
    handle_error(e)
```

**Rust Rewiring**: "I'll use Result to make errors part of the type"

```rust
match risky_operation() {
    Ok(result) => result,
    Err(e) => handle_error(e),
}
```

Errors are values, not control flow. You can't forget to handle them.

### Python Instinct: "I'll mutate data in place"

```python
def add_tax(prices):
    for i in range(len(prices)):
        prices[i] *= 1.1
```

**Rust Rewiring**: "I'll think about whether I own this data"

```rust
fn add_tax(prices: &mut [f64]) {
    for price in prices {
        *price *= 1.1;
    }
}
```

The `&mut` in the signature makes it clear this function mutates its input. Callers must explicitly pass `&mut prices`.

## The Aha Moment

Most Python developers have an "aha moment" around week 3-4:

**Before**: "Why won't the compiler let me do this simple thing?"

**Aha**: "Oh, the compiler just prevented a bug I've had in Python a dozen times."

**After**: "I trust the compiler. If it compiles, it probably works."

This is the mindset shift. The compiler is not your enemy - it's your safety net.

## Resources for Continued Learning

### Official Resources

1. **The Rust Book**: [https://doc.rust-lang.org/book/](https://doc.rust-lang.org/book/)
   - Start here. It's well-written and comprehensive.

2. **Rust by Example**: [https://doc.rust-lang.org/rust-by-example/](https://doc.rust-lang.org/rust-by-example/)
   - Learn by reading code examples.

3. **Rustlings**: [https://github.com/rust-lang/rustlings](https://github.com/rust-lang/rustlings)
   - Small exercises to learn Rust syntax and concepts.

### For Python Developers Specifically

1. **"From Python to Rust"**: Search for blog posts with this title
2. **"Rust for Pythonistas"**: Various talks and articles
3. **PyO3 Documentation**: If you want to call Rust from Python

### Advanced Topics

1. **"The Rustonomicon"**: Unsafe Rust and advanced topics
2. **"Rust Design Patterns"**: Idiomatic Rust patterns
3. **"Rust API Guidelines"**: How to design good Rust APIs

## Conclusion

The Python-to-Rust transition is not about learning syntax. It's about:

- **Trusting the compiler** instead of runtime checks
- **Making invariants explicit** instead of documenting them
- **Thinking about ownership** instead of assuming garbage collection
- **Preventing bugs at compile time** instead of finding them in production

The first few weeks are frustrating. The compiler rejects code that would "obviously work" in Python. But that's because you're learning to see bugs the compiler already sees.

After a few months, you'll write code that compiles on the first try and works correctly. You'll refactor fearlessly because the compiler checks everything.

And then you'll try to go back to Python and think: "Wait, how do I know if this reference is still valid? How do I know if this function modifies its argument? How do I know if this can be null?"

That's when you know you've made the transition. **You're thinking in Rust.**
