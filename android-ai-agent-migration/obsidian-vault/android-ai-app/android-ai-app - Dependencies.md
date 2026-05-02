# Android AI App — Dependency Manifest (May 2026)

> All versions verified via live web search on May 1, 2026. Sources: Maven Central, crates.io, developer.android.com, GitHub releases, official docs.

---

## Android / Kotlin Dependencies

### Jetpack Compose — BOM `2026.04.01`

| Artifact | Latest Stable | Release Date | Notes |
|---|---|---|---|
| `compose.animation` | **1.11.0** | Apr 22, 2026 | 1.12.0-alpha01 available |
| `compose.compiler` | **1.5.15** | Aug 7, 2024 | Separate versioning track |
| `compose.foundation` | **1.11.0** | Apr 22, 2026 | |
| `compose.material` | **1.11.0** | Apr 22, 2026 | |
| `compose.material3` | **1.4.0** | Apr 22, 2026 | 1.5.0-alpha18 available |
| `compose.runtime` | **1.11.0** | Apr 22, 2026 | |
| `compose.ui` | **1.11.0** | Apr 22, 2026 | |

```
// build.gradle.kts
val composeBom = platform("androidx.compose:compose-bom:2026.04.01")
implementation(composeBom)
implementation("androidx.compose.ui:ui")
implementation("androidx.compose.material3:material3")
implementation("androidx.compose.ui:ui-tooling-preview")
debugImplementation("androidx.compose.ui:ui-tooling")
```

### Kotlin Coroutines

| Artifact | Version | Status |
|---|---|---|
| `kotlinx-coroutines-core` | **1.10.2** | Stable |
| `kotlinx-coroutines-android` | **1.10.2** | Stable |
| 1.11.0-rc02 | Apr 27, 2026 | Release candidate |

```
implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.10.2")
implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.10.2")
```

### AndroidX Room

| Artifact | Version | Status |
|---|---|---|
| `androidx.room:room-runtime` | **2.8.4** | Stable (Nov 2025) |
| `androidx.room:room-ktx` | **2.8.4** | Stable |
| `androidx.room:room-compiler` | **2.8.4** | KSP processor |
| `androidx.room3:room3-*` | 3.0.0-alpha01 | KMP rewrite (Mar 2026) |

```
implementation("androidx.room:room-runtime:2.8.4")
implementation("androidx.room:room-ktx:2.8.4")
ksp("androidx.room:room-compiler:2.8.4")
```

### OkHttp

| Artifact | Version | Status |
|---|---|---|
| `com.squareup.okhttp3:okhttp` | **5.3.2** | Stable (Nov 2025) |
| `com.squareup.okhttp3:logging-interceptor` | **5.3.2** | |

```
implementation(platform("com.squareup.okhttp3:okhttp-bom:5.3.0"))
implementation("com.squareup.okhttp3:okhttp")
implementation("com.squareup.okhttp3:logging-interceptor")
```

### AndroidX Camera (CameraX)

| Artifact | Version | Status |
|---|---|---|
| `camera-camera2` | **1.6.0** | Stable (Mar 25, 2026) |
| `camera-core` | **1.6.0** | Stable |
| `camera-lifecycle` | **1.6.0** | Stable |
| `camera-view` | **1.6.0** | Stable |
| `camera-video` | **1.6.0** | Stable |

```
implementation("androidx.camera:camera-camera2:1.6.0")
implementation("androidx.camera:camera-lifecycle:1.6.0")
implementation("androidx.camera:camera-view:1.6.0")
```

### Other AndroidX

| Artifact | Version | Used For |
|---|---|---|
| `activity-ktx` | **1.13.0** | Activity + Compose integration |
| `core-ktx` | **1.18.0** | Core extensions |
| `lifecycle-runtime-ktx` | **2.10.0** | Lifecycle-aware coroutines |
| `navigation-compose` | **2.9.7** | Navigation in Compose |
| `datastore-preferences` | **1.2.1** | Key-value storage |

---

## Rust Dependencies (Cargo.toml)

### Core Framework

| Crate | Version | MSRV | Released | Notes |
|---|---|---|---|---|
| **tokio** | `1.52.1` | 1.71 | Apr 16, 2026 | Async runtime; features: `full` |
| **reqwest** | `0.13.3` | 1.64 | Apr 27, 2026 | HTTP client; features: `json`, `rustls` |

### Serialization

