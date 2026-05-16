## 2024-05-27 - [Gateway Server Missing Authentication Check]
**Vulnerability:** The GatewayServer component in `src/gateway_server.rs` accepted an authentication token via `.with_auth()` and passed it into the connection handler, but the `handle_connection` function was not validating the token at all.
**Learning:** This is a case where the setup code implies an auth check, but the actual request handler simply ignores the token parameter, leaving the gateway completely unprotected.
**Prevention:** Make sure authentication logic is actually implemented and enforced inside the request/connection handler, not just passed around and ignored. Add tests that specifically try to connect without a token and fail.

## 2024-05-27 - [Plaintext API Key Storage]
**Vulnerability:** The TelegramBotService, SettingsScreen, and ChannelsScreen were storing and reading the `telegramToken` in plain text using Android's default SharedPreferences (`AgentPrefs`). This is a critical security vulnerability as plain text storage can easily be extracted by malicious actors or applications with root access.
**Learning:** Even though KeystoreManager exists in the project and provides secure AES-256/GCM encryption backed by the Android Keystore, legacy components were still directly interacting with plain-text SharedPreferences for token storage and retrieval.
**Prevention:** Always ensure that new UI screens or background services interacting with authentication keys or sensitive tokens utilize the secure `KeystoreManager` class. Additionally, when refactoring insecure legacy code, it is crucial to provide a migration path so users do not lose their currently configured API tokens.

## 2024-05-28 - [Gateway Server Timing Attack on Auth Token]
**Vulnerability:** The GatewayServer component in `src/gateway_server.rs` used a standard string equality check (`==`) to validate the authentication token in `handle_connection`. This is vulnerable to timing attacks.
**Learning:** Even if authentication logic is implemented, using standard equality operators for sensitive tokens can leak token structure byte-by-byte.
**Prevention:** Always use constant-time comparisons for sensitive secrets or tokens. In this environment, a secure manual comparison using a cryptographic hash like `sha2::Sha256::digest` normalizes the length and bitwise operations can hide timing side-channels.

## 2026-05-05 - [Batching queries to avoid N+1 problem]
**Learning:** Calling the database sequentially in a nested loop (such as iterating through entity pairs or list of entities) leads to severe performance degradation via N+1 queries. We can eliminate this by creating batched APIs (like `probe_batched` using `OR`) that allow retrieving multiple items with a single query, followed by in-memory processing.
**Action:** When implementing routines that lookup multiple facts, ensure the underlying `FactStore` provides a batch API rather than looping over single-entity lookup queries.
