# Specification - Unified Chat & LLM Routing

## Goal
Implement a unified, clean chat interface in the Android application and integrate the Rust-based multi-model routing core to support various LLM providers (Google, Anthropic, OpenRouter, etc.).

## Requirements
- **Unified UI:** A single, distraction-free chat screen built with Jetpack Compose and Material3.
- **Direct Filesystem Access:** The chat agent should have the ability to read, write, and manage files on the Linux filesystem (Android storage).
- **Multi-Model Integration:** Support for switching between different LLM providers via a unified interface in the Rust core.
- **Real-time Interaction:** Streaming responses from LLMs should be reflected in the chat UI.
- **Standalone Package:** All logic should be self-contained within the APK, leveraging the Rust core for heavy processing.

## Architecture
- **Rust Core:** Handles API calls, model routing, and filesystem operations.
- **JNI Bridge:** Passes messages and events between the Rust core and the Kotlin UI.
- **Kotlin UI:** A Material3-based chat frontend using Jetpack Compose and ViewModels.
