# Android AI App — Channels & Voice

## Telegram Bot

### Polling Architecture
Kotlin `TelegramBotService` as a foreground service. Polls Telegram API every second for new messages, dispatches commands to the agent engine.

### Commands

| Command | Action |
|---|---|
| `/status` | Current agent state + last action |
| `/stop` | Halt running task |
| `/logs [n]` | Last n log lines |
| `/model <name>` | Switch active model |
| `/skills` | List installed skills |
| `/memory` | Show current MEMORY.md |
| `/tier <trivial\|standard\|complex\|critical> <model>` | Update routing tier |
| Any other text | Execute as a new task |

Tasks started via Telegram stream progress back in real-time as the agent works.

### Telegram Settings
- Bot token input
- Test connection button
- Notification settings

## WhatsApp Channel

WhatsApp has no bot API. Instead, the app uses the AccessibilityService to:
1. Monitor WhatsApp notifications from a designated trigger contact/group
2. Parse the incoming message
3. Reply via the WhatsApp compose field

Opt-in only, clearly disclosed to the user.

## Voice Mode

```kotlin
class VoiceService : Service() {
    // Wake word detection → STT → agent task → TTS response
    private val speechRecognizer = SpeechRecognizer.createSpeechRecognizer(this)
    private val tts = TextToSpeech(this) { /* init */ }

    fun startListening(wakeWord: String = "hey agent") {
        // On-device wake word detection (Porcupine or Rhino via ONNX)
        // Then full STT on wake
        speechRecognizer.startListening(intent)
    }

    fun speak(text: String) {
        tts.speak(text, TextToSpeech.QUEUE_FLUSH, null, null)
    }
}
```

**Voice Settings:**
- Wake word on/off toggle
- Wake word phrase (default: "hey agent")
- TTS engine/voice selection

## Gateway WebSocket Server (Optional)

Expose the agent over local network or Tailscale so a Pi, laptop, or desktop can send tasks.

```rust
pub struct GatewayServer {
    port: u16,
    auth_token: String,
}
```

### WebSocket Protocol

```
Client → Agent: { "type": "task", "prompt": "...", "model": "..." }
Agent → Client: { "type": "log", "text": "..." }
Agent → Client: { "type": "action", "action": {...} }
Agent → Client: { "type": "done", "summary": "..." }
Agent → Client: { "type": "confirm_required", "action": {...} }
Client → Agent: { "type": "confirm", "approved": true }
```

### Gateway Settings
- Enable toggle
- Port (default 8765)
- Auth token (auto-generated)
- QR code for easy pairing with Pi
- Tailscale mode

## Security: Gateway
Never expose to 0.0.0.0 without reverse proxy auth. Bind to `127.0.0.1` or use Tailscale.
