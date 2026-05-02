# Android Local Agent — Master Index

> Source: `canonrebel04/android-ai-app` — `android_agent_spec_v2-1.md`

A self-contained Android agent that surpasses OpenClaw: every model OpenRouter offers, a skills plugin system, messaging-app UI channels, voice mode, persistent memory — plus direct AccessibilityService control of any app on the phone.

## Differentiator vs OpenClaw

| Capability | OpenClaw | This App |
|---|---|---|
| 300+ models via OpenRouter | yes | yes |
| Skills / plugin system | yes | yes |
| Telegram / WhatsApp as UI | yes | yes |
| Voice mode (TTS + STT) | yes | yes |
| Persistent memory | yes | yes |
| Runs fully on-device | no | **yes** |
| Screen reading (AccessibilityService) | no | **yes** |
| Tapping / swiping any app | no | **yes** |
| Native Android UI (Jetpack Compose) | no | **yes** |
| Rust core (memory-safe, fast) | no (Node.js) | **yes** |

OpenClaw is a messaging bot that can run shell commands. This app IS the phone — it can open your banking app, navigate to transfers, and hit Send autonomously.

## Document Map

- [[android-ai-app - Architecture]] — Rust core + Android project structure + JNI bridge
- [[android-ai-app - Model Layer]] — Providers, tiered routing, fallback chains, prompt caching
- [[android-ai-app - Skills System]] — Plugin architecture, built-in skills, TOML configs
- [[android-ai-app - Channels & Voice]] — Telegram bot, WhatsApp, voice mode, WebSocket gateway
- [[android-ai-app - Memory & Settings]] — MEMORY.md system, settings screens, permissions
- [[android-ai-app - Roadmap & Security]] — 7-week dev plan, security considerations
- [[android-ai-app - Dependencies]] — All pinned versions with crate docs
- [[Self-Hosted Search Engines]] — On-device search engine comparison (Websurfx, Meilisearch, tinysearch)
- [[Swarm - Reusable Patterns]] — Patterns mined from `canonrebel04/swarm` for safety, roles, and orchestration
- [[KimiClaw - Feature Comparison]] — Comparison with KimiClaw floating pet app; borrowed NotificationListenerService + floating overlay patterns

---

## Quick Architecture

```
Jetpack Compose UI (Kotlin)
  → AgentViewModel (Kotlin coroutines)
    → JNI bridge
      → Rust Core (libagent.so)
        ├── Agent Loop (perception→reason→act)
        ├── Model Router (tiered + fallback)
        ├── Skills Engine (plugin loader)
        └── HTTP Client (reqwest → OpenRouter/Anthropic/Mistral/Gemini/Local)

Android Services:
  ├── AccessibilityService (screen read + gestures)
  ├── TelegramBotService (bot polling)
  ├── VoiceService (STT + TTS)
  └── GatewayWebSocketServer (optional LAN/Tailscale)
```

**Key repos:** `canonrebel04/android-ai-app`  
**Spec version:** v2.1  
**License:** Unknown (MIT assumed from structure)
