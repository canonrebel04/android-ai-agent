# Android AI Agent

An autonomous AI agent for Android that combines a high-performance **Rust core** with an **Android Accessibility Service** to perceive and interact with other applications.

## Project Overview

The project follows a split-architecture pattern:
- **Rust Core (`src/`)**: Handles heavy lifting including model routing (OpenRouter, Anthropic, Google, Mistral, DeepSeek), context management, safety policy enforcement, and tool parsing. It is compiled as a JNI library (`libagent_core.so`).
- **Android App (`app/`)**: A Jetpack Compose application that hosts the `AccessibilityService` (for screen perception and gesture injection), a floating overlay UI, and various background services (Telegram bot, notification monitoring).

### Key Features
- **Screen Perception**: Uses `AccessibilityService` to dump UI trees and understand app state.
- **Action Injection**: Can perform taps, swipes, and text input on behalf of the user.
- **Multi-Model Support**: Routes requests to various LLM providers based on task complexity.
- **Safety Loop**: Implements a "confirm before act" loop for sensitive operations.
- **Memory Management**: Sophisticated context windowing and fact extraction in Rust.
- **Notification Monitoring**: Listens for notifications from 6 social apps and can trigger agent actions.
- **Floating Overlay**: Always-on-top status indicator showing agent state and recent notifications.
- **Telegram Integration**: Remote control via Telegram bot.

---

## Building and Running

### Prerequisites
- **Rust 1.85+** with Android targets (`aarch64-linux-android`, etc.)
- **Android Studio** with **NDK r29**
- **cargo-ndk**: `cargo install cargo-ndk`
- **Java 17+** for Gradle builds

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

**Test Status**: ✅ **107 tests passing** (as of latest commit)

---

## Features

### Core Capabilities

#### 1. Multi-Provider LLM Support
- **OpenRouter**: Default provider with 300+ models
- **Anthropic**: Claude Sonnet, Opus, and Haiku
- **Google**: Gemini Flash and Pro models
- **Mistral**: Mistral Small, Medium, and Large
- **DeepSeek**: V4 Flash and Pro models
- **Local**: Offline models (free)

**Model Pricing Transparency**: The app displays current API costs per model for user awareness. Note: We do NOT track or enforce budget limits - API costs are controlled by the providers.

#### 2. Task Complexity Classification
Automatically classifies tasks into 4 tiers:
- **Trivial**: Simple queries, short responses
- **Standard**: Informational questions, moderate length
- **Complex**: Code generation, multi-step tasks
- **Critical**: Destructive actions (send, delete, pay, etc.)

The classifier provides **model suggestions** based on complexity, but the user's configured model always takes precedence.

#### 3. Screen Control (AccessibilityService)
Full screen interaction capabilities:
- **Node Finding**: Find UI elements by text (exact or partial match)
- **Tapping**: Tap at coordinates or on specific nodes
- **Swiping**: Swipe between any two points
- **Scrolling**: Scroll up/down
- **Text Input**: Type text into focused fields
- **Navigation**: Back, Home, open apps
- **Screen Reading**: Extract all visible text from the screen

**Required**: Enable `AgentAccessibilityService` in Android Accessibility Settings.

#### 4. Notification Monitoring
Monitors notifications from 6 social applications:
- WeChat (`com.tencent.mm`)
- QQ (`com.tencent.mobileqq`)
- Weibo (`com.sina.weibo`)
- DingTalk (`com.alibaba.android.rimet`)
- WhatsApp (`com.whatsapp`)
- Telegram (`org.telegram.messenger`)

**Features**:
- Per-app enable/disable toggles
- Contact-specific monitoring (comma-separated list)
- Sender extraction with app-specific parsing
- Broadcasts notifications to agent for processing

**Required**: Grant Notification Access permission to the app.

#### 5. Floating Agent Overlay
Always-on-top status indicator:
- Shows current agent state (idle/running/waiting)
- Displays recent notification previews
- Compact pill-shaped design
- Tap to open full UI
- Non-focusable (passes through touches)

