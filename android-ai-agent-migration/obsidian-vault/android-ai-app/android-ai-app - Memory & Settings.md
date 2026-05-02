# Android AI App — Memory & Settings

## Persistent Memory System

The agent reads and writes a structured `MEMORY.md` file stored at `~/.agent/MEMORY.md`. This file is injected into every system prompt, giving the agent cross-session context.

### Memory File Structure

```markdown
# Agent Memory

## User Profile
- Name: [extracted from conversations]
- Preferred model: claude-sonnet-4-6
- Communication style: concise, technical

## Persistent Facts
- Work email: ...
- Home WiFi name: ...
- Usual morning routine: ...

## Recent Context
- Last task: "Sent email to Alex about the meeting" (2026-04-21)
- Pending: "Follow up with dentist office"

## Learned Preferences
- Prefers Telegram for status updates
- Dislikes verbose responses
- Always confirms before sending messages

## Skills Used Most
1. send_message (47 times)
2. web_search (31 times)
3. screen_control (28 times)
```

### Memory Update Loop

After every completed task, the agent calls the model with:

```
System: You are a memory manager. The user just completed a task.
Update the memory file to reflect any new facts, preferences, or context learned.
Return ONLY the updated memory file. Keep it under 800 tokens.

Current memory: {MEMORY_MD}
Completed task: {TASK}
Task summary: {SUMMARY}
```

The Rust core writes the result back to `~/.agent/MEMORY.md`. This model call uses the **Trivial** tier (cheap).

### Memory Settings
- Enable persistent memory: toggle
- View/edit MEMORY.md directly
- Update frequency: after every task / daily summary / manual only
- Clear memory

---

## Full Settings Screen

### Models
- Provider picker (OpenRouter / Mistral / Anthropic / Gemini / OpenAI / Local / Custom)
- API key per provider (Android Keystore-backed)
- Tier configuration: Trivial / Standard / Complex / Critical model per tier
- Fallback chain editor (add/reorder models)
- Enable prompt caching toggle (for supported providers)
- Auto model: use `openrouter/auto` for intelligent routing

### Skills
- Installed skills list with enable/disable toggles
- Skill detail: description, trigger keywords, complexity tier, last used, usage count
- Install from URL (`.toml` file) or built-in gallery
- Per-skill confirmation toggle

### Channels
- Telegram: bot token input, test connection, notification settings
- WhatsApp: enable monitoring (explicit consent UI), trigger contact selector
- Voice: wake word on/off, wake word phrase, TTS engine/voice selection
- Gateway: enable toggle, port, auth token, QR code, Tailscale mode

### Agent Behavior
- Max steps per task (10–100, default 50)
- Action delay between steps (0–3000ms, default 300ms)
- Stall timeout (2–10s)
- Loop guard (max identical screens before abort)
- Require confirmation for: Send / Delete / Payment / Call (individual toggles)
- Vision Mode: off / on (adds screenshot to every LLM call, higher cost)

### Usage & Cost
- Token usage by task (today / week / month)
- Estimated cost by tier
- Budget alert threshold (notify when monthly spend exceeds $X)
- Export usage log as CSV

---

## Permissions Manifest

```
Screen control:  BIND_ACCESSIBILITY_SERVICE
Background:      FOREGROUND_SERVICE, WAKE_LOCK, IGNORE_BATTERY_OPTIMIZATIONS, BOOT_COMPLETED
Networking:      INTERNET, ACCESS_NETWORK_STATE
Channels:        RECORD_AUDIO, CAMERA, BIND_NOTIFICATION_LISTENER_SERVICE
Phone:           READ_CONTACTS, READ_CALL_LOG, CALL_PHONE, SEND_SMS
Calendar:        READ_CALENDAR, WRITE_CALENDAR
Storage:         READ_EXTERNAL_STORAGE, WRITE_EXTERNAL_STORAGE
Vision:          FOREGROUND_SERVICE_MEDIA_PROJECTION
```

**Onboarding flow:** Each permission group explained on a dedicated card before the system dialog fires — what it enables, why it's needed, what it will never be used for.
