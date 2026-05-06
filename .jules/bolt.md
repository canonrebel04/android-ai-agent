## 2025-02-23 - ContextManager::trim O(N^2) bottleneck
**Learning:** `ContextManager::trim` iteratively recalculates total token counts and repeatedly removes items from the head of the `Vec` inside a `while` loop, leading to an O(N^2) time complexity. For large context windows (like ~20k messages), `trim` synchronously blocked the thread for >3 seconds.
**Action:** Replaced iterative removal with O(N) pre-calculation of total characters and a bulk removal via `Vec::drain`. Next time, watch out for `Vec::remove(0)` patterns inside loops when processing unbounded conversation context arrays.
## 2024-05-24 - Regex compilation overhead
**Learning:** Compiling regex patterns using `Regex::new` inside hot paths like entity extraction (`fact_index.rs`) is extremely expensive and can cause severe bottlenecks, specifically when executed for many entries or continuously.
**Action:** Use `once_cell::sync::Lazy` (or similar lazy-static abstractions) to cache regex compilations so they are only compiled once during the program's lifecycle.