| Crate | Version | MSRV | Released |
|---|---|---|---|
| **serde** | `1.0.228` | 1.56 | Sep 27, 2025 |
| **serde_json** | `1.0.149` | 1.68 | Jan 6, 2026 |
| **toml** | `1.1.2` | 1.85 | Apr 1, 2026 |

### JNI Bridge

| Crate | Version | MSRV | Released |
|---|---|---|---|
| **jni** | `0.22.4` | 1.85.0 | Mar 16, 2026 |

```toml
[dependencies]
tokio = { version = "1.52", features = ["full"] }
reqwest = { version = "0.13", features = ["json", "rustls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "1.1"
jni = "0.22"

[lib]
crate-type = ["cdylib"]
```

---

## External Services / APIs

### OpenRouter API

| Detail | Value |
|---|---|
| Base URL | `https://openrouter.ai/api/v1` |
| Auth | `Bearer <API_KEY>` |
| Platform fee | 5.5% on credit purchase (no inference markup) |
| Pay-as-you-go | No minimum, credit card / crypto / bank transfer |
| Free tier | 25+ free models, 50 req/day, no credit card |
| Rate limits (paid) | No platform-level limits (provider limits may apply) |
| Prompt caching | Supported on Anthropic + OpenRouter select models |

Key model pricing (pass-through, matches direct provider pricing):

| Model | Input ($/1M tokens) | Output ($/1M tokens) |
|---|---|---|
| Claude Opus 4.6 | $5.00 | $25.00 |
| Claude Sonnet 4.5 | $3.00 | $15.00 |
| GPT-5 | $1.25 | $10.00 |
| Gemini Flash 2.5 | ~$0.075 | ~$0.30 |
| DeepSeek Chat | $0.32 | $0.89 |

### Self-Hosted Search — Runs on the Android Phone (replaces Brave Search)

The app runs its own search engine **on the phone itself** — zero external cost, no API keys, no cloud dependency. The Rust-based search binary is cross-compiled for Android ARM64 (aarch64) and runs as a local companion process alongside the agent's `libagent.so` core. See [[Self-Hosted Search Engines]] for the full comparison.

**Primary: Websurfx** (Rust, Actix-Web) — runs on-device

| Detail | Value |
|---|---|
| Language | Rust — cross-compile for Android ARM64 with `cargo build --target aarch64-linux-android` |
| Type | Meta-search engine (aggregates Google, Bing, DuckDuckGo, etc.) |
| Footprint | Single Rust binary, ~15-25MB, runs as local HTTP server on `127.0.0.1:8080` |
| Config | Lua-based, stored in `~/.agent/websurfx/` config dir |
| Integration | `GET http://127.0.0.1:8080/search?q={query}` — called by the Rust core's `web_search` skill |
| Startup | Launched by the agent as a child process via shell_cmd or bundled as a native binary |
| Repo | https://github.com/neon-mmd/websurfx |

**Fallback: Direct HTTP queries** (no proxy needed)

If a locally-running meta-search binary is too heavy, the `web_search` skill can query search engines directly:
- DuckDuckGo HTML: `https://html.duckduckgo.com/html/?q={query}` (no JS, lightweight)
- The Rust `http_client` module already handles multi-provider HTTP — no separate binary needed
- Parse results directly in the agent's `tool_parser` — no external process

**Local RAG search (on-device indexing):**

| Engine | Language | Footprint | Use Case |
|---|---|---|---|
| **Meilisearch** | Rust | Single binary, ~30MB | Full-text search over local docs/notes |
| **tinysearch** | Rust+WASM | ~50-100kB gzipped | Embedded static search for agent memory/docs |

### Other External Services (Android built-in, no Maven dep)

| Component | Package/API | Notes |
|---|---|---|
| SpeechRecognizer | `android.speech.SpeechRecognizer` | On-device STT |
| TextToSpeech | `android.speech.tts.TextToSpeech` | On-device TTS |
| AccessibilityService | `android.accessibilityservice.AccessibilityService` | Screen reading + gestures |
| NotificationListenerService | `android.service.notification.NotificationListenerService` | Read notifications |
| MediaProjection | `android.media.projection.MediaProjection` | Screenshot capture |
| ClipboardManager | `android.content.ClipboardManager` | Clipboard read/write |
| AlarmManager | `android.app.AlarmManager` | Alarms/timers |
| ContentProvider (Calendar/Contacts) | `android.content.ContentProvider` | Calendar + contacts CRUD |