**Required**: Enable floating overlay in Settings.

#### 6. Telegram Bot Integration
Remote control via Telegram:
- **Commands**:
  - `/start` - Check if bot is running
  - `/status` - Get current agent status
  - `/stop` - Stop running task
  - `/help` - Show available commands
- **Any other text** - Executed as a task by the agent

**Setup**: Add bot token and start service from Settings > Telegram Bot.

#### 7. Message Queue
Thread-safe notification aggregation:
- Max 20 messages retained
- Observer pattern for real-time updates
- Process messages individually or in batch
- Clear all messages with one tap

**Integration**: Notifications from monitored apps are added to the queue and can be processed by the agent.

### Safety Features

#### Policy Enforcer
Gates every skill invocation before execution:
- **Tier Requirements**: Critical skills require Critical complexity tier
- **Confirmation Rules**: Sensitive actions require user confirmation
- **Skill Blocking**: Denies unsafe skill combinations

#### Permission Guard
Controls AccessibilityService access:
- Validates package names before interaction
- Prevents actions on system apps
- Rate limits gesture injection

#### Memory System
- **Holographic Memory**: Persists user facts across sessions
- **Fact Store**: SQLite-based fact storage with trust scoring
- **Entity Extraction**: Automatic entity recognition from text
- **Deduplication**: Prevents duplicate facts

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

### NO Budget Tracking
Per project requirements, **we do NOT implement any budget tracking, token usage monitoring, or cost enforcement**. The app displays API costs per model for transparency only. Users are responsible for monitoring their own API usage with their providers.

---

## Repository Structure

- `src/`: Rust core implementation logic.
  - `lib.rs`: Main library exports
  - `agent_loop.rs`: Agent state machine
  - `unified_agent.rs`: Agent orchestrator
  - `provider/`: Multi-provider LLM backends
  - `safety/`: Policy enforcement and permission guards
  - `memory/`: Holographic memory system
  - `complexity_classifier.rs`: Task complexity detection
  - `model_pricing.rs`: API cost display data
  - `jni_exports.rs`: JNI bridge to Kotlin

- `app/`: Android Studio project (Kotlin/Compose).
  - `AgentAccessibilityService.kt`: Screen reading and gesture injection
  - `NotificationMonitorService.kt`: Notification listening and parsing
  - `FloatingAgentOverlay.kt`: Status overlay
  - `TelegramBotService.kt`: Remote control via Telegram
  - `AgentViewModel.kt`: Main ViewModel
  - `MessageQueue.kt`: Notification aggregation
  - `HomeScreen.kt`, `ModelsScreen.kt`, `SkillsScreen.kt`, etc.: UI screens

- `jniLibs/`: Pre-compiled Rust binaries for Android.
- `android-ai-agent-migration/`: Project SOUL, implementation plans, and migration context.

---

## Documentation

- `BUILD.md`: Detailed build instructions and dependency mapping.
- `TEST_MATRIX.md`: Coverage and verification status for different modules.
- `android-ai-agent-migration/`: Contains implementation plans and persona context for the "Hermes" agent.

---

## Recent Changes

### Batch 1 (2026-05-07)
- ✅ Complexity Classifier with model suggestions
- ✅ Model Pricing Display for API cost transparency
- ✅ AccessibilityService for screen control
- ✅ Telegram Bot Service for remote control
- ✅ JNI Bridge cleanup (removed budget functions)

### Batch 2 (2026-05-07)
- ✅ NotificationMonitorService with per-app sender extraction
- ✅ FloatingAgentOverlay with status display
- ✅ MessageQueue with observer pattern

### Bug Fixes
- ✅ Fixed duplicate static definitions in `fact_index.rs`

---

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## License

This project is licensed under the GPL-3.0 License - see the [LICENSE](LICENSE) file for details.

---

## Acknowledgments

- Inspired by **KimiClaw** (notification monitoring patterns)
- Borrowed safety patterns from **Swarm**
- Built on **OpenClaw** foundation (accessibility-based automation)
