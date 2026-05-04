# Tech Stack

## Core Languages
- **Rust (2021 Edition):** Primary language for performance-critical core logic, asynchronous operations, and multi-model orchestration.
- **Kotlin:** Primary language for the Android application layer and system-level integrations.

## Frontend Frameworks
- **Jetpack Compose:** Used for building a clean, modern, and reactive user interface on Android.
- **Material3:** Implementation of Material Design 3 for a consistent and modern aesthetic.

## System & Integration
- **Android SDK:** Provides access to Accessibility Services, Notification Monitoring, and other OS-level capabilities.
- **JNI (Java Native Interface):** Facilitates high-speed communication between the Kotlin UI and the Rust core (`libagent_core.so`).

## Backend & Utilities (Rust)
- **Tokio:** Provides the asynchronous runtime for handling non-blocking I/O and concurrent tasks.
- **Reqwest:** Handles all outgoing HTTP requests to various LLM provider APIs.
- **Serde:** Comprehensive serialization and deserialization framework for JSON, YAML, and TOML.
- **Sqlite (rusqlite):** Local persistent storage for agent memory, facts, and settings.

## Infrastructure & Build
- **Gradle:** Build system for the Android application.
- **Cargo:** Package manager and build system for the Rust core.
- **cargo-ndk:** Tooling for cross-compiling Rust modules for Android architectures (arm64-v8a, armeabi-v7a, x86_64).