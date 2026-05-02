# Manual Fallback Example — android-ai-app

Session: 2026-05-01, repo `canonrebel04/android-ai-app`

## Repo Profile
- Single 740-line `android_agent_spec_v2-1.md` file + 16-byte `README.md`
- Zero code files — spec/documentation only
- Ollama was not available on this machine

## What Went Wrong
1. Crawler was fired in background → would have hung waiting for Ollama
2. User intervened: "its not going to generate docs as i dont have the llm hooked up"
3. Process killed, manual extraction done instead

## Manual Extraction Result
740-line spec → 6 Obsidian notes (675 lines total):

| Note | Lines | Content |
|---|---|---|
| `android-ai-app.md` | 56 | Master index + OpenClaw comparison table + doc map |
| `android-ai-app - Architecture.md` | 114 | Rust core module map, Kotlin project structure, JNI bridge, full architecture diagram |
| `android-ai-app - Model Layer.md` | 102 | 6 providers, tiered routing (Trivial→Critical), complexity classifier, fallback chains, prompt caching |
| `android-ai-app - Skills System.md` | 111 | Plugin trait + TOML format, 19 built-in skills, implementation types |
| `android-ai-app - Channels & Voice.md` | 92 | Telegram bot commands, WhatsApp accessibility channel, voice STT/TTS, WebSocket gateway protocol |
| `android-ai-app - Memory & Settings.md` | 112 | MEMORY.md structure, update loop, full settings screen, permissions manifest |
| `android-ai-app - Roadmap & Security.md` | 88 | 6-phase 7-week dev plan, API key storage, prompt injection defense, confirmation gates |

## Extraction Pattern
1. Identify top-level `##` sections in the spec (14 total)
2. Group related sections into 5-6 thematic notes
3. Each note gets: section content (tables, code blocks, diagrams preserved), wikilinks to sibling notes
4. Master index lists all notes with one-line descriptions + `[[wikilinks]]`
5. Write all files to vault at `/home/miyabi/Documents/Obsidian Vault/<repo-name>/`

## Key Takeaway
Spec-only repos are common (design docs, RFCs, architecture specs). Don't try to crawl them — just do structured extraction. The crawler is for repos with actual source code files.
