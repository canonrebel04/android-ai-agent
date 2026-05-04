## 2024-05-27 - [Gateway Server Missing Authentication Check]
**Vulnerability:** The GatewayServer component in `src/gateway_server.rs` accepted an authentication token via `.with_auth()` and passed it into the connection handler, but the `handle_connection` function was not validating the token at all.
**Learning:** This is a case where the setup code implies an auth check, but the actual request handler simply ignores the token parameter, leaving the gateway completely unprotected.
**Prevention:** Make sure authentication logic is actually implemented and enforced inside the request/connection handler, not just passed around and ignored. Add tests that specifically try to connect without a token and fail.
