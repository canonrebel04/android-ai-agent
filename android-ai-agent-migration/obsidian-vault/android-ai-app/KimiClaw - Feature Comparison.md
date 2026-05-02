# KimiClaw — Feature Comparison & Borrowed Patterns

> Source: `catcatboy-cyber/KimiClaw` (Java, 8 source files, MIT licensed)
> Official: `www.kimi.com/help/kimi-claw`

## What KimiClaw Is

Desktop floating pet app (lobster) for Android. Keyword-based chat. Monitors 6 social apps. NOT an AI agent.

## Feature Matrix

| Feature | KimiClaw | android-ai-agent |
|---|---|---|
| Rust core + LLM | ❌ keyword matching | ✅ OpenRouter 300+ models |
| Tiered model routing | ❌ | ✅ 4 tiers |
| AccessibilityService | ❌ | ✅ tap/swipe/type |
| **NotificationListenerService** | ✅ | ✅ (ported) |
| **Floating overlay** | ✅ WindowManager | ✅ (ported) |
| **Message queue** | ✅ CopyOnWriteArrayList | ✅ StateFlow-based |
| **Multi-app parsing** | ✅ 6 apps | ✅ 6 apps |
| Compose UI (Material3) | ❌ legacy XML | ✅ 6 screens |
| Skill plugin system | ❌ | ✅ 19 skills |
| Safety enforcer | ❌ | ✅ tier gating |
| Android Keystore | ❌ | ✅ AES-256/GCM |
| Telegram bot | ❌ | ✅ |
| Voice mode | ❌ | ✅ |
| WebSocket gateway | ❌ | ✅ |

## Patterns Borrowed

### 1. NotificationListenerService
- `onNotificationPosted(StatusBarNotification)` → reads `extras` for title/text
- App-specific sender extraction (WeChat: title or "sender: msg"; QQ: title)
- Contact matching via `SharedPreferences.getStringSet`
- Broadcast to overlay on match
- **Files:** `NotificationMonitorService.kt`

### 2. WindowManager Floating Overlay
- `TYPE_APPLICATION_OVERLAY` for always-on-top
- `FLAG_NOT_FOCUSABLE | FLAG_NOT_TOUCH_MODAL` for pass-through
- Drag handling, gravity positioning
- **Files:** `FloatingAgentOverlay.kt`

### 3. Thread-Safe Message Buffer
- Max capacity (20 items)
- Thread-safe via `CopyOnWriteArrayList` (Java) → `StateFlow` (Kotlin)
- **Files:** `MessageQueue.kt`

## What KimiClaw Doesn't Have (our advantage)

- No LLM — just keyword matching for "AI chat"
- No AccessibilityService — can't control other apps
- No skill system — everything is hardcoded
- No Compose UI — legacy XML views
- No security model — SharedPreferences for everything, no Keystore
- No model routing — single hardcoded reply logic
