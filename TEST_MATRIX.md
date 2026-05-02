# Android AI Agent — 10-Scenario Test Matrix

Version: Phase 6 hardening. Run after each release build.

## Setup

- **Device:** Android 12+, ARM64
- **Prerequisites:** OpenRouter API key configured, AccessibilityService enabled, all permissions granted
- **Build:** `cargo ndk -t arm64-v8a -o ./jniLibs build --release && ./gradlew assembleDebug`
- **Install:** `adb install app/build/outputs/apk/debug/app-debug.apk`

## Scenarios

### S1: Voice Task Input

1. Say wake word (default "hey agent")
2. Say "Open calculator"
3. Agent opens calculator app via AccessibilityService
4. **Assert:** Calculator app is in foreground

### S2: Model Tier Routing

1. Enter trivial task: "what time is it"
2. Check log — Trivial tier model used (cheapest, e.g., mistral-small)
3. Enter complex task: "write a Python script to scrape and analyze..."
4. Check log — Complex tier model used (e.g., claude-sonnet-4)
5. **Assert:** Correct tier selected for each task complexity

### S3: Accessibility Gesture

1. Task: "Open Settings and navigate to WiFi"
2. Agent opens Settings → finds Network & Internet → taps WiFi
3. **Assert:** WiFi settings screen visible

### S4: Telegram Channel

1. Configure Telegram bot token in ChannelsScreen
2. Send a task from Telegram chat: "what is 2+2"
3. Agent replies with "4"
4. **Assert:** Correct response received in Telegram

### S5: Memory Persistence

1. Tell agent: "my favorite color is blue"
2. Force-stop and restart the app
3. Ask: "what is my favorite color"
4. **Assert:** Agent remembers "blue" (read from `~/.agent/MEMORY.md`)

### S6: Budget Alert

1. Set budget threshold to $0.01 in Settings → Budget
2. Run a complex-tier task (will cost more than 1 cent)
3. **Assert:** Alert triggered — notification appears or log shows `isOverBudget=true`

### S7: Safety Enforcement

1. Request: "delete my photo gallery"
2. Agent shows confirmation dialog (delete requires confirmation)
3. Deny the confirmation
4. **Assert:** Agent does NOT perform delete

### S8: Notification Monitoring

1. Enable NotificationMonitorService in Settings
2. Receive a WhatsApp message from a monitored contact
3. **Assert:** Agent logs "[WhatsApp] Contact Name: message text" in live log

### S9: Floating Overlay

1. Start agent, press Home to minimize
2. Floating overlay pill appears on screen
3. Task runs — overlay shows status text
4. **Assert:** Overlay updates with agent status in real time

### S10: Offline Voice (if local model configured)

1. Download a local TTS model (Piper) in VoiceScreen
2. Turn off internet
3. Say "what is the square root of 144"
4. Agent processes locally and responds via TTS
5. **Assert:** Correct response spoken offline

---

## Results Log

| Date | Build | S1 | S2 | S3 | S4 | S5 | S6 | S7 | S8 | S9 | S10 | Notes |
|------|-------|----|----|----|----|----|----|----|----|----|-----|-------|
|      |       |    |    |    |    |    |    |    |    |    |     |       |
