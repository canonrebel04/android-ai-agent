# Android AI App — Architecture

## Rust Core Module Map

```
rust/src/
├── lib.rs
├── jni_exports.rs          # All JNI entry points
├── agent_loop.rs           # Main perception→reason→act cycle
├── state_machine.rs        # Task lifecycle states
├── model_router.rs         # Tier selection + fallback chains
├── http_client.rs          # Multi-provider HTTP with retries
├── provider/
│   ├── openrouter.rs       # OpenRouter request shaping
│   ├── anthropic.rs        # Anthropic direct (cache markers)
│   ├── google.rs           # Gemini via OpenAI-compat
│   ├── mistral.rs          # Mistral direct
│   └── local.rs            # llama.cpp / Ollama
├── tool_parser.rs          # Parse LLM function calls → AgentAction
├── skill_registry.rs       # Trait + loader for skills
├── skills/
│   ├── screen_control.rs   # Bridges to AccessibilityService via JNI callback
│   ├── web_search.rs       # HTTP search skill
│   └── shell_cmd.rs        # Termux IPC
├── context_manager.rs      # Rolling history with token budget
├── memory_manager.rs       # MEMORY.md read/write/update
├── prompt_cache.rs         # Anthropic/OpenRouter cache_control markers
├── complexity_classifier.rs
├── log_ring_buffer.rs
└── gateway_server.rs       # Optional WebSocket gateway
```

## Android Project Structure (Kotlin)

```
app/src/main/kotlin/com/yourdomain/agent/
├── ui/screens/
│   ├── HomeScreen.kt           # Task input + live log
│   ├── ModelsScreen.kt         # Browse/configure all model tiers
│   ├── SkillsScreen.kt         # Enable/disable/install skills
│   ├── ChannelsScreen.kt       # Telegram, WhatsApp, Voice setup
│   ├── MemoryScreen.kt         # View and edit MEMORY.md
│   ├── HistoryScreen.kt        # Past tasks with token/cost data
│   ├── GatewayScreen.kt        # Local gateway config + QR code
│   └── PermissionsScreen.kt    # Onboarding
├── service/
│   ├── AgentAccessibilityService.kt
│   ├── TelegramBotService.kt
│   ├── VoiceService.kt
│   ├── NotificationListenerService.kt
│   └── GatewayWebSocketService.kt
├── bridge/
│   └── RustBridge.kt
└── data/
    ├── KeystoreManager.kt
    ├── SkillRepository.kt
    └── db/  (Room: tasks, skills, memory_snapshots)
```

## JNI Bridge

Kotlin `AgentViewModel` communicates with Rust core via JNI. The ViewModel manages:
- Coroutine-based async dispatch
- Channel bridge for Telegram/WhatsApp/Voice
- Log relay from the Rust ring buffer to the UI

## Architecture Diagram

```
┌──────────────────────────────────────────────────────────────┐
│                    Android App Process                        │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              Jetpack Compose UI                          │ │
│  │   Home · History · Skills · Models · Settings · Channels │ │
│  └────────────────────┬────────────────────────────────────┘ │
│                       │ StateFlow                             │
│  ┌────────────────────▼────────────────────────────────────┐ │
│  │              AgentViewModel (Kotlin)                     │ │
│  │       Coroutines · Channel bridge · Log relay           │ │
│  └────────────────────┬────────────────────────────────────┘ │
│                       │ JNI                                   │
│  ┌────────────────────▼────────────────────────────────────┐ │
│  │                Rust Core (libagent.so)                   │ │
│  │                                                          │ │
│  │  ┌───────────────┐ ┌─────────────┐ ┌──────────────────┐ │ │
│  │  │  Agent Loop   │ │Model Router │ │  Skills Engine   │ │ │
│  │  │  State Mach.  │ │Tier Select  │ │  Plugin Loader   │ │ │
│  │  └──────┬────────┘ └──────┬──────┘ └────────┬─────────┘ │ │
│  │         │                 │                  │           │ │
│  │  ┌──────▼─────────────────▼──────────────────▼─────────┐│ │
│  │  │            HTTP Client (reqwest)                     ││ │
│  │  │   OpenRouter · Mistral · Anthropic · Google · Local  ││ │
│  │  └──────────────────────────────────────────────────────┘│ │
│  │                                                          │ │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────────┐ │ │
│  │  │Context Mgr   │ │Memory Mgr    │ │ Log Ring Buffer  │ │ │
│  │  │Token Trimmer │ │MEMORY.md I/O │ │ Trace Output     │ │ │
│  │  └──────────────┘ └──────────────┘ └──────────────────┘ │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                               │
│  ┌───────────────────┐  ┌──────────────────────────────────┐ │
│  │AccessibilityService│  │     Channel Services             │ │
│  │Screen Reader       │  │ TelegramBotService               │ │
│  │Node Resolver       │  │ WhatsApp (via Accessibility)     │ │
│  │Gesture Executor    │  │ VoiceService (STT/TTS)           │ │
│  └───────────────────┘  └──────────────────────────────────┘ │
│                                                               │
│  ┌──────────────────────────────────────────────────────────┐│
│  │        Gateway WebSocket Server (optional)                ││
│  │   Expose agent to local network / Pi / desktop           ││
│  └──────────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────────┘
```
