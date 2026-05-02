#!/usr/bin/env python3
"""
Obsidian Documentation Generator — GitHub repos AND generic websites.
Auto-detects GitHub vs generic URL structures.
Phase 1: Crawl + LLM-generate docs for every page
Phase 2: Linkify — cross-reference markdown links between generated docs

Usage:
  python3 obsidian_crawler.py https://github.com/user/repo        # GitHub mode
  python3 obsidian_crawler.py https://docs.example.com/v1/         # Generic mode
  python3 obsidian_crawler.py --test --depth=2 URL                 # Test with depth
  python3 obsidian_crawler.py --no-linkify URL                     # Skip Phase 2
"""

import asyncio
import json
import os
import re
import sys
from pathlib import Path
from urllib.parse import urlparse, urljoin, unquote
from urllib.request import urlopen, Request

from crawl4ai import AsyncWebCrawler, CrawlerRunConfig, CacheMode, BrowserConfig

# ── Config ──────────────────────────────────────────────────────────────
OLLAMA_BASE = os.environ.get("OLLAMA_HOST", "http://100.70.230.26:11434")
OLLAMA_MODEL = os.environ.get("OLLAMA_MODEL", "gemma4:e4b")
LLM_CONCURRENCY = 2
MAX_SOURCE_CHARS = 6000  # for 16k Ollama context

EXT_HINTS = {
    '.sh': 'shell script', '.bash': 'shell script', '.zsh': 'shell script',
    '.py': 'Python script',
    '.qml': 'QML UI component (Qt/Quickshell)',
    '.js': 'JavaScript module', '.ts': 'TypeScript module',
    '.json': 'JSON configuration', '.yaml': 'YAML', '.yml': 'YAML', '.toml': 'TOML',
    '.conf': 'Hyprland/ini config', '.cfg': 'config', '.ini': 'INI config',
    '.css': 'CSS', '.html': 'HTML', '.xml': 'XML',
    '.c': 'C source', '.cpp': 'C++ source', '.h': 'C/C++ header',
    '.rs': 'Rust', '.go': 'Go', '.lua': 'Lua', '.rb': 'Ruby',
}


def slugify(path_component):
    """Turn URL path fragments into safe filesystem names."""
    name = path_component.strip('/') or 'index'
    name = re.sub(r'[<>:"|?*\\]', '_', name)
    return name[:120]