### Optional / External Tools

| Tool | Purpose | Setup |
|---|---|---|
| **Termux** | Shell command execution (IPC via Intent) | User must install Termux separately |
| **Porcupine/Rhino** | Wake word detection (ONNX) | Embedded; Picovoice license |
| **llama.cpp** | Local LLM inference | Side-loaded, exposed via `http://127.0.0.1:8080/v1` |

---

## Rust Crate Docs (Key API Reference)

### reqwest v0.13 — HTTP Client

Making requests with the async `Client`:

```rust
let client = reqwest::Client::new();

// GET
let body = client.get("https://api.example.com/data")
    .send().await?.text().await?;

// POST JSON
let mut map = HashMap::new();
map.insert("key", "value");
let res = client.post("https://api.example.com/submit")
    .json(&map).send().await?;

// POST form
let params = [("foo", "bar")];
let res = client.post("https://httpbin.org/post")
    .form(&params).send().await?;
```

Features: `json`, `rustls` (TLS), `cookies`, `gzip`, `brotli`, `socks` (SOCKS5 proxy).  
Redirects: automatic, max 10 hops. Proxies: system proxy env vars enabled by default.

### tokio v1.52 — Async Runtime

```rust
#[tokio::main]
async fn main() {
    // Spawn concurrent tasks
    let handle = tokio::spawn(async {
        "done"
    });
    let result = handle.await.unwrap();

    // Blocking work
    let result = tokio::task::spawn_blocking(|| {
        heavy_computation()
    }).await.unwrap();

    // TCP echo server
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            let n = socket.read(&mut buf).await?;
            socket.write_all(&buf[0..n]).await?;
        });
    }
}
```

Key modules: `tokio::sync` (channels, Mutex), `tokio::time` (sleep, timeout, interval), `tokio::net` (TCP/UDP).

### jni v0.22 — Java Native Interface

Two approaches: export a mangled symbol, or register at runtime.

**Export approach** — for Android native methods in `libagent.so`:

```rust
use jni::EnvUnowned;
use jni::objects::{JClass, JString};

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_yourdomain_agent_RustBridge_startAgent<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    class: JClass<'caller>,
    prompt: JString<'caller>,
) -> JString<'caller> {
    let outcome = unowned_env.with_env(|env| -> Result<_, jni::errors::Error> {
        let input: String = prompt.to_string();
        let result = agent_process(&input);  // your Rust logic
        JString::from_str(env, &result)
    });
    outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}
```

Cargo.toml: `crate-type = ["cdylib"]` is required.  
MSRV: 1.85.0. Docs: https://docs.rs/jni

### serde v1.0 + serde_json v1.0

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct AgentRequest {
    prompt: String,
    model: String,
    max_tokens: u32,
}

let request = AgentRequest {
    prompt: "search for flights".into(),
    model: "claude-sonnet-4-6".into(),
    max_tokens: 4096,
};

let json = serde_json::to_string(&request).unwrap();
let parsed: AgentRequest = serde_json::from_str(&json).unwrap();
```

### toml v1.1 — TOML Parser

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct SkillConfig {
    skill: SkillMeta,
    tool: Option<ToolDef>,
    implementation: ImplConfig,
}

let config: SkillConfig = toml::from_str(&skill_toml)?;
```

Also supports `toml::Table` for dynamic access: `let t = "key = 'val'".parse::<Table>().unwrap();`

---

## Quick Reference — All Pinned Versions

### Android (`build.gradle.kts`)

```kotlin
val composeBom = "2026.04.01"
val coroutines = "1.10.2"
val room = "2.8.4"
val okhttp = "5.3.2"
val camerax = "1.6.0"
val activityKtx = "1.13.0"
val coreKtx = "1.18.0"
val lifecycle = "2.10.0"
val navigation = "2.9.7"
```

### Rust (`Cargo.toml`)

```toml
tokio = "1.52"
reqwest = "0.13"
serde = "1.0"
serde_json = "1.0"
toml = "1.1"
jni = "0.22"
```

### External APIs

| Service | Endpoint | Auth | Cost |
|---|---|---|---|
| OpenRouter | `https://openrouter.ai/api/v1` | Bearer token | 5.5% platform fee |
| Websurfx (self-hosted) | `http://127.0.0.1:8080/search` | None | Zero |

---

*All versions verified May 1, 2026. Check official sources before production use as versions may have updated since.*
