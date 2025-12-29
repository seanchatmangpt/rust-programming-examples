# Summary

[Introduction](./introduction.md)
[The Pattern Form](./pattern-form.md)
[How to Use This Book](./how-to-use.md)

---

# Part I: Project Scale

- [1. Binary with Main Function](./patterns/01-binary-with-main-function.md)
- [2. Library Crate with Public API](./patterns/02-library-crate-with-public-api.md)
- [3. Binary and Library Together](./patterns/03-binary-and-library-together.md)
- [4. Tests Directory Beside Source](./patterns/04-tests-directory-beside-source.md)
- [5. Examples Directory for Usage](./patterns/05-examples-directory-for-usage.md)
- [6. Async Runtime with Attribute Main](./patterns/06-async-runtime-with-attribute-main.md)
- [7. Unsafe FFI Wrapper Crate](./patterns/07-unsafe-ffi-wrapper-crate.md)
- [8. Safe Wrapper Around Unsafe](./patterns/08-safe-wrapper-around-unsafe.md)

---

# Part II: Architecture Scale

- [9. Module Tree in Lib File](./patterns/09-module-tree-in-lib-file.md)
- [10. Submodule in Separate File](./patterns/10-submodule-in-separate-file.md)
- [11. Nested Submodules in Directory](./patterns/11-nested-submodules-in-directory.md)
- [12. Private Module Public Reexport](./patterns/12-private-module-public-reexport.md)
- [13. Feature-Based Module Groups](./patterns/13-feature-based-module-groups.md)
- [14. Flat Module All in Lib](./patterns/14-flat-module-all-in-lib.md)
- [15. Test Module with Use Super Star](./patterns/15-test-module-with-use-super-star.md)
- [16. Raw Bindings Module](./patterns/16-raw-bindings-module.md)
- [17. Public Facade Module](./patterns/17-public-facade-module.md)
- [18. Crate Root Reexporting Core](./patterns/18-crate-root-reexporting-core.md)
- [19. Build Script for C Dependencies](./patterns/19-build-script-for-c-dependencies.md)
- [20. Conditional Compilation by OS](./patterns/20-conditional-compilation-by-os.md)

---

# Part III: Type Scale

- [21. Struct with Vec Fields](./patterns/21-struct-with-vec-fields.md)
- [22. Struct with Two Vecs for Queue](./patterns/22-struct-with-two-vecs-for-queue.md)
- [23. Enum with Empty and NonEmpty](./patterns/23-enum-with-empty-and-nonempty.md)
- [24. Generic Type with Parameter T](./patterns/24-generic-type-with-parameter-t.md)
- [25. Trait Bound on Impl Block](./patterns/25-trait-bound-on-impl-block.md)
- [26. Newtype Wrapping Raw Pointer](./patterns/26-newtype-wrapping-raw-pointer.md)
- [27. PhantomData for Lifetime](./patterns/27-phantomdata-for-lifetime.md)
- [28. Derive Debug for Testing](./patterns/28-derive-debug-for-testing.md)
- [29. Derive Deserialize for Forms](./patterns/29-derive-deserialize-for-forms.md)
- [30. Custom Error Struct with Display](./patterns/30-custom-error-struct-with-display.md)
- [31. Type Alias for Result](./patterns/31-type-alias-for-result.md)
- [32. Unit Struct for Marker](./patterns/32-unit-struct-for-marker.md)

---

# Part IV: Function Scale

- [33. Method Taking Self by Reference](./patterns/33-method-taking-self-by-reference.md)
- [34. Method Taking Self by Mut Reference](./patterns/34-method-taking-self-by-mut-reference.md)
- [35. Method Consuming Self](./patterns/35-method-consuming-self.md)
- [36. Constructor Function Named New](./patterns/36-constructor-function-named-new.md)
- [37. Builder Method Returning Self](./patterns/37-builder-method-returning-self.md)
- [38. Function Returning Result](./patterns/38-function-returning-result.md)
- [39. Async Function with Await](./patterns/39-async-function-with-await.md)
- [40. Unsafe Function with Safety Comment](./patterns/40-unsafe-function-with-safety-comment.md)
- [41. Generic Function with Where Clause](./patterns/41-generic-function-with-where-clause.md)
- [42. Function Taking AsRef Path](./patterns/42-function-taking-asref-path.md)

---

# Part V: Expression Scale

- [43. Match on Result with Question Mark](./patterns/43-match-on-result-with-question-mark.md)
- [44. If Let for Option Unwrapping](./patterns/44-if-let-for-option-unwrapping.md)
- [45. While Let for Iteration](./patterns/45-while-let-for-iteration.md)
- [46. For Loop over Borrowed Reference](./patterns/46-for-loop-over-borrowed-reference.md)
- [47. Assert Macro in Function Body](./patterns/47-assert-macro-in-function-body.md)
- [48. Mem Swap for Moving Values](./patterns/48-mem-swap-for-moving-values.md)
- [49. Clone to Extend Lifetime](./patterns/49-clone-to-extend-lifetime.md)
- [50. Test Function with Attribute](./patterns/50-test-function-with-attribute.md)

---

# Reference

[Pattern Catalog](./pattern-catalog.md)
[Pattern Map](./pattern-map.md)
