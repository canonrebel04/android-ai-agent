## 2024-05-27 - [Gateway Server Missing Authentication Check]
**Vulnerability:** The GatewayServer component in `src/gateway_server.rs` accepted an authentication token via `.with_auth()` and passed it into the connection handler, but the `handle_connection` function was not validating the token at all.
**Learning:** This is a case where the setup code implies an auth check, but the actual request handler simply ignores the token parameter, leaving the gateway completely unprotected.
**Prevention:** Make sure authentication logic is actually implemented and enforced inside the request/connection handler, not just passed around and ignored. Add tests that specifically try to connect without a token and fail.

## 2026-05-05 - [Batching queries to avoid N+1 problem]
**Learning:** Calling the database sequentially in a nested loop (such as iterating through entity pairs or list of entities) leads to severe performance degradation via N+1 queries. We can eliminate this by creating batched APIs (like `probe_batched` using `OR`) that allow retrieving multiple items with a single query, followed by in-memory processing.
**Action:** When implementing routines that lookup multiple facts, ensure the underlying `FactStore` provides a batch API rather than looping over single-entity lookup queries.
