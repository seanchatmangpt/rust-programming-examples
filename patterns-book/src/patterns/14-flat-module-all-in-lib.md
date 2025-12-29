# 14. FLAT MODULE ALL IN LIB

*A simple library where all code lives in a single lib.rs file, with internal modules used only for organization and testing, not for splitting across multiple files*

...within a [SIMPLE LIBRARY](#single-purpose), when your entire implementation fits comfortably in one file...

◆ ◆ ◆

**When should you resist the temptation to create a directory structure and keep everything in one file?**

Premature modularization wastes time. Creating directories, moving code between files, and maintaining module declarations adds friction. For small libraries with focused functionality, this overhead costs more than it saves. The `complex` crate demonstrates this—it provides complex number operations in under 300 lines, yet it defines *eight* separate modules in one file.

But these aren't file-based modules. They're conceptual partitions using `mod { }` blocks: `first_cut`, `non_generic_add`, `somewhat_generic`, `very_generic`, `impl_compound`, and `formatting`. Each explores a different API design. The file-level grouping actually helps here—you can compare approaches side-by-side, see how they differ, and understand the evolution without opening multiple files.

A single file also maintains locality. When debugging, you don't hunt through directories—you scroll. When refactoring, you don't update file paths—you move code blocks. For libraries under 500-1000 lines with clear internal structure, this simplicity wins.

The breaking point comes when you lose the mental model. If you can't hold the file's structure in your head, or if different logical sections would benefit from independent testing or development, split it. But not before.

**Therefore:**

**Keep library code in lib.rs until the file becomes unwieldy (>500-1000 lines) or when modules represent truly independent concerns that would benefit from separate compilation units.**

```rust
// complex/src/lib.rs - entire library in one file

// Module for explaining operator overloading progression
mod first_cut {
    #[derive(Clone, Copy, Debug)]
    struct Complex<T> {
        re: T,
        im: T,
    }

    impl<T: Add<Output = T>> Add for Complex<T> {
        type Output = Self;
        fn add(self, rhs: Self) -> Self {
            Complex {
                re: self.re + rhs.re,
                im: self.im + rhs.im,
            }
        }
    }

    #[test]
    fn try_it_out() {
        let z = Complex { re: 1, im: 2 };
        // ... tests inline with code
    }
}

// More variations exploring different designs
mod somewhat_generic { /* ... */ }
mod very_generic { /* ... */ }
```

*The diagram shows a single file lib.rs containing multiple mod blocks arranged vertically, with dotted lines between them indicating conceptual separation without file separation.*

◆ ◆ ◆

This pattern contrasts with [NESTED SUBMODULES IN DIRECTORY](#11). When it grows, migrate to [FEATURE-BASED MODULE GROUPS](#13). It uses [TEST MODULE WITH USE SUPER STAR](#15) for testing.
