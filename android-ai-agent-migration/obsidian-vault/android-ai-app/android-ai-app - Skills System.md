# Android AI App — Skills System

## Plugin Architecture

Each skill is a `.toml` config file with optional Rust `.so` plugin. Skills live in `~/.agent/skills/`. The Rust `SkillRegistry` loads them at startup via the `Skill` trait.

```rust
pub trait Skill: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn to_tool_schema(&self) -> serde_json::Value;
    fn execute(&self, params: &Value) -> Pin<Box<dyn Future<Output = SkillResult>>>;
}

pub struct SkillRegistry {
    skills: HashMap<String, Box<dyn Skill>>,
}
```

## Skill TOML Format

### HTTP-based skill
```toml
# ~/.agent/skills/web_search.toml
[skill]
name = "web_search"
description = "Search the web via local self-hosted meta-search engine"
trigger_keywords = ["search", "look up", "find", "what is", "news"]
complexity = "Standard"

[tool]
name = "web_search"
parameters = { query = "string", max_results = "integer" }

# Primary: Websurfx (Rust, Actix-Web) — zero cost, runs on-device
[implementation]
type = "http"
url = "http://127.0.0.1:8080/search?q={query}&page=1"

# Fallback: SearXNG (Python, mature JSON API)
# url = "http://127.0.0.1:8888/search?q={query}&format=json"
```

### Android Intent skill
```toml
# ~/.agent/skills/phone_call.toml
[skill]
name = "phone_call"
description = "Make a phone call to a contact or number"
trigger_keywords = ["call", "phone", "dial", "ring"]
complexity = "Critical"
requires_confirmation = true

[implementation]
type = "android_intent"
action = "android.intent.action.CALL"
data_template = "tel:{number}"
```

### Calendar Intent skill
```toml
# ~/.agent/skills/reminder.toml
[skill]
name = "set_reminder"
complexity = "Trivial"

[implementation]
type = "android_intent"
action = "android.intent.action.INSERT"
extras = { title = "{title}", beginTime = "{time_ms}" }
```

## Built-in Skill Library

| Skill | Description | Implementation |
|---|---|---|
| `screen_control` | Tap, swipe, type on any app | AccessibilityService |
| `open_app` | Launch app by name or package | Android Intent |
| `web_search` | Search via local self-hosted engine (Websurfx/SearXNG) | HTTP |
| `web_browse` | Open URL, read page content | Chrome + Accessibility |
| `send_message` | Send SMS, Telegram, WhatsApp | Accessibility / Intent |
| `calendar` | Read/create calendar events | ContentProvider |
| `contacts` | Look up contact info | ContentProvider |
| `phone_call` | Make calls | Intent (with confirmation) |
| `set_alarm` | Set alarms/timers | AlarmManager Intent |
| `file_read` | Read a file from storage | File API |
| `file_write` | Write/create a file | File API |
| `clipboard` | Read or write clipboard | ClipboardManager |
| `screenshot` | Capture current screen | MediaProjection |
| `notifications` | Read notification list | NotificationListenerService |
| `shell_cmd` | Run shell command (Termux IPC) | Intent to Termux |
| `camera` | Take a photo | CameraX |
| `tts_speak` | Speak text aloud | Android TTS |
| `memory_read` | Read from MEMORY.md | File API |
| `memory_write` | Update MEMORY.md | File API |

## Skill Management (Settings UI)
- Installed skills list with enable/disable toggles
- Skill detail: description, trigger keywords, complexity tier, last used, usage count
- Install from URL (`.toml` file) or built-in gallery
- Per-skill confirmation toggle (require user approval before execution)

## Implementation Types

1. **http** — REST API calls (e.g., Brave Search)
2. **android_intent** — Android Intents (open apps, make calls, set alarms)
3. **shell_cmd** — Termux IPC (gated behind Developer Mode)
4. **Rust .so plugin** — Compiled native code (treat with same caution as APK installs)

## Security
- External `.toml` skills can only use declared implementation types
- `shell_cmd` gated behind explicit "Developer Mode" toggle with red warning UI
- Rust `.so` plugins require sideloading — same risk as APK installs
