## 2025-02-23 - ContextManager::trim O(N^2) bottleneck
**Learning:** `ContextManager::trim` iteratively recalculates total token counts and repeatedly removes items from the head of the `Vec` inside a `while` loop, leading to an O(N^2) time complexity. For large context windows (like ~20k messages), `trim` synchronously blocked the thread for >3 seconds.
**Action:** Replaced iterative removal with O(N) pre-calculation of total characters and a bulk removal via `Vec::drain`. Next time, watch out for `Vec::remove(0)` patterns inside loops when processing unbounded conversation context arrays.

## 2025-05-05 - Avoid dynamic Regex compilation in hot loops
**Learning:** The Rust codebase frequently compiled identical `Regex` patterns inside functions (e.g., `Regex::new(...).unwrap()`) that were called multiple times (like `extract_entities` or `extract_urls`). This repeats the expensive regex parsing/compilation steps on every invocation. The codebase convention for preventing this performance bottleneck is to statically instantiate the compiled regexes once using `once_cell::sync::Lazy`.
**Action:** Use `once_cell::sync::Lazy` to compile static regular expressions instead of repeatedly calling `Regex::new()` inside frequently executed functions.

## 2026-05-06 - Jetpack Compose LazyColumn Missing Keys
**Learning:** In Jetpack Compose, missing unique keys in `LazyColumn` items causes unnecessary and expensive recompositions when elements are inserted, removed, or re-ordered, particularly noticeable in dynamic lists like the chat interface. Adding stable, unique keys ensures proper component recycling and smoother scrolling.
**Action:** Always provide a stable, unique `key` parameter to `items` or `itemsIndexed` within a `LazyColumn` to prevent redundant recompositions when data changes.
