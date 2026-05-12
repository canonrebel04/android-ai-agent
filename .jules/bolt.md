## 2025-02-23 - ContextManager::trim O(N^2) bottleneck
**Learning:** `ContextManager::trim` iteratively recalculates total token counts and repeatedly removes items from the head of the `Vec` inside a `while` loop, leading to an O(N^2) time complexity. For large context windows (like ~20k messages), `trim` synchronously blocked the thread for >3 seconds.
**Action:** Replaced iterative removal with O(N) pre-calculation of total characters and a bulk removal via `Vec::drain`. Next time, watch out for `Vec::remove(0)` patterns inside loops when processing unbounded conversation context arrays.

## 2025-05-05 - Avoid dynamic Regex compilation in hot loops
**Learning:** The Rust codebase frequently compiled identical `Regex` patterns inside functions (e.g., `Regex::new(...).unwrap()`) that were called multiple times (like `extract_entities` or `extract_urls`). This repeats the expensive regex parsing/compilation steps on every invocation. The codebase convention for preventing this performance bottleneck is to statically instantiate the compiled regexes once using `once_cell::sync::Lazy`.
**Action:** Use `once_cell::sync::Lazy` to compile static regular expressions instead of repeatedly calling `Regex::new()` inside frequently executed functions.

## 2025-05-06 - Avoid unnecessary recompositions in Compose LazyColumn
**Learning:** By default, Jetpack Compose uses the item's index as the key in `LazyColumn` / `itemsIndexed`. When items are added or list order changes, this causes Compose to unnecessarily recompose existing list items, which can degrade scrolling performance in chat UIs or long lists.
**Action:** Always provide a stable, unique `key` (like `message.id`) to `itemsIndexed` or `items` inside Compose `LazyColumn`s to prevent unnecessary recompositions and improve UI performance.

## 2026-05-18 - Avoid dynamic Regex compilation in Kotlin methods
**Learning:** In Kotlin/Android, `java.util.regex.Pattern.compile` is an expensive operation. Placing it inside a frequently invoked method (such as `parseAnsi` which is called per output line in `TerminalScreen.kt`) creates a significant performance bottleneck.
**Action:** Extract `Pattern.compile` calls to static constants (e.g. inside an `object` singleton or `companion object`) to ensure the regular expression is only compiled once, as `Pattern` instances are thread-safe and immutable.

## 2025-05-19 - Bulk update in CopyOnWriteArrayList
**Learning:** Iterating through a `CopyOnWriteArrayList` and replacing elements by index (e.g. `list[i] = ...`) allocates a new backing array for every single modification, resulting in O(N^2) time complexity and memory overhead.
**Action:** When performing bulk conditional updates on `CopyOnWriteArrayList`, use `replaceAll { ... }` instead to do it in O(N) time with only a single array allocation.
