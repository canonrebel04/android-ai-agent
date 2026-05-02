# Android AI App — Roadmap & Security

## Development Roadmap (7 Weeks)

### Phase 1 — Core Engine (Weeks 1–2)
- [ ] Rust: `http_client` module with all 5 provider implementations
- [ ] Rust: `tool_parser` + `model_router` with tier selection
- [ ] Rust: `fallback_chain` retry logic
- [ ] Test via Termux: fire OpenRouter requests to 5+ different models
- [ ] Rust: `skill_registry` trait + TOML loader
- [ ] Rust: `context_manager` with token trimming
- [ ] Rust: `complexity_classifier` (rule-based, no LLM needed)

### Phase 2 — AccessibilityService (Week 3)
- [ ] Screen tree serializer → compact JSON
- [ ] Gesture executor (tap, swipe, type, scroll, key)
- [ ] Action confirmation dialog
- [ ] `screen_control` skill wrapping the service
- [ ] End-to-end: "Open calculator and type 42"

### Phase 3 — Agent Loop Integration (Week 4)
- [ ] JNI bridge + ViewModel wiring
- [ ] Full perception → model call → action → verify cycle
- [ ] Stall detection + loop guard
- [ ] Memory manager: read/write MEMORY.md, post-task update call

### Phase 4 — Skills & Channels (Week 5)
- [ ] Built-in skill library: web_search, open_app, calendar, contacts, clipboard, camera
- [ ] Telegram bot service + full command set
- [ ] Voice: STT task input + TTS response readback
- [ ] Notification listener skill
- [ ] Termux shell_cmd skill (IPC via Intent)

### Phase 5 — UI & Settings (Week 6)
- [ ] Full Jetpack Compose UI: Home, Models, Skills, Channels, Memory, History
- [ ] Tier configuration UI (drag-to-reorder fallbacks)
- [ ] Token usage + cost estimation tracking
- [ ] Onboarding permission flow
- [ ] Foreground service + persistent notification

### Phase 6 — Hardening (Week 7)
- [ ] Prompt caching for Anthropic + OpenRouter
- [ ] Budget alert system
- [ ] Gateway WebSocket server
- [ ] Vision Mode (MediaProjection)
- [ ] 10-scenario manual test matrix
- [ ] APK signing + sideload

---

## Security Notes

### API Key Storage
All API keys stored in **Android Keystore** (hardware-backed AES/GCM). Never logged, never in plain text config files.

### Gateway Binding
Never expose the gateway to `0.0.0.0` without reverse proxy auth. Bind to `127.0.0.1` or use Tailscale.

### Skill Security
- External `.toml` skills can only use declared implementation types (`http`, `android_intent`, `shell_cmd`)
- They cannot execute arbitrary Rust code
- `shell_cmd` skill (Termux IPC) gated behind explicit **Developer Mode** toggle with red warning UI
- Compiled `.so` plugins: treat with same caution as APK installs

### Prompt Injection Defense
Any text the agent reads from the screen (emails, web pages, notifications) could contain adversarial instructions. Apply a sanitization pass in `context_manager.rs` that strips common injection patterns before they reach the model.

### Confirmation Gates
| Action | Requires Confirmation |
|---|---|
| Send message | Toggle in settings |
| Delete anything | Toggle in settings |
| Payment | Toggle in settings |
| Phone call | Toggle in settings |
| shell_cmd | Always (Developer Mode) |
| Critical-tier actions | Per-skill toggle |

## What Makes This Better Than OpenClaw

1. **Phone control** — No other agent framework does this. Can open Telegram, navigate to a contact, compose a message, and send it — without any API.

2. **Zero dependency on a gateway PC** — OpenClaw's Android app is dead without a Mac/Linux gateway running somewhere. This app is fully standalone.

3. **Rust core** — OpenClaw is Node.js. On a phone: lower memory, faster startup, no GC pauses during action loops.

4. **Unified interface** — Same app controls the phone AND chats via Telegram AND responds to voice. OpenClaw needs multiple separate pieces.

5. **On-device model path** — Route trivial tasks to a local llama.cpp model. Zero cost, zero latency, zero network for simple stuff.
