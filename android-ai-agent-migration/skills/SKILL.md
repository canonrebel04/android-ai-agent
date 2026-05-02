---
name: obsidian-github-docgen
description: Two-phase pipeline that crawls a GitHub repo, feeds every source file through a local Ollama LLM to generate Obsidian documentation notes, then linkifies them with cross-reference markdown links. Deeply integrated with Hermes memory and auto-crawl.
version: 2.1.0
category: productivity
metadata:
  hermes:
    tags: [obsidian, github, ollama, documentation, llm, crawl, markdown]
    related_skills: [obsidian, obsidian-web-crawler]
    auto_trigger: github.com URLs in user messages
    requires:
      env:
        - OBSIDIAN_VAULT_PATH
        - OLLAMA_HOST
        - OLLAMA_MODEL
      commands:
        - python3
        - crawl4ai
      services:
        - ollama
---

# Obsidian Documentation Generator

Two-phase pipeline: crawl any website → generate per-page docs with local LLM → linkify cross-references. Auto-detects GitHub repos vs generic documentation sites.

## Quick Start

```bash
source /home/miyabi/crawl-venv/bin/activate

# GitHub repo
python3 obsidian_crawler.py https://github.com/Devvvmn/ActivSpot

# Generic documentation site (auto-detected)
python3 obsidian_crawler.py https://quickshell.org/docs/v0.2.1/

# Flags
python3 obsidian_crawler.py --test --depth=2 URL    # Test: 8 pages, depth 2
python3 obsidian_crawler.py --no-linkify URL         # Skip Phase 2
python3 obsidian_crawler.py --depth=3 URL            # Custom max depth
```

## Architecture

```
Phase 1: Crawl + Generate
  crawl4ai discovers files → raw.githubusercontent.com fetches content
  → Ollama LLM generates docs (Purpose, Key Components, Dependencies, Notes)

Phase 2: Linkify
  Each doc re-fed to LLM with full file index
  → LLM inserts [text](relative/path.md) links between related files
```

### LLM Prompt Structure
Each file gets: file type hint, truncated source (6k chars for 16k context), structured output template.

### Concurrency
- Crawl: semaphore(3) to avoid Playwright EPIPE on Arch
- LLM: semaphore(2) to avoid overwhelming local Ollama

## File Structure

```
~/.hermes/skills/productivity/obsidian-github-docgen/
├── SKILL.md                     ← this file
├── scripts/
│   └── obsidian_crawler.py      ← the crawler script
├── templates/
│   └── env.template             ← OLLAMA_HOST, OLLAMA_MODEL, OBSIDIAN_VAULT_PATH
└── references/
    └── setup.md                 ← machine setup instructions
```

## Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `OLLAMA_HOST` | `http://100.70.230.26:11434` | Ollama API endpoint |
| `OLLAMA_MODEL` | `gemma4:e4b` | Model ID |
| `OBSIDIAN_VAULT_PATH` | `~/Documents/Obsidian Vault` | Vault location |

## Noise Filter

The following GitHub paths are excluded from crawling:
`/login`, `/search`, `/signup`, `/actions`, `/projects`, `/security`,
`/pulse`, `/network`, `/stargazers`, `/commits`, `/branches`, `/tags`,
`/watchers`, `/forks`, `/releases`, `/graphs`, `/activity`, `/community`,
`/settings`, `/issues`, `/pull`, `/raw/`, `/commit/`

Binary/image extensions are also skipped: `.png`, `.jpg`, `.gif`, `.ico`, `.svg`, `.pdf`, `.zip`, `.tar.gz`

## Hermes Integration

### Auto-Crawl Trigger
When the user sends a `github.com` URL, Hermes detects it and offers to run the crawler in the background. The persona file at `~/.hermes/persona.md` contains the detection rules.

### Memory Integration
- Key code structures, function signatures, and architectural decisions are saved to memory during chat
- After each session, a summary (problem, solution, changes, target files) is saved to memory
- Pre-response memory search ensures context from past sessions is available

### Cron Job (Optional)
Set up a recurring cron job for periodic re-crawls:

```
cronjob action=create schedule="0 6 * * *" prompt="Run obsidian_crawler.py on https://github.com/Devvvmn/ActivSpot"
```

## Related Skills (Absorbed)

The `obsidian-web-crawler` skill (v1 single-file approach) has been absorbed into this skill. See `references/obsidian-web-crawler-legacy.md` for historical context, original CLI flags, and design decisions.

## Reference Files

- `references/manual-extraction.md` — Workflow for repos with no source code or when Ollama is unavailable. Covers triage, structuring, writing notes, dependency pinning, and cleanup.
- `references/manual-fallback-example.md` — Full session transcript from `canonrebel04/android-ai-app`, a 740-line spec extracted into 6 Obsidian notes.
- `references/dependency-research.md` — Post-extraction workflow: live version pinning for every dependency, docs.rs API extraction, and consolidated dependency manifest creation.

