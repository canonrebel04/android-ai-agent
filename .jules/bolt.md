## 2025-02-23 - ContextManager::trim O(N^2) bottleneck
**Learning:** `ContextManager::trim` iteratively recalculates total token counts and repeatedly removes items from the head of the `Vec` inside a `while` loop, leading to an O(N^2) time complexity. For large context windows (like ~20k messages), `trim` synchronously blocked the thread for >3 seconds.
**Action:** Replaced iterative removal with O(N) pre-calculation of total characters and a bulk removal via `Vec::drain`. Next time, watch out for `Vec::remove(0)` patterns inside loops when processing unbounded conversation context arrays.

## 2025-05-05 - Avoid dynamic Regex compilation in hot loops
**Learning:** The Rust codebase frequently compiled identical `Regex` patterns inside functions (e.g., `Regex::new(...).unwrap()`) that were called multiple times (like `extract_entities` or `extract_urls`). This repeats the expensive regex parsing/compilation steps on every invocation. The codebase convention for preventing this performance bottleneck is to statically instantiate the compiled regexes once using `once_cell::sync::Lazy`.
**Action:** Use `once_cell::sync::Lazy` to compile static regular expressions instead of repeatedly calling `Regex::new()` inside frequently executed functions.