class DocGenerator:
    """Crawls a website, generates docs via Ollama, linkifies cross-refs."""

    def __init__(self, base_url, vault_path, max_depth=5, max_pages=500,
                 crawl_concurrency=3):
        self.base_url = base_url.rstrip('/')
        parsed = urlparse(self.base_url)
        self.domain = parsed.netloc
        self.base_path = parsed.path or '/'
        self.is_github = 'github.com' in self.domain

        self.vault_path = Path(vault_path)
        self.max_depth = max_depth
        self.max_pages = max_pages

        self.page_count = 0
        self.visited = set()
        self.crawl_sem = asyncio.Semaphore(crawl_concurrency)
        self.llm_sem = asyncio.Semaphore(LLM_CONCURRENCY)

        # Determine project name for output folder
        if self.is_github:
            parts = [p for p in self.base_path.split('/') if p]
            self.user = parts[0] if len(parts) > 0 else ''
            self.repo = parts[1] if len(parts) > 1 else slugify(self.domain)
            self.project_name = self.repo
        else:
            self.project_name = slugify(self.domain) or 'website'

        self.root_folder = self.vault_path / self.project_name
        self.root_folder.mkdir(parents=True, exist_ok=True)
        print(f"Project: {self.project_name}  Mode: {'GitHub' if self.is_github else 'Generic'}")

    # ── URL → local path mapping ────────────────────────────────────────

    def url_to_path(self, url):
        """Convert a crawled URL to a relative path for saving."""
        parsed = urlparse(url)
        path = unquote(parsed.path.rstrip('/')) or '/'

        if self.is_github:
            m = re.match(
                rf"^/{re.escape(self.user)}/{re.escape(self.repo)}"
                rf"/(?:blob|tree)/[^/]+/(.+)?$", path)
            if m:
                inner = m.group(1)
                return inner if inner else "README.md"
            if path == f"/{self.user}/{self.repo}":
                return None  # root page, don't save
            return None

        else:
            rel = path.lstrip('/')
            if not rel:
                rel = 'index'
            return rel if rel.endswith('.md') else rel + '.md'

    # ── URL classification ──────────────────────────────────────────────

    def is_internal(self, url):
        """Should this URL be crawled?"""
        p = urlparse(url)

        if p.netloc != self.domain:
            return False
        if not p.path.startswith(self.base_path):
            return False

        skip_ext = ['.png', '.jpg', '.jpeg', '.gif', '.ico', '.svg',
                    '.pdf', '.zip', '.tar.gz', '.mp4', '.webm', '.woff2']
        if any(p.path.lower().endswith(e) for e in skip_ext):
            return False

        if p.fragment:
            return False

        if self.is_github:
            noise = ['/login', '/search', '/signup', '/pulse', '/network',
                     '/stargazers', '/commits', '/branches', '/tags',
                     '/watchers', '/forks', '/releases', '/graphs',
                     '/activity', '/community', '/settings', '/issues',
                     '/pull', '/raw/', '/commit/', '/actions', '/projects',
                     '/security']
            return not any(x in p.path for x in noise)

        return True

    # ── Content fetching ────────────────────────────────────────────────

    def fetch_raw(self, url):
        if self.is_github:
            parsed = urlparse(url)
            path = unquote(parsed.path)
            raw_path = re.sub(r'^/([^/]+)/([^/]+)/blob/', r'/\1/\2/', path)
            raw_url = f"https://raw.githubusercontent.com{raw_path}"
            try:
                req = Request(raw_url, headers={'User-Agent': 'DocGen/2.0'})
                with urlopen(req, timeout=15) as resp:
                    return resp.read().decode('utf-8', errors='replace')
            except Exception as e:
                print(f"   [RAW-FAIL] {raw_url}: {e}")
                return None
        else:
            return None

    def extract_markdown(self, result):
        if hasattr(result, 'markdown') and not isinstance(result.markdown, str):
            return (getattr(result.markdown, 'fit_markdown', '') or
                    getattr(result.markdown, 'raw_markdown', ''))
        return result.markdown or ''

    def clean_markdown(self, md):
        md = re.sub(r'^[=*#~_-]{20,}\s*$', '', md, flags=re.MULTILINE)
        md = re.sub(r'\n{4,}', '\n\n\n', md)
        return md.strip()

    # ── Ollama ──────────────────────────────────────────────────────────

    def _ollama(self, messages):
        body = json.dumps({"model": OLLAMA_MODEL, "messages": messages,
                           "stream": False}).encode()
        req = Request(f"{OLLAMA_BASE}/api/chat", data=body,
                      headers={'Content-Type': 'application/json'})
        with urlopen(req, timeout=180) as resp:
            return json.loads(resp.read())['message']['content']

    async def _ollama_async(self, messages):
        async with self.llm_sem:
            loop = asyncio.get_running_loop()
            return await loop.run_in_executor(None, self._ollama, messages)

    # ── Phase 1: Generate docs ──────────────────────────────────────────

    async def generate_docs(self, rel_path, content):
        ext = Path(rel_path).suffix.lower().replace('.md', '')
        file_type = EXT_HINTS.get(ext, 'documentation page')
        filename = Path(rel_path).name

        if len(content) > MAX_SOURCE_CHARS:
            content = content[:MAX_SOURCE_CHARS] + "\n\n... [truncated]"

        prompt = f"""Analyze this {file_type} from "{self.project_name}".

File: `{filename}`
Path: `{rel_path}`

=== CONTENT ===
{content}
=== END ===

Write a Markdown documentation note:
# {Path(rel_path).stem}
A one-line summary.

## Purpose
2-4 sentences about what this covers.

## Key Points
- Bullet list of main topics/functions/sections

## Dependencies / References
- Other pages or files it links to (use relative paths)

## Notes
Any architecture decisions, patterns, or important details.

Rules: under 400 words, proper Markdown, no code fences around the whole response.
Just output the note — no preamble."""

        try:
            return await self._ollama_async(
                [{"role": "user", "content": prompt}])
        except Exception as e:
            print(f"   [LLM-FAIL] {rel_path}: {e}")
            return None

    # ── Phase 2: Linkify ────────────────────────────────────────────────

    async def linkify_docs(self, rel_path, doc_text, all_files):
        file_list = "\n".join(f"- {f}" for f in sorted(all_files))

        prompt = f"""Edit this doc note for `{rel_path}` in "{self.project_name}".

Add Markdown links wherever it references another file below.
Format: [display text](relative/path/to/file.md)

=== CURRENT NOTE ===
{doc_text}
=== END NOTE ===

=== ALL FILES (link targets) ===
{file_list}
=== END FILES ===

Rules:
- Only link files in the list above
- Link filenames, paths, config references, imports
- Don't change structure or add sections
- Return the COMPLETE updated note"""

        try:
            return await self._ollama_async(
                [{"role": "user", "content": prompt}])
        except Exception as e:
            print(f"   [LINKIFY-FAIL] {rel_path}: {e}")
            return doc_text

    # ── Crawl ───────────────────────────────────────────────────────────

    async def process_url(self, crawler, url, depth, run_config):
        async with self.crawl_sem:
            clean = url.split('?')[0].split('#')[0].rstrip('/')
            if clean in self.visited or self.page_count >= self.max_pages:
                return
            self.visited.add(clean)

            rel_path = self.url_to_path(clean)

            # Resume logic
            if rel_path:
                dest = self.root_folder / rel_path
                if not dest.name.lower().endswith('.md'):
                    dest = dest.with_suffix(dest.suffix + '.md')
                if dest.exists() and dest.stat().st_size > 200:
                    if self.is_github and '/blob/' in clean:
                        return
                    if not self.is_github:
                        return

            short = clean
            if len(short) > 90:
                short = short[:87] + '...'
            print(f"\n[{self.page_count + 1}:d{depth}] {short}")

            try:
                result = await crawler.arun(url=clean, config=run_config)
                if not result.success:
                    print(f"   [SKIP] crawl failed")
                    return
                self.page_count += 1

                if rel_path:
                    dest = self.root_folder / rel_path
                    if not dest.name.lower().endswith('.md'):
                        dest = dest.with_suffix(dest.suffix + '.md')

                    if self.is_github:
                        raw = self.fetch_raw(clean)
                    else:
                        raw = self.extract_markdown(result)
                        if raw:
                            raw = self.clean_markdown(raw)

                    if raw and len(raw.strip()) > 50:
                        dest.parent.mkdir(parents=True, exist_ok=True)
                        print(f"   [LLM] {rel_path} ...")
                        docs = await self.generate_docs(rel_path, raw)
                        if docs:
                            ext = Path(rel_path).suffix.lower().replace('.md', '')
                            tag = ext.lstrip('.') if ext else 'page'
                            with open(dest, "w") as f:
                                f.write("---\n")
                                f.write(f"source: {clean}\n")
                                f.write(f"tags: [{tag}, {self.project_name}]\n")
                                f.write("phase: 1\n")
                                f.write("---\n\n")
                                f.write(docs)
                            print(f"   [OK] -> {rel_path}")
                    elif raw:
                        print(f"   [SKIP] too short ({len(raw)} chars)")
                    else:
                        print(f"   [SKIP] no content")

                if depth < self.max_depth:
                    links = getattr(result, 'links', {})
                    for link in links.get('internal', []):
                        href = link.get('href')
                        if href:
                            full = urljoin(clean, href)
                            full = full.split('?')[0].split('#')[0].rstrip('/')
                            if self.is_internal(full) and full not in self.visited:
                                asyncio.create_task(
                                    self.process_url(crawler, full, depth + 1, run_config))
            except Exception as e:
                print(f"   [ERR] {clean}: {e}")

    async def crawl_and_generate(self):
        browser_config = BrowserConfig(headless=True)
        run_config = CrawlerRunConfig(cache_mode=CacheMode.BYPASS)
        async with AsyncWebCrawler(config=browser_config) as crawler:
            await self.process_url(crawler, self.base_url, 0, run_config)
            while True:
                tasks = [t for t in asyncio.all_tasks()
                         if t.get_coro().__name__ == 'process_url' and not t.done()]
                if not tasks:
                    break
                await asyncio.sleep(1)

    async def linkify_all(self):
        md_files = sorted(self.root_folder.rglob("*.md"))
        if not md_files:
            return

        all_paths = [str(f.relative_to(self.root_folder)) for f in md_files]

        print(f"\n=== Phase 2: Linkifying {len(md_files)} docs ===")
        for i, md_file in enumerate(md_files):
            rel = str(md_file.relative_to(self.root_folder))
            print(f"\n[{i+1}/{len(md_files)}] {rel} ...")

            text = md_file.read_text()

            if text.startswith('---\n'):
                end = text.index('\n---\n', 3)
                frontmatter = text[4:end]
                body = text[end + 5:]
            else:
                frontmatter = ""
                body = text

            linked_body = await self.linkify_docs(rel, body, all_paths)

            with open(md_file, "w") as f:
                f.write("---\n")
                f.write(frontmatter.replace("phase: 1", "phase: 2"))
                f.write("\n---\n\n")
                f.write(linked_body)

            print(f"   [OK]")


