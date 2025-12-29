# 22. STRUCT WITH TWO VECS FOR QUEUE

*Two stacks facing opposite directions, like train cars being shuffled from one track to another*

...within a **[STRUCT WITH VEC FIELDS](./21-struct-with-vec-fields.md)**, when you need efficient queue operations (FIFO) but Vec only provides efficient stack operations (LIFO)...

◆ ◆ ◆

**How do you implement an efficient queue when your only efficient data structure is a stack (Vec)?**

A queue needs to be fast at both ends: push to the back, pop from the front. But Vec is only fast at one end. `vec.push(x)` is O(1), and `vec.pop()` is O(1), but removing from the front with `vec.remove(0)` is O(n)—every element must shift down.

You could use a single Vec and accept the slow removal, but that defeats the purpose of having a data structure. You could use a VecDeque from the standard library, but you're learning to build your own structures. The insight is that two stacks can simulate a queue: one for new elements, one for old elements.

When you pop from an empty "older" stack, reverse the "younger" stack onto it. This reversal happens rarely—amortized over many operations, it's O(1). Each element is reversed at most once in its lifetime. The cost is space: you need room for two Vecs, but you gain algorithmic efficiency.

**Therefore:**

**Use two Vec fields: one for newly pushed elements ("younger"), one for elements ready to pop ("older"). When popping from empty older, reverse younger onto it.**

```rust
pub struct Queue {
    older: Vec<char>,   // older elements, eldest last.
    younger: Vec<char>  // younger elements, youngest last.
}

impl Queue {
    pub fn push(&mut self, c: char) {
        self.younger.push(c);
    }

    pub fn pop(&mut self) -> Option<char> {
        if self.older.is_empty() {
            if self.younger.is_empty() {
                return None;
            }

            // Bring the elements in younger over to older, and put them in
            // the promised order.
            use std::mem::swap;
            swap(&mut self.older, &mut self.younger);
            self.older.reverse();
        }

        // Now older is guaranteed to have something. Vec's pop method
        // already returns an Option, so we're set.
        self.older.pop()
    }
}
```

*The younger Vec collects new elements; when older is empty, younger is reversed onto it, restoring FIFO order*

◆ ◆ ◆

This pattern demonstrates **[AMORTIZED CONSTANT TIME](./19-amortized-constant-time.md)** through lazy rebalancing, and becomes more powerful with **[GENERIC TYPE WITH PARAMETER T](./24-generic-type-with-parameter-t.md)** to hold any type, not just char.
