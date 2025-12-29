# 46. FOR LOOP OVER BORROWED REFERENCE

*A programmer sits before a collection of file paths, needing to process each one without consuming the collection itself.*

...within a **COLLECTION ITERATION** (43), when you need to visit each element but still use the collection afterward...

◆ ◆ ◆

**How do you iterate over a collection without losing access to it?**

In many languages, iteration automatically borrows or references collection elements. But Rust's ownership system requires explicit choice: consume the collection by moving, or borrow it by referencing.

The `for` loop syntax in Rust implements the `IntoIterator` trait, which by default takes ownership of the collection. This means after the loop, your collection is gone—moved into the iterator and consumed. For many algorithms, this is unacceptable: you may need the collection again, or you may be working with a borrowed collection you don't own.

Consider the grep program that searches multiple files. Each file path needs to be opened and searched, but the list of paths should remain available—perhaps to count them, log them, or retry failed searches. If the loop consumed the collection, this would be impossible.

The solution lies in Rust's three iteration modes: by value (`collection`), by shared reference (`&collection`), and by mutable reference (`&mut collection`). When you iterate by reference, you borrow each element without moving the collection itself.

**Therefore:**

**Iterate over a shared reference to the collection using `for item in &collection`, which borrows elements and preserves the collection for future use.**

```rust
// From grep/src/main.rs - iterate over file paths without consuming them
fn grep_main() -> Result<(), Box<dyn Error>> {
    let files: Vec<PathBuf> = args.map(PathBuf::from).collect();

    // Iterate over borrowed reference - collection remains valid
    for file in &files {
        let f = File::open(file)?;
        grep(&target, BufReader::new(f))?;
    }

    // files is still accessible here
    Ok(())
}

// From binary-tree/src/lib.rs - iterate over shared reference
for kind in &tree {
    v.push(*kind);
}
// tree is still accessible here
```

*The ampersand before the collection creates a shared borrow: each loop iteration receives a reference to an element, and when the loop ends, the original collection remains intact, ready for further use.*

◆ ◆ ◆

Use **MUTABLE BORROWED ITERATION** when elements need modification; use **OWNED ITERATION** when the collection will not be needed again; combine with **ITERATOR ADAPTERS** (52) to transform elements while borrowing.