# ── CLI ──────────────────────────────────────────────────────────────────

async def main():
    test_mode = '--test' in sys.argv
    no_linkify = '--no-linkify' in sys.argv
    depth_flag = None
    for i, a in enumerate(sys.argv):
        if a.startswith('--depth='):
            depth_flag = int(a.split('=', 1)[1])
            break

    args = [a for a in sys.argv[1:]
            if a not in ('--test', '--no-linkify')
            and not a.startswith('--depth=')]

    if not args:
        print("Usage: python3 obsidian_crawler.py [--test] [--depth=N] [--no-linkify] <URL>")
        print("  --test        Limit to 8 pages")
        print("  --depth=N     Max crawl depth (default 5)")
        print("  --no-linkify  Skip Phase 2 cross-linking")
        sys.exit(1)

    url = args[0]
    vault = os.environ.get(
        "OBSIDIAN_VAULT_PATH",
        os.path.expanduser("~/Documents/Obsidian Vault"),
    )

    max_pages = 8 if test_mode else 500
    max_depth = depth_flag or 5
    mode_str = f"TEST ({max_pages}p)" if test_mode else "FULL"
    print(f"=== DocGenerator [{mode_str}] depth={max_depth} ===")
    print(f"URL:    {url}")
    print(f"Vault:  {vault}")
    print(f"LLM:    {OLLAMA_MODEL} @ {OLLAMA_BASE}")

    gen = DocGenerator(url, vault, max_depth=max_depth, max_pages=max_pages,
                       crawl_concurrency=3)
    await gen.crawl_and_generate()

    if not no_linkify:
        await gen.linkify_all()

    print(f"\n=== Done: {gen.page_count} pages, {len(gen.visited)} URLs ===")


if __name__ == "__main__":
    asyncio.run(main())
