# Implementation Plan - Unified Chat & LLM Routing

This plan outlines the steps to implement a unified chat interface and integrate the Rust-based multi-model provider routing.

## Phase 1: JNI & Core Foundation
- [x] Task: Define JNI interfaces for unified chat message flow [161a023]
    - [x] Define Kotlin `ChatMessage` data class and `ChatBridge` interface [161a023]
    - [x] Implement Rust-side JNI exports for receiving/sending messages [161a023]
- [x] Task: Implement message persistence in Rust core [161a023]
    - [x] Update Sqlite schema for unified chat history [161a023]
    - [x] Implement CRUD operations in `memory_manager.rs` [161a023]
- [ ] Task: Conductor - User Manual Verification 'JNI & Core Foundation' (Protocol in workflow.md)

## Phase 2: Multi-Model Provider Integration
- [ ] Task: Refactor `model_router.rs` for unified provider interface
    - [ ] Define `Provider` trait in Rust
    - [ ] Implement `Anthropic` and `Google` provider backends
- [ ] Task: Implement streaming response handling in Rust core
    - [ ] Update `gateway_server.rs` or `agent_loop.rs` to support stream buffers
    - [ ] Expose streaming events to JNI
- [ ] Task: Conductor - User Manual Verification 'Multi-Model Provider Integration' (Protocol in workflow.md)

## Phase 3: Android Chat UI
- [ ] Task: Create `UnifiedChatScreen` using Jetpack Compose
    - [ ] Implement message list with Material3 `Card` and `Text`
    - [ ] Create sticky input field with multi-line support
- [ ] Task: Implement ViewModel for chat state management
    - [ ] Connect `AgentViewModel` to `RustBridge` for real-time updates
    - [ ] Handle loading states and provider selection logic
- [ ] Task: Conductor - User Manual Verification 'Android Chat UI' (Protocol in workflow.md)

## Phase 4: Integration & Polish
- [ ] Task: End-to-end testing of chat flow with live APIs
    - [ ] Verify message sending/receiving via JNI
    - [ ] Test multiple LLM providers (Google, Anthropic)
- [ ] Task: UI polish and accessibility review
    - [ ] Ensure WCAG AA compliance for chat bubbles and contrast
    - [ ] Polish transitions and loading animations
- [ ] Task: Conductor - User Manual Verification 'Integration & Polish' (Protocol in workflow.md)