## Pre-Flight Check

Before firing the crawler, verify Ollama is reachable. If not, skip the crawler and use Manual Fallback Mode.

```bash
# Check if Ollama is responding
curl -s --max-time 5 "$OLLAMA_HOST/api/tags" >/dev/null 2>&1 && echo "OLLAMA_OK" || echo "OLLAMA_DOWN"
```

If `OLLAMA_DOWN`, do NOT start the crawler — it will hang forever waiting for LLM responses. Offer Manual Fallback instead.

## Repo Type Detection

After cloning, check what kind of repo this is BEFORE running the crawler:

```bash
# Count code files (non-markdown, non-config)
find . -not -path './.git/*' -type f \
  ! -name '*.md' ! -name '*.txt' ! -name '*.toml' ! -name '*.yaml' ! -name '*.json' \
  ! -name 'LICENSE' ! -name 'Makefile' ! -name 'Dockerfile' \
  | wc -l
```

If the count is 0 (spec/doc-only repo), skip the crawler and use Manual Fallback Mode. The crawler needs actual source files to feed through Ollama — a single markdown spec file produces nothing useful.

## Manual Fallback Mode

When Ollama is unavailable OR the repo is spec/docs-only (no code files), manually extract structured Obsidian notes:

1. Clone the repo, read the key files
2. Identify logical sections — in a spec doc, each top-level heading is a candidate note
3. Split into 4-7 cross-linked notes using `[[wikilinks]]`
4. Create a master index note listing all sub-notes with descriptions
5. Write all notes to `$OBSIDIAN_VAULT_PATH/<repo-name>/`

Format: plain markdown with Obsidian wikilinks. Code blocks, tables, and architecture diagrams preserved from source. No LLM embellishment — just clean extraction and organization.

Reference: `references/manual-fallback-example.md` — full session transcript from `canonrebel04/android-ai-app` showing the pattern applied to a 740-line spec.

See `references/manual-fallback-example.md` for the `android-ai-app` session template.

After manual extraction, if the spec lists dependencies, consider the dependency research workflow in `references/dependency-research.md` — live version pinning + crate docs extraction produces a valuable consolidated manifest note.

## Pitfalls

- **Spec-only repos:** If a GitHub repo contains only markdown/docs/spec files (no source code), skip the crawler. The crawler needs actual code files to feed through Ollama. Instead, manually read the spec and write structured Obsidian notes by hand. Example: `canonrebel04/android-ai-app` is a single 740-line markdown spec — 6 hand-written notes captured it better than the crawler would have.
- **16k context limit**: Source truncated to 6000 chars. Very large files get incomplete analysis.
- **EPIPE errors**: Keep crawl concurrency ≤ 3 on Arch/Playwright.
- **Ollama timeouts**: 180s timeout per LLM call. Large repos take time.
- **Resume logic**: Files > 200 bytes are skipped on re-crawl. Delete the repo folder to force fresh.
- **crawl4ai markdown API**: `result.markdown` is an object. Use `result.markdown.fit_markdown` or `raw_markdown` (fallback only — raw content comes from raw.githubusercontent.com).
- **Doc-only repos (no source code)**: When a GitHub repo contains only spec/docs/markdown files with no actual source code, the crawler has nothing to feed through Ollama. In this case, skip the crawler entirely and manually extract structured Obsidian notes from the markdown files. Read the full doc, split into logical sections, write each as a separate `.md` note in the vault with `[[wikilinks]]` between them. Example: `canonrebel04/android-ai-app` was a single 740-line spec file — manually extracted into 6 notes covering architecture, model layer, skills, channels, memory, and roadmap.

## Manual Fallback — No LLM Available

When Ollama is unreachable or the user has no local LLM configured, skip the crawler entirely and document manually:

1. **Kill any background crawler** — it will hang without LLM and produce an empty directory
2. **Clone the repo** with `gh repo clone owner/repo`
3. **Map the file tree** — read the full directory structure
4. **Read key files** — source code, README, docs, specs
5. **Write structured Obsidian notes** — split by topic, not by file. Use `[[wikilinks]]` for cross-references. Target 5-8 notes for a typical repo
6. **For spec-only repos** (like android_agent_spec_v2-1.md @ 740 lines): read the full spec, extract sections into topic notes (Architecture, Model Layer, Skills, Channels, Memory, Roadmap, Dependencies)
7. **For code repos**: read key source files, extract the module map + architecture, write per-module notes with code excerpts

This manual workflow is also the right approach for repos that contain only markdown/docs (no source code to crawl).
- **Spec-only repos (no source files)**: When a GitHub repo contains only markdown/docs/spec files (no `.rs`, `.kt`, `.py`, `.js`, etc.), skip the crawler entirely. The LLM pipeline needs actual source files to analyze. Instead, manually read the spec files and extract structured Obsidian notes — split by section, use `[[wikilinks]]` for cross-referencing, and create a master index note. This is faster and produces better results than forcing the crawler on docs.
