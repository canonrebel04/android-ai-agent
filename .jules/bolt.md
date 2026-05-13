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

## 2026-05-12 - Avoid inline collection operations in Compose build blocks
**Learning:** Operations like `.filter` inside a `LazyColumn` builder block execute during every recomposition of the screen. In screens with frequent state updates (like model download progress), this repeatedly allocates memory and performs O(N) filtering, degrading performance.
**Action:** Memoize derived collections using `remember(dependency) { ... }` so the operation only runs when the underlying data source actually changes.

## 2026-05-18 - Avoid O(N^2) allocations in CopyOnWriteArrayList modifications
**Learning:** When updating elements in a `CopyOnWriteArrayList`, modifying items individually in a `for` loop causes O(N^2) allocations since it creates a new copy of the underlying array for each modification. Additionally, using the Kotlin extension function `list.replaceAll { ... }` causes an `UnsupportedOperationException` crash due to Kotlin's `MutableList` iterator resolution.
**Action:** Use `replaceAll`, but explicitly use the Java member method `list.replaceAll(java.util.function.UnaryOperator { ... })` to perform a bulk update, avoiding the crash and reducing the operation to O(1) array allocation.
