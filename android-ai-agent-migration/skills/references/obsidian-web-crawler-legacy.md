# obsidian-web-crawler (Archived Reference)

This is the predecessor single-file crawler approach. `obsidian-github-docgen` v2.0 absorbed and extended this with a two-phase pipeline, auto-crawl triggers, and Cron integration.

## Original Architecture

Two-phase pipeline:

```
Phase 1: Crawl + Generate
  crawl4ai discovers all blob URLs
    → fetch raw content from raw.githubusercontent.com
    → send to Ollama with structured prompt
    → save generated .md doc with YAML frontmatter

Phase 2: Linkify
  collect all generated .md files
    → for each doc, resend to Ollama with full file index
    → LLM inserts [text](relative/path.md) links for cross-references
    → rewrite doc with phase: 2 in frontmatter
```

## Original Script Location

`/home/miyabi/obsidian_crawler.py` (now superseded by the v2.0 crawler in obsidian-github-docgen's scripts/)

## Running (deprecated — use obsidian-github-docgen instead)

```bash
source /home/miyabi/crawl-venv/bin/activate
python3 obsidian_crawler.py <GITHUB_URL>            # Full crawl (500 pages)
python3 obsidian_crawler.py <GITHUB_URL> --test     # Test (8 pages)
python3 obsidian_crawler.py <GITHUB_URL> --no-linkify  # Skip Phase 2
```

## Original Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Raw file fetching (not blob page) | Zero GitHub UI chrome, pristine content for LLM |
| Two-phase (generate then linkify) | Can't link to files that haven't been documented yet |
| LLM semaphore = 2 | Ollama single-GPU; overloading causes timeouts |
| 6000-char source truncation | 16k context window; leave room for prompt + response |
| Skip actions/projects/security URLs | GitHub noise pages waste crawl slots |
| `--no-linkify` flag | Phase 2 is slow; useful for testing Phase 1 alone |

## Output Structure

```
~/Documents/Obsidian Vault/<REPO_NAME>/
├── README.md              (phase: 2, linked)
├── LICENSE.md             (phase: 2)
├── .gitignore.md          (phase: 2)
├── install.sh.md          (phase: 2, links to hyprland.conf.md)
├── hyprland.conf.md       (phase: 2, links to scripts/*)
└── scripts/
    ├── init.sh.md
    └── quickshell/
        └── TopBar.qml.md
```

## Original Pitfalls (historical reference)

- 16k context: Don't send full source for files >6000 chars
- Ollama timeouts: Keep LLM concurrency at 2
- Ollama API: Uses non-streaming mode (`"stream": false`) — returns `{"message": {"content": "..."}}`
- crawl4ai v0.8.x: `result.markdown` is an object. Access `result.links['internal']` for link discovery
- GitHub noise: `/actions`, `/projects`, `/security`, `/pulse`, `/network` — all filtered by `is_internal()`
- Resume: Files >200 bytes with valid .md extension are skipped on re-crawl
- Frontmatter parsing: Uses `text.index('\n---\n', 3)` to find the closing `---`
