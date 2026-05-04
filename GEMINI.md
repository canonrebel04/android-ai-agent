# Android AI Agent

An autonomous AI agent for Android that combines a high-performance **Rust core** with an **Android Accessibility Service** to perceive and interact with other applications.

## Project Overview

The project follows a split-architecture pattern:
- **Rust Core (`src/`)**: Handles heavy lifting including model routing (OpenRouter, Anthropic, Google, Mistral), context management, safety policy enforcement, and tool parsing. It is compiled as a JNI library (`libagent_core.so`).
- **Android App (`app/`)**: A Jetpack Compose application that hosts the `AccessibilityService` (for screen perception and gesture injection), a floating overlay UI, and various background services (Telegram bot, notification monitoring).

### Key Features
- **Screen Perception**: Uses `AccessibilityService` to dump UI trees and understand app state.
- **Action Injection**: Can perform taps, swipes, and text input on behalf of the user.
- **Multi-Model Support**: Routes requests to various LLM providers based on complexity and cost.
- **Safety Loop**: Implements a "confirm before act" loop for sensitive operations.
- **Memory Management**: Sophisticated context windowing and fact extraction in Rust.

---

## Building and Running

### Prerequisites
- **Rust 1.85+** with Android targets (`aarch64-linux-android`, etc.)
- **Android Studio** with **NDK r27+**
- **cargo-ndk**: `cargo install cargo-ndk`

### 1. Build Rust Core (JNI)
Cross-compile the Rust library for Android architectures:
```bash
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o ./jniLibs build --release
```
This populates the `jniLibs/` directory with `.so` files which are then used by the Android app.

### 2. Build Android App
Use the Gradle wrapper to build the APK:
```bash
./gradlew assembleDebug
```
The resulting APK will be at `app/build/outputs/apk/debug/app-debug.apk`.

### 3. Deploy and Test
Install the APK via ADB:
```bash
adb install app/build/outputs/apk/debug/app-debug.apk
```

To run Rust unit tests (on host):
```bash
cargo test
```

---

## Development Conventions

### Architecture Patterns
- **JNI Bridge**: Communication between Kotlin and Rust is handled in `com.yourdomain.agent.RustBridge` and `src/jni_exports.rs`.
- **Accessibility Integration**: `AgentAccessibilityService.kt` is the primary entry point for screen interaction. It must be enabled in Android's Accessibility Settings.
- **Theme & UI**: Built with Material3 and Jetpack Compose. Screens are located in `app/src/main/kotlin/com/yourdomain/agent/`.

### Coding Standards
- **Rust**: Use `jni 0.22` with the `EnvUnowned` pattern. Prefer the `with_env` closure for JNI calls.
- **Kotlin**: Use standard Android naming conventions and Compose-first patterns.
- **Security**: Sensitive keys should be stored in the Android Keystore via `KeystoreManager.kt`. Never log raw API keys.

### Documentation
- `BUILD.md`: Detailed build instructions and dependency mapping.
- `TEST_MATRIX.md`: Coverage and verification status for different modules.
- `android-ai-agent-migration/`: Contains implementation plans and persona context for the "Hermes" agent.

---

## Repository Structure

- `src/`: Rust core implementation logic.
- `app/`: Android Studio project (Kotlin/Compose).
- `jniLibs/`: Pre-compiled Rust binaries for Android.
- `rust/`: Minimal Rust project for host-side testing/utilities.
- `examples/`: Rust smoke tests and gateway server examples.
- `android-ai-agent-migration/`: Project SOUL and Phase-based implementation history.
