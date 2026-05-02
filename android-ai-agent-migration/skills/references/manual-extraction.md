# Manual Extraction Workflow (No LLM Fallback)

Use when: the repo has no source code files, or Ollama is unavailable.

## The 3-Phase Pattern

### Phase 1: Triage the Repo

```bash
# Clone and inspect
gh repo clone owner/repo
ls -la repo/
file repo/*
```

Decision matrix:
- **Source code** (.rs/.py/.kt/.js/etc.) + Ollama available → use the crawler
- **Spec/docs only** (.md only) → manual extraction
- **Source code but no Ollama** → manual extraction

### Phase 2: Read and Structure

1. Read the entire spec/doc file(s) with `read_file`
2. Identify natural section boundaries
3. Plan 5-8 Obsidian notes max — one master index + topic pages
4. Use `[[wikilinks]]` between related notes

### Phase 3: Write Notes

Write directly to the Obsidian vault with `write_file`:

```
/home/miyabi/Documents/Obsidian Vault/<repo-name>/
├── repo-name.md                       ← Master index
├── repo-name - Topic A.md             ← Self-contained topic
├── repo-name - Topic B.md
├── repo-name - Dependencies.md        ← If applicable (see below)
└── ...
```

Each note should have:
- A clear H1 title
- Source attribution (repo URL + file name)
- Cross-reference links to sibling notes
- Code blocks, tables, and diagrams that stand alone

## Dependency Pinning (if the spec mentions tools/libraries)

When a spec references dependencies without pinning versions:

1. List every dependency mentioned (Android libs, Rust crates, external APIs)
2. Run parallel `web_search` queries for each:
   - Android: `"<lib> latest stable version 2026 developer.android.com"`
   - Rust: `"<crate> latest version crates.io 2026"`
   - APIs: `"<service> API pricing documentation 2026"`
3. Also run `web_extract` on docs.rs pages for key Rust crates to get API reference
4. Write a consolidated `repo-name - Dependencies.md` with:
   - Pinned versions with release dates and sources
   - Copy-pasteable build config snippets (Cargo.toml, build.gradle.kts)
   - Quick-reference summary table at the bottom
   - Key API reference examples for core crates
5. Update the master index to include a link to the dependencies note

## Cleaning Up

After all notes are written to the vault, offer to remove the cloned repo:
```bash
rm -rf ./repo-name
```
