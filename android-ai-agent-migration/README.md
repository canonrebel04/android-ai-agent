# Android AI Agent — Migration Package

Complete context dump for moving the project to another Hermes client machine.

## Contents

```
android-ai-agent-migration/
├── README.md                          ← this file
├── SOUL.md                            ← Hermes persona (copy to ~/.hermes/)
├── facts.json                         ← Holographic memory facts (import to new Hermes)
├── 2026-05-01-*.md                    ← Implementation plans (5 phases)
├── skills/                            ← Hermes skills to copy
│   ├── SKILL.md
│   ├── references/
│   ├── scripts/obsidian_crawler.py
│   └── templates/env.template
└── obsidian-vault/android-ai-app/     ← Obsidian vault notes (copy to vault)
    ├── android-ai-app.md
    ├── android-ai-app - Architecture.md
    ├── android-ai-app - Model Layer.md
    ├── android-ai-app - Skills System.md
    ├── android-ai-app - Channels & Voice.md
    ├── android-ai-app - Memory & Settings.md
    ├── android-ai-app - Roadmap & Security.md
    ├── android-ai-app - Dependencies.md
    ├── Self-Hosted Search Engines.md
    ├── Swarm - Reusable Patterns.md
    └── KimiClaw - Feature Comparison.md
```

## Recreation Instructions

### Step 1: Clone the repo

```bash
git clone https://github.com/canonrebel04/android-ai-agent
cd android-ai-agent
```

### Step 2: Restore Hermes persona

```bash
cp SOUL.md ~/.hermes/
```

### Step 3: Restore Hermes skills

```bash
cp -r skills/obsidian-github-docgen ~/.hermes/skills/
```

### Step 4: Restore implementation plans

```bash
mkdir -p ~/.hermes/plans
cp 2026-05-01-*.md 2026-05-02-*.md ~/.hermes/plans/
```

### Step 5: Restore Obsidian vault notes

```bash
cp -r obsidian-vault/android-ai-app "~/Documents/Obsidian Vault/"
```

### Step 6: Import facts to Hermes holographic memory

On the new Hermes client, say:

> "Import the following facts from ~/android-ai-agent-migration/facts.json into fact_store"

Or run each fact manually:

```
fact_store(action='add', content='android-ai-agent Phase 1...', category='project', tags='android-ai-agent,rust,phase1')
```

### Step 7: Rebuild for Android ARM64

```bash
export ANDROID_NDK_HOME=/path/to/ndk
cargo ndk -t arm64-v8a -o ./jniLibs build --release
```

### Step 8: Open in Android Studio

Open the `android-ai-agent` directory. The `app/` module contains all Kotlin files. The `jniLibs/` contains the pre-built `.so` (or rebuild from Step 7).

## Key Project Facts (for Hermes context)

- **Tech stack:** Rust core (25 files, 34 tests) + Kotlin Compose UI (20 files, 7 screens)
- **Cross-compile:** edition 2021, serde_json pinned 1.0.140 (nightly compat), jni 0.22 with EnvUnowned pattern
- **Built .so:** `jniLibs/arm64-v8a/libagent_core.so` (1.3MB, Android 21+, NDK r29)
- **Services:** AccessibilityService, NotificationMonitorService (6 apps), TelegramBotService, VoiceService, FloatingAgentOverlay
- **Security:** Android Keystore (AES-256/GCM), SkillPolicyEnforcer (tier gating), AndroidPermissionGuard
- **Patterns borrowed:** KimiClaw (notifications+overlay), Swarm (safety+roles), Kimi Claw help page (official docs)
- **Dep pinning workflow:** web_search each dep → web_extract docs.rs → write consolidated manifest in Obsidian
