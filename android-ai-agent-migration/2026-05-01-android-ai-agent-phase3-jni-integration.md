# Android AI Agent — Phase 3: JNI Bridge + Android Integration Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Build the JNI bridge layer connecting the Rust core to Kotlin, implement the memory manager for MEMORY.md persistence, and define the Kotlin-side specifications for ViewModel, AccessibilityService, and KeystoreManager — all ready to drop into Android Studio.

**Architecture:** The Rust library compiles as a `cdylib` targeting `aarch64-linux-android` via `cargo-ndk`. JNI exports at `jni_exports.rs` expose a flat C API. Kotlin calls these via a `RustBridge` class. UniFFI is skipped in favor of hand-written JNI (we already have the `jni` crate, simpler for our use case, avoids UniFFI's Android async instability).

**Research Summary (May 2026):**
- **cargo-ndk v4.1.2**: `cargo ndk -t arm64-v8a -o ./jniLibs build --release`
- **jni crate 0.22**: Already in Cargo.toml. Uses `unsafe(no_mangle) extern "C"` for exported symbols
- **AccessibilityService**: `canPerformGestures=true`, `dispatchGesture()`, `GestureDescription`
- **Android Keystore**: `KeyStore.getInstance("AndroidKeyStore")`, hardware-backed AES/GCM
- **UniFFI**: Recommended by Mozilla for Kotlin+Rust, but has Android async stability issues — skipped

**Tech Stack:** Rust 1.85+, jni 0.22, cargo-ndk (for cross-compile), Kotlin + Jetpack Compose (Android Studio project).

---

### Task 1: Update Cargo.toml for Android cdylib target

**Objective:** Configure the crate to build as a shared library for Android.

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add cdylib crate type and Android-specific metadata**

```toml
[lib]
crate-type = ["lib", "cdylib"]
name = "agent_core"

[target.'cfg(target_os = "android")'.dependencies]
jni = { version = "0.22", default-features = false }
```

**Step 2: Create .cargo/config.toml for NDK linker paths**

```toml
# .cargo/config.toml
# Linker paths for Android NDK cross-compilation.
# These paths are examples — replace with actual NDK path on your machine.
# cargo-ndk handles this automatically when used, these are fallbacks.

[target.aarch64-linux-android]
linker = "aarch64-linux-android30-clang"

[target.armv7-linux-androideabi]
linker = "armv7a-linux-androideabi30-clang"

[target.x86_64-linux-android]
linker = "x86_64-linux-android30-clang"
```

**Step 3: Verify**
```bash
cargo check  # Must still work for host target
```

**Step 4: Commit**
```bash
git add Cargo.toml .cargo/
git commit -m "chore: add Android cdylib target and NDK linker config"
```

---

### Task 2: Implement JNI exports module (jni_exports.rs)

**Objective:** Create the flat C API that Kotlin calls via JNI. Each function follows the JNI naming convention `Java_com_yourdomain_agent_RustBridge_<methodName>`.

**Files:**
- Create: `src/jni_exports.rs`
- Modify: `src/lib.rs` (add module, gate with `#[cfg(target_os = "android")]`)

**Code for `src/jni_exports.rs`:**

```rust
// JNI exports for the Android agent core.
// Each function follows JNI naming: Java_{package}_{Class}_{method}
// These are called from Kotlin's RustBridge class.

#[cfg(target_os = "android")]
pub mod android {
    use jni::objects::{JClass, JString};
    use jni::sys::jstring;
    use jni::JNIEnv;

    /// Initialize the agent with API keys and return a session token.
    /// Kotlin: RustBridge.nativeInit(env, class, openRouterKey, anthropicKey)
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_RustBridge_nativeInit(
        mut env: JNIEnv,
        _class: JClass,
        openrouter_key: JString,
    ) -> jstring {
        let key: String = env.get_string(&openrouter_key)
            .map(|s| s.into())
            .unwrap_or_default();

        let result = format!("init_ok:{}", &key[..4.min(key.len())]);
        let output = env.new_string(result).expect("failed to create string");
        output.into_raw()
    }

    /// Run the agent loop with a user prompt. Returns the response.
    /// Kotlin: RustBridge.nativeRun(env, class, prompt)
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_RustBridge_nativeRun(
        mut env: JNIEnv,
        _class: JClass,
        prompt: JString,
    ) -> jstring {
        let input: String = env.get_string(&prompt)
            .map(|s| s.into())
            .unwrap_or_default();

        // In production: this runs the full agent_loop
        // For now: echo back with classification
        let complexity = crate::complexity_classifier::classify(&input);
        let result = format!("[{:?}] Processing: {}", complexity, input);
        let output = env.new_string(result).expect("failed to create string");
        output.into_raw()
    }

    /// Get agent status: "idle", "running", "waiting_confirmation"
    /// Kotlin: RustBridge.nativeStatus(env, class)
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_RustBridge_nativeStatus(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jstring {
        let output = env.new_string("idle").expect("failed to create string");
        output.into_raw()
    }

    /// Get recent log lines from the ring buffer
    /// Kotlin: RustBridge.nativeGetLogs(env, class, count)
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_RustBridge_nativeGetLogs(
        mut env: JNIEnv,
        _class: JClass,
        count: jni::sys::jint,
    ) -> jstring {
        let result = format!("[log] last {} entries", count);
        let output = env.new_string(result).expect("failed to create string");
        output.into_raw()
    }

    /// Confirm or reject a pending confirmation-required action
    /// Kotlin: RustBridge.nativeConfirm(env, class, approved)
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_RustBridge_nativeConfirm(
        mut env: JNIEnv,
        _class: JClass,
        approved: jni::sys::jboolean,
    ) -> jstring {
        let msg = if approved != 0 { "confirmed" } else { "rejected" };
        let output = env.new_string(msg).expect("failed to create string");
        output.into_raw()
    }
}

// Allow dead code on non-Android targets (these functions only get called from JVM)
#[cfg(not(target_os = "android"))]
pub mod android {
    // Stub module for host compilation
}
```

**Step 2: Update `src/lib.rs`**

```rust
pub mod jni_exports;
```

**Step 3: Verify compilation for host target**
```bash
cargo check
```

**Step 4: Commit**
```bash
git add -A && git commit -m "feat: add JNI exports module for Kotlin→Rust bridge"
```

---

### Task 3: Implement Memory Manager (MEMORY.md read/write/update)

**Objective:** Build the memory manager that persists user facts across sessions. Reads and writes `~/.agent/MEMORY.md` using the markdown structure from the spec.

**Files:**
- Create: `src/memory_manager.rs`
- Modify: `src/lib.rs` (add module)

**Code for `src/memory_manager.rs`:**

```rust
use std::path::PathBuf;

/// The default memory file path: ~/.agent/MEMORY.md
fn default_memory_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("EXTERNAL_STORAGE"))
        .unwrap_or_else(|_| "/data/data/com.yourdomain.agent".to_string());
    PathBuf::from(home).join(".agent").join("MEMORY.md")
}

pub struct MemoryManager {
    path: PathBuf,
}

impl MemoryManager {
    pub fn new() -> Self {
        let path = default_memory_path();
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        Self { path }
    }

    pub fn with_path(path: PathBuf) -> Self {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        Self { path }
    }

    /// Read the full memory file
    pub fn read(&self) -> String {
        std::fs::read_to_string(&self.path).unwrap_or_default()
    }

    /// Write (overwrite) the memory file
    pub fn write(&self, content: &str) -> std::io::Result<()> {
        std::fs::write(&self.path, content)
    }

    /// Append a fact to the "Persistent Facts" section
    pub fn add_fact(&self, fact: &str) -> std::io::Result<()> {
        let mut content = self.read();
        let section = "## Persistent Facts";
        let new_entry = format!("\n- {}", fact);

        if let Some(pos) = content.find(section) {
            // Insert after the section header, before the next section
            let after_header = &content[pos + section.len()..];
            if let Some(next_section) = after_header.find("\n## ") {
                let insert_pos = pos + section.len() + next_section;
                content.insert_str(insert_pos, &new_entry);
            } else {
                content.push_str(&new_entry);
            }
        } else {
            // No Persistent Facts section yet — append one
            content.push_str(&format!("\n\n{}\n{}", section, new_entry));
        }

        self.write(&content)
    }

    /// Update the "Recent Context" section with the last completed task
    pub fn update_last_task(&self, task: &str) -> std::io::Result<()> {
        let mut content = self.read();
        let marker = "## Recent Context";
        let task_line = format!("\n- Last task: \"{}\" ({})", task, chrono::Local::now().format("%Y-%m-%d"));

        if let Some(pos) = content.find(marker) {
            // Replace or append after the section
            let after = &content[pos + marker.len()..];
            if let Some(existing) = after.find("- Last task:") {
                let start = pos + marker.len() + existing;
                let end = after[existing..].find('\n').map(|e| start + e).unwrap_or(content.len());
                content.replace_range(start..end, &task_line);
            } else {
                let insert_pos = pos + marker.len();
                content.insert_str(insert_pos, &task_line);
            }
        } else {
            content.push_str(&format!("\n\n{}{}", marker, task_line));
        }

        self.write(&content)
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_read_write() {
        let dir = std::env::temp_dir().join("agent_mem_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("MEMORY.md");
        let mgr = MemoryManager::with_path(path.clone());

        mgr.write("# Test Memory\n\n## Persistent Facts\n- fact 1").unwrap();
        let content = mgr.read();
        assert!(content.contains("fact 1"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_add_fact() {
        let dir = std::env::temp_dir().join("agent_mem_test2");
        let path = dir.join("MEMORY.md");
        let mgr = MemoryManager::with_path(path.clone());

        mgr.write("## Persistent Facts\n- existing fact").unwrap();
        mgr.add_fact("new fact").unwrap();
        let content = mgr.read();
        assert!(content.contains("existing fact"));
        assert!(content.contains("new fact"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_empty_memory() {
        let dir = std::env::temp_dir().join("agent_mem_test3");
        let path = dir.join("MEMORY.md");
        let mgr = MemoryManager::with_path(path);
        let content = mgr.read();
        assert!(content.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }
}
```

Note: `chrono` is used for timestamps. Add to Cargo.toml: `chrono = "0.4"` if we want timestamps, otherwise use a simpler approach.

**Step 2: Update `src/lib.rs`** — add `pub mod memory_manager;`

**Step 3: Run tests**
```bash
cargo test memory_manager
```
Expected: 3 tests pass.

**Step 4: Commit**
```bash
git add -A && git commit -m "feat: add memory manager for MEMORY.md persistence"
```

---

### Task 4: Create Kotlin RustBridge class (spec, for Android Studio)

**Objective:** Define the Kotlin class that calls into the Rust JNI exports. This is a spec file — it documents the exact JNI contract. Drop it into `app/src/main/kotlin/com/yourdomain/agent/bridge/RustBridge.kt` when the Android project is created.

**Files:**
- Create: `kotlin/RustBridge.kt` (spec file in the Rust repo for reference)

**Code:**

```kotlin
package com.yourdomain.agent.bridge

/**
 * Bridge to the native Rust agent core (libagent_core.so).
 *
 * Each method maps to a JNI export in src/jni_exports.rs.
 * JNI naming convention:
 *   Java_com_yourdomain_agent_bridge_RustBridge_<methodName>
 *
 * Build: cargo ndk -t arm64-v8a -o ./app/src/main/jniLibs build --release
 */
object RustBridge {
    init {
        System.loadLibrary("agent_core")
    }

    /** Initialize the agent with API keys. Returns session token. */
    external fun nativeInit(openrouterKey: String): String

    /** Run the agent loop. Returns the response. */
    external fun nativeRun(prompt: String): String

    /** Get current agent status: "idle", "running", "waiting_confirmation" */
    external fun nativeStatus(): String

    /** Get recent log entries from the ring buffer */
    external fun nativeGetLogs(count: Int): String

    /** Confirm or reject a pending action */
    external fun nativeConfirm(approved: Boolean): String
}
```

**Step 2: Commit**
```bash
git add kotlin/ && git commit -m "feat: add Kotlin RustBridge spec with JNI contract"
```

---

### Task 5: Create AgentViewModel spec (for Android Studio)

**Objective:** Define the Kotlin ViewModel that wires the Rust bridge to the Compose UI. Manages coroutine-based async dispatch, log relay, and state.

**Files:**
- Create: `kotlin/AgentViewModel.kt`

**Code:**

```kotlin
package com.yourdomain.agent.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import com.yourdomain.agent.bridge.RustBridge

data class AgentState(
    val status: String = "idle",       // idle | running | waiting_confirmation
    val currentTask: String = "",
    val logLines: List<String> = emptyList(),
    val activeModel: String = "",
    val pendingConfirmation: String? = null,
)

class AgentViewModel : ViewModel() {

    private val _state = MutableStateFlow(AgentState())
    val state: StateFlow<AgentState> = _state

    fun initAgent(openrouterKey: String) {
        viewModelScope.launch {
            withContext(Dispatchers.IO) {
                val result = RustBridge.nativeInit(openrouterKey)
                _state.value = _state.value.copy(status = "idle")
            }
        }
    }

    fun startTask(prompt: String) {
        _state.value = _state.value.copy(status = "running", currentTask = prompt)
        viewModelScope.launch {
            withContext(Dispatchers.IO) {
                val result = RustBridge.nativeRun(prompt)
                _state.value = _state.value.copy(
                    status = "idle",
                    logLines = _state.value.logLines + result,
                )
            }
        }
    }

    fun stopTask() {
        _state.value = _state.value.copy(status = "idle")
    }

    fun refreshStatus() {
        viewModelScope.launch {
            withContext(Dispatchers.IO) {
                val status = RustBridge.nativeStatus()
                _state.value = _state.value.copy(status = status)
            }
        }
    }

    fun refreshLogs(count: Int = 20) {
        viewModelScope.launch {
            withContext(Dispatchers.IO) {
                val logs = RustBridge.nativeGetLogs(count)
                _state.value = _state.value.copy(
                    logLines = logs.split("\n").filter { it.isNotBlank() }
                )
            }
        }
    }

    fun confirmAction(approved: Boolean) {
        viewModelScope.launch {
            withContext(Dispatchers.IO) {
                RustBridge.nativeConfirm(approved)
                _state.value = _state.value.copy(
                    pendingConfirmation = null,
                    status = "running",
                )
            }
        }
    }
}
```

**Step 2: Commit**
```bash
git add kotlin/ && git commit -m "feat: add AgentViewModel spec with coroutine-based Rust bridge wiring"
```

---

### Task 6: Create AccessibilityService spec (for Android Studio)

**Objective:** Define the Android AccessibilityService that reads screen content and performs gestures. This is the phone-control layer.

**Files:**
- Create: `kotlin/AgentAccessibilityService.kt`
- Create: `kotlin/res/xml/accessibility_service_config.xml`

**Code for `kotlin/AgentAccessibilityService.kt`:**

```kotlin
package com.yourdomain.agent.service

import android.accessibilityservice.AccessibilityService
import android.accessibilityservice.GestureDescription
import android.graphics.Path
import android.util.DisplayMetrics
import android.view.accessibility.AccessibilityEvent
import android.view.accessibility.AccessibilityNodeInfo

class AgentAccessibilityService : AccessibilityService() {

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {
        // Events are processed by the agent loop via the Rust bridge
    }

    override fun onInterrupt() {}

    override fun onServiceConnected() {
        super.onServiceConnected()
        // Service ready — agent can now read screens and perform gestures
    }

    /** Get the current screen's UI tree as a simplified string */
    fun getScreenContent(): String {
        val root = rootInActiveWindow ?: return ""
        return nodeToString(root, 0)
    }

    /** Perform a tap at screen coordinates */
    fun tap(x: Float, y: Float, callback: (Boolean) -> Unit = {}) {
        val path = Path().apply { moveTo(x, y) }
        val stroke = GestureDescription.StrokeDescription(path, 0, 100)
        val gesture = GestureDescription.Builder()
            .addStroke(stroke)
            .build()
        dispatchGesture(gesture, object : GestureResultCallback() {
            override fun onCompleted(gestureDescription: GestureDescription?) {
                callback(true)
            }
            override fun onCancelled(gestureDescription: GestureDescription?) {
                callback(false)
            }
        }, null)
    }

    /** Perform a swipe between two points */
    fun swipe(
        fromX: Float, fromY: Float,
        toX: Float, toY: Float,
        duration: Long = 300,
        callback: (Boolean) -> Unit = {}
    ) {
        val path = Path().apply {
            moveTo(fromX, fromY)
            lineTo(toX, toY)
        }
        val stroke = GestureDescription.StrokeDescription(path, 0, duration)
        val gesture = GestureDescription.Builder()
            .addStroke(stroke)
            .build()
        dispatchGesture(gesture, object : GestureResultCallback() {
            override fun onCompleted(gestureDescription: GestureDescription?) {
                callback(true)
            }
            override fun onCancelled(gestureDescription: GestureDescription?) {
                callback(false)
            }
        }, null)
    }

    /** Type text into the currently focused field */
    fun typeText(text: String) {
        val focused = findFocus(AccessibilityNodeInfo.FOCUS_INPUT) ?: return
        val args = android.os.Bundle().apply {
            putCharSequence(
                AccessibilityNodeInfo.ACTION_ARGUMENT_SET_TEXT_CHARSEQUENCE,
                text
            )
        }
        focused.performAction(AccessibilityNodeInfo.ACTION_SET_TEXT, args)
    }

    /** Launch an app by package name */
    fun openApp(packageName: String) {
        val intent = packageManager.getLaunchIntentForPackage(packageName)
        if (intent != null) {
            intent.addFlags(android.content.Intent.FLAG_ACTIVITY_NEW_TASK)
            startActivity(intent)
        }
    }

    /** Get screen dimensions */
    fun getScreenSize(): Pair<Int, Int> {
        val metrics = DisplayMetrics()
        val display = getSystemService(android.content.Context.WINDOW_SERVICE)
            as? android.view.WindowManager
        display?.defaultDisplay?.getRealMetrics(metrics)
        return Pair(metrics.widthPixels, metrics.heightPixels)
    }

    private fun nodeToString(node: AccessibilityNodeInfo, depth: Int): String {
        val indent = "  ".repeat(depth)
        val sb = StringBuilder()
        val className = node.className?.toString()?.substringAfterLast(".") ?: "?"
        val text = node.text?.toString() ?: ""
        val contentDesc = node.contentDescription?.toString() ?: ""
        val clickable = if (node.isClickable) " [TAP]" else ""
        val id = node.viewIdResourceName ?: ""

        sb.appendLine("$indent$className$clickable: $text $contentDesc ($id)")
        for (i in 0 until node.childCount) {
            val child = node.getChild(i) ?: continue
            sb.append(nodeToString(child, depth + 1))
            child.recycle()
        }
        return sb.toString()
    }
}
```

**Code for `kotlin/res/xml/accessibility_service_config.xml`:**

```xml
<?xml version="1.0" encoding="utf-8"?>
<accessibility-service
    xmlns:android="http://schemas.android.com/apk/res/android"
    android:description="@string/accessibility_service_description"
    android:accessibilityEventTypes="typeAllMask"
    android:accessibilityFlags="flagDefault|flagReportViewIds"
    android:canRetrieveWindowContent="true"
    android:canPerformGestures="true"
    android:notificationTimeout="100" />
```

**Step 2: Commit**
```bash
git add kotlin/ && git commit -m "feat: add AccessibilityService spec with screen read, tap, swipe, type, openApp"
```

---

### Task 7: Create KeystoreManager spec (for Android Studio)

**Objective:** Define the Kotlin class that securely stores API keys in Android Keystore (hardware-backed AES/GCM).

**Files:**
- Create: `kotlin/KeystoreManager.kt`

**Code:**

```kotlin
package com.yourdomain.agent.data

import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/**
 * Hardware-backed encrypted storage for API keys.
 * Uses Android Keystore with AES-256/GCM.
 * Keys never leave secure hardware on supported devices.
 */
object KeystoreManager {
    private const val KEYSTORE_PROVIDER = "AndroidKeyStore"
    private const val KEY_ALIAS = "agent_api_key"
    private const val TRANSFORMATION = "AES/GCM/NoPadding"
    private const val IV_SIZE = 12

    private fun getOrCreateKey(): SecretKey {
        val ks = KeyStore.getInstance(KEYSTORE_PROVIDER).apply { load(null) }
        ks.getKey(KEY_ALIAS, null)?.let { return it as SecretKey }

        val keyGenerator = KeyGenerator.getInstance(
            KeyProperties.KEY_ALGORITHM_AES,
            KEYSTORE_PROVIDER
        )
        val spec = KeyGenParameterSpec.Builder(
            KEY_ALIAS,
            KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT
        )
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .setKeySize(256)
            .setIsStrongBoxBacked(true)  // Use StrongBox if available
            .build()

        keyGenerator.init(spec)
        return keyGenerator.generateKey()
    }

    fun saveApiKey(key: String, value: String) {
        val prefs = android.preference.PreferenceManager
            .getDefaultSharedPreferences(
                // Context would be injected
                throw UnsupportedOperationException("Inject Context")
            )
        val encrypted = encrypt(value)
        prefs.edit().putString("key_$key", encrypted).apply()
    }

    fun getApiKey(key: String): String? {
        val prefs = android.preference.PreferenceManager
            .getDefaultSharedPreferences(
                throw UnsupportedOperationException("Inject Context")
            )
        val encrypted = prefs.getString("key_$key", null) ?: return null
        return decrypt(encrypted)
    }

    private fun encrypt(plainText: String): String {
        val key = getOrCreateKey()
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(Cipher.ENCRYPT_MODE, key)
        val iv = cipher.iv
        val encrypted = cipher.doFinal(plainText.toByteArray(Charsets.UTF_8))
        val combined = iv + encrypted
        return Base64.encodeToString(combined, Base64.NO_WRAP)
    }

    private fun decrypt(encryptedBase64: String): String? {
        return try {
            val key = getOrCreateKey()
            val combined = Base64.decode(encryptedBase64, Base64.NO_WRAP)
            val iv = combined.copyOfRange(0, IV_SIZE)
            val encrypted = combined.copyOfRange(IV_SIZE, combined.size)
            val cipher = Cipher.getInstance(TRANSFORMATION)
            cipher.init(Cipher.DECRYPT_MODE, key, GCMParameterSpec(128, iv))
            String(cipher.doFinal(encrypted), Charsets.UTF_8)
        } catch (e: Exception) {
            null
        }
    }
}
```

**Step 2: Commit**
```bash
git add kotlin/ && git commit -m "feat: add KeystoreManager spec with hardware-backed AES-256/GCM"
```

---

### Task 8: Update plan with Android Studio project setup instructions

**Objective:** Add a README section or build script documenting how to create the Android Studio project and integrate the Rust library.

**Files:**
- Create: `BUILD.md` (build instructions)

**Step 1: Write `BUILD.md`**

```markdown
# Build Instructions — Android AI Agent

## Prerequisites

- Rust 1.85+
- Android Studio (with NDK r27+)
- cargo-ndk: `cargo install cargo-ndk`

## 1. Cross-compile Rust for Android

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o ./jniLibs build --release
```

Output: `jniLibs/arm64-v8a/libagent_core.so`, etc.

## 2. Create Android Studio Project

1. New Project → Empty Activity → Kotlin + Jetpack Compose
2. Package: `com.yourdomain.agent`
3. Min SDK: 26 (Android 8.0)

## 3. Integrate Rust Library

1. Copy `jniLibs/` into `app/src/main/jniLibs/`
2. Copy `kotlin/RustBridge.kt` → `app/src/main/kotlin/com/yourdomain/agent/bridge/RustBridge.kt`
3. Copy `kotlin/AgentViewModel.kt` → `app/src/main/kotlin/com/yourdomain/agent/ui/AgentViewModel.kt`
4. Copy `kotlin/AgentAccessibilityService.kt` → `app/src/main/kotlin/com/yourdomain/agent/service/AgentAccessibilityService.kt`
5. Copy `kotlin/KeystoreManager.kt` → `app/src/main/kotlin/com/yourdomain/agent/data/KeystoreManager.kt`

## 4. Add Android Manifest Permissions

```xml
<uses-permission android:name="android.permission.BIND_ACCESSIBILITY_SERVICE"/>
<uses-permission android:name="android.permission.INTERNET"/>
<uses-permission android:name="android.permission.FOREGROUND_SERVICE"/>
```

## 5. Build APK

```bash
./gradlew assembleDebug
# Output: app/build/outputs/apk/debug/app-debug.apk
```

## 6. Deploy

```bash
adb install app/build/outputs/apk/debug/app-debug.apk
```
```

**Step 2: Commit**
```bash
git add BUILD.md && git commit -m "docs: add Android build and integration instructions"
```

---

## Completion Checklist

- [ ] `cargo check` — passes (host target)
- [ ] `cargo test` — all tests pass (~30+ tests)
- [ ] 4 Kotlin spec files ready for Android Studio
- [ ] JNI exports match Kotlin RustBridge signatures
- [ ] BUILD.md documents end-to-end integration

## Phase 3 Module Map (after completion)

```
android-ai-agent/
├── Cargo.toml            ← cdylib target + Android deps
├── .cargo/config.toml    ← NDK linker paths
├── BUILD.md              ← Android build instructions
├── src/
│   ├── jni_exports.rs    ← NEW: 5 JNI functions for Kotlin
│   ├── memory_manager.rs ← NEW: MEMORY.md read/write/update
│   ├── agent_loop.rs
│   ├── identity.rs
│   ├── ... (Phase 1-2 modules)
└── kotlin/
    ├── RustBridge.kt              ← JNI contract
    ├── AgentViewModel.kt          ← Compose UI wiring
    ├── AgentAccessibilityService.kt ← Screen read + gestures
    ├── KeystoreManager.kt         ← Hardware-backed key storage
    └── res/xml/
        └── accessibility_service_config.xml
```
