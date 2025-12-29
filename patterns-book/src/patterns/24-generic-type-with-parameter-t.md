# 24. GENERIC TYPE WITH PARAMETER T

*A template that works for any shape, like a universal shipping container*

...within a **[CUSTOM DATA TYPE](./20-custom-data-type.md)**, when your structure's logic doesn't depend on the specific type of data it holds...

◆ ◆ ◆

**How do you write a data structure that works with any type, without duplicating code or losing type safety?**

You've built a queue that holds chars. Now you need one for integers. Then strings. Then custom types. You could copy-paste the code, changing `char` to `i32`, but this violates DRY. You could use `Box<dyn Any>` and runtime type checking, but that loses compile-time safety and requires downcasting.

The tension is between reusability and type safety. Dynamic typing gives you reusability but loses safety. Copy-pasting gives you safety but loses reusability. Generics give you both: one implementation, many types, all checked at compile time. The compiler generates specialized code for each concrete type you use.

Generic parameters are compile-time placeholders. When you write `Queue<i32>`, the compiler generates code specifically for i32. When you write `Queue<String>`, it generates different code for String. There's no runtime overhead—generics are a zero-cost abstraction. You pay nothing for the flexibility.

**Therefore:**

**Add a generic type parameter `<T>` to your struct or enum. Replace concrete types with T throughout. The compiler will generate specialized code for each concrete type.**

```rust
pub struct Queue<T> {
    older: Vec<T>,
    younger: Vec<T>
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Queue { older: Vec::new(), younger: Vec::new() }
    }

    pub fn push(&mut self, t: T) {
        self.younger.push(t);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.older.is_empty() {
            use std::mem::swap;

            if self.younger.is_empty() {
                return None;
            }

            swap(&mut self.older, &mut self.younger);
            self.older.reverse();
        }

        self.older.pop()
    }
}

// Now works with any type:
let mut chars = Queue::<char>::new();
let mut numbers = Queue::<i32>::new();
let mut strings = Queue::<String>::new();
```

*The same code works for any type T, with full type safety and zero runtime cost*

◆ ◆ ◆

When your generic type needs specific capabilities, use **[TRAIT BOUND ON IMPL BLOCK](./25-trait-bound-on-impl-block.md)** to constrain T. When you need multiple generic parameters, add more: `<T, U>`.
