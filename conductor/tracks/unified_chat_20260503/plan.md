# Implementation Plan - Unified Chat & LLM Routing

This plan outlines the steps to implement a unified chat interface and integrate the Rust-based multi-model provider routing.

## Phase 1: JNI & Core Foundation [checkpoint: 16d4708]
- [x] Task: Define JNI interfaces for unified chat message flow [161a023]
    - [x] Define Kotlin `ChatMessage` data class and `ChatBridge` interface [161a023]
    - [x] Implement Rust-side JNI exports for receiving/sending messages [161a023]
- [x] Task: Implement message persistence in Rust core [161a023]
    - [x] Update Sqlite schema for unified chat history [161a023]
    - [x] Implement CRUD operations in `memory_manager.rs` [161a023]
- [x] Task: Conductor - User Manual Verification 'JNI & Core Foundation' (Protocol in workflow.md) [16d4708]

## Phase 2: Multi-Model Provider Integration [checkpoint: bca8b16]
- [x] Task: Refactor `model_router.rs` for unified provider interface [bca8b16]
    - [x] Define `ProviderBackend` enum in Rust [bca8b16]
    - [x] Implement `Anthropic` and `Google` provider backends [bca8b16]
- [x] Task: Implement streaming response handling in Rust core [bca8b16]
    - [x] Update `gateway_server.rs` to support stream variant [bca8b16]
    - [x] Implement `call_stream` for OpenRouter and Anthropic [bca8b16]
- [~] Task: Conductor - User Manual Verification 'Multi-Model Provider Integration' (Protocol in workflow.md) [bca8b16]

## Phase 3: Android Chat UI [checkpoint: 203ecf7]
- [x] Task: Create `UnifiedChatScreen` using Jetpack Compose [203ecf7]
    - [x] Implement message list with Material3 `Card` and `Text` [203ecf7]
    - [x] Create sticky input field with multi-line support [203ecf7]
    - [x] Apply Cyberpunk Terminal aesthetic and animations [203ecf7]
- [x] Task: Implement ViewModel for chat state management [203ecf7]
    - [x] Connect `AgentViewModel` to `RustBridge` for real-time updates [203ecf7]
    - [x] Handle loading states and provider selection logic [203ecf7]
- [~] Task: Conductor - User Manual Verification 'Android Chat UI' (Protocol in workflow.md) [203ecf7]

## Phase 4: Integration & Polish
- [~] Task: End-to-end testing of chat flow with live APIs
    - [ ] Verify message sending/receiving via JNI
    - [ ] Test multiple LLM providers (Google, Anthropic)
- [ ] Task: UI polish and accessibility review
    - [ ] Ensure WCAG AA compliance for chat bubbles and contrast
    - [ ] Polish transitions and loading animations
- [ ] Task: Conductor - User Manual Verification 'Integration & Polish' (Protocol in workflow.md)
