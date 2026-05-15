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
## 2024-05-14 - Optimize bulk updates in CopyOnWriteArrayList
**Learning:** Modifying individual elements in a `CopyOnWriteArrayList` within a loop causes O(N^2) complexity because a new array is allocated for each modification.
**Action:** Use `replaceAll` for bulk updates on `CopyOnWriteArrayList` to allocate the backing array only once.

## 2024-05-14 - Kotlin extension function vs Java member method resolution gotcha
**Learning:** In Kotlin, `CopyOnWriteArrayList` is treated as a `MutableList`. If we use `messages.replaceAll { ... }`, Kotlin prioritizes the `MutableList<T>.replaceAll(operator: (T) -> T)` extension function over the Java `replaceAll(UnaryOperator<E>)` member method. Kotlin's extension function implementation uses a standard iterator and calls `.set()` for every item. However, `CopyOnWriteArrayList`'s iterator does not support mutation, resulting in an unconditional `UnsupportedOperationException` and an application crash.
**Action:** When calling Java member functions that take SAM (Single Abstract Method) interfaces like `UnaryOperator` on Java collections that implement interfaces mapped to Kotlin standard library equivalents (like `List` -> `MutableList`), explicitly cast the lambda (e.g. `messages.replaceAll(java.util.function.UnaryOperator { ... })`) to force the use of the Java member method and avoid incompatible Kotlin extension functions.

## 2026-05-18 - Optimize chained sequence operations in Kotlin log parsing
**Learning:** In Kotlin, chaining operations like `logs.split("\n").filter { ... }` on large strings creates multiple intermediate lists in memory (one for the split result, one for the filtered result).
**Action:** Use `lineSequence().filter { ... }.toList()` instead to process elements lazily, significantly reducing memory allocation overhead without sacrificing code readability.
