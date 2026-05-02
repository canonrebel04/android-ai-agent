# Build Instructions — Android AI Agent

## Prerequisites

- Rust 1.85+
- Android Studio (with NDK r27+)
- cargo-ndk: `cargo install cargo-ndk`
- Android target toolchains

## 1. Add Android Targets

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
```

## 2. Cross-compile Rust for Android

```bash
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o ./jniLibs build --release
```

Output: `jniLibs/arm64-v8a/libagent_core.so`, etc.

## 3. Create Android Studio Project

1. New Project → Empty Activity → Kotlin + Jetpack Compose
2. Package: `com.yourdomain.agent`
3. Min SDK: 26 (Android 8.0)

## 4. Integrate Rust Library

Copy generated Kotlin specs into the Android project:

| Source (this repo) | Destination (Android project) |
|---|---|
| `jniLibs/` | `app/src/main/jniLibs/` |
| `kotlin/RustBridge.kt` | `app/src/main/kotlin/com/yourdomain/agent/bridge/RustBridge.kt` |
| `kotlin/AgentViewModel.kt` | `app/src/main/kotlin/com/yourdomain/agent/ui/AgentViewModel.kt` |
| `kotlin/AgentAccessibilityService.kt` | `app/src/main/kotlin/com/yourdomain/agent/service/AgentAccessibilityService.kt` |
| `kotlin/KeystoreManager.kt` | `app/src/main/kotlin/com/yourdomain/agent/data/KeystoreManager.kt` |
| `kotlin/res/xml/accessibility_service_config.xml` | `app/src/main/res/xml/accessibility_service_config.xml` |

## 5. Add Dependencies to build.gradle.kts

```kotlin
dependencies {
    implementation("androidx.compose:compose-bom:2026.04.01")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.10.0")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.10.2")
}
```

## 6. Add Android Manifest Permissions

```xml
<uses-permission android:name="android.permission.BIND_ACCESSIBILITY_SERVICE"/>
<uses-permission android:name="android.permission.INTERNET"/>
<uses-permission android:name="android.permission.FOREGROUND_SERVICE"/>
<uses-permission android:name="android.permission.WAKE_LOCK"/>
<uses-permission android:name="android.permission.REQUEST_IGNORE_BATTERY_OPTIMIZATIONS"/>
```

## 7. Build APK

```bash
./gradlew assembleDebug
# Output: app/build/outputs/apk/debug/app-debug.apk
```

## 8. Deploy

```bash
adb install app/build/outputs/apk/debug/app-debug.apk
```

## Running Tests (Rust only)

```bash
cargo test                    # 30+ tests
cargo check                   # Host target
```

## Phases

- **Phase 1** — Rust core (http_client, model_router, skill_registry, etc.)
- **Phase 2** — Safety layer (policy_enforcer, permission_guard, agent_loop, identity)
- **Phase 3** — JNI bridge + Android specs (jni_exports, memory_manager, Kotlin files)
- **Phase 4** — Android Studio integration (Compose UI, AccessibilityService, Telegram bot)
