<img src="https://r2cdn.perplexity.ai/pplx-full-logo-primary-dark%402x.png" style="height:64px;margin-right:32px"/>

# Self-Hosted Search Engines for Android On-Device AI Agent

> These run **on the Android phone itself** — no cloud, no server, no API keys. Cross-compiled Rust binaries or lightweight WASM.

For a SearXNG-style meta-search that's small, simple, and good for AI integration, the standout Rust option is **Websurfx**, with **LibreX** and **Whoogle** as lightweight non-Rust alternatives; for "search backend for your own data" in Rust, **Meilisearch** and **tinysearch** are worth a look.[^1][^2][^3][^4][^5][^6]

Below is a quick breakdown, focusing on “small/simple, self-hosted, AI-harness-friendly”, and calling out Rust or easy deployment where possible.

## Rust meta-search engines

### Websurfx (Rust, Actix-Web)

- **What it is:** A self-hostable meta-search engine written entirely in Rust and built on Actix-Web, explicitly positioned as an open‑source alternative to Searx/SearXNG.[^7][^5][^1]
- **Why it fits you:**
    - Rust, async (Actix), privacy-focused meta-search (aggregates other engines, no ads, user privacy emphasized).[^3][^5]
    - Designed to be **lightning‑fast** and “extremely fast,” with multiple caching layers and compression/encryption options for performance and privacy.[^5][^3]
    - Docker images and bare-metal install options make it easy to drop into an existing stack; there are ready-made docker-compose examples.[^3][^5]
    - Config is in **Lua**, giving you a programmable config surface that’s nice if you want to adapt behavior per environment or device.[^7]

**AI harness integration:**

- Presents a normal HTTP web search endpoint (like SearXNG) so an AI agent can just call `GET /search?...` and scrape results or consume whatever response format you choose. The project advertises “organic search ranking” and advanced filtering, which you can surface to your agent via query params.[^5][^3]

If you specifically want “SearXNG but Rust”, this is the closest thing right now.

## Lightweight meta-search (non‑Rust but simple)

### LibreX

- **What it is:** A **privacy‑respecting meta search engine framework**, focused on operating **without JavaScript**, aggregating results from Google, Qwant, Ahmia and some torrent sources.[^4][^6][^8]
- **Why it fits you:**
    - Explicitly marketed as “framework and javascript free privacy respecting meta search engine,” which keeps the front‑end minimal and light.[^6][^4]
    - Has an official Docker image with a single container (`librex/librex`) that you can run with one `docker run` line or a tiny `docker-compose` file.[^9]
    - Highly configurable via environment variables (Google domain, language, OpenSearch metadata, etc.), which is nice for tuning your AI harness or client behavior.[^9]

**AI harness integration:**

- It exposes a standard HTTP search endpoint; there’s also OpenSearch metadata (title, description, host) configured via env vars, which is a hint that it’s meant to be easily integrated as a programmable search provider.[^9]


### Whoogle Search

- **What it is:** A self-hosted, ad‑free, privacy‑respecting meta-search engine that proxies **Google** results and strips out ads, JavaScript, AMP links, cookies, and IP-based tracking.[^10][^11][^12][^13][^14]
- **Why it fits you:**
    - Simple architecture: one container (`benbusby/whoogle-search`) with a small config, very easy to spin up and put behind a reverse proxy.[^11][^13][^14]
    - Minimal UI, no JS on the client side, and a focus on just returning clean search results, which keeps it lighter than full-blown portals.[^10][^11]
- **AI harness integration:**
    - Like SearXNG, it’s just an HTTP search endpoint (e.g., `/search?q=...`), so your agent can call it and parse the HTML for links/snippets. Guides emphasize using it as a drop‑in “default search engine,” which is exactly the interface an AI harness can drive programmatically.[^12][^11]


### SearXNG itself (baseline)

- You already know SearXNG, but worth noting: it’s still one of the better **“meta-search as a service for agents”** options, with a JSON API and even **MCP server / JSON API** integration highlighted in recent walkthroughs.[^15][^16]
- If you’re okay with Python, SearXNG may still be the most mature choice, and you can reserve Websurfx/LibreX/Whoogle as “lighter or Rustier” alternatives.


## Rust search backends for your own data

If you also want small Rust search engines to index local docs/logs for RAG, rather than meta-searching the public web:

### Meilisearch

- **What it is:** An open-source, full-text search engine written in Rust; designed to be “ultra relevant, instant and typo-tolerant,” usually used as an API-first search backend for applications.[^17][^18][^19]
- **Why it fits you:**
    - Rust, simple HTTP+JSON API, self-hostable; geared toward fast, low-latency search over your own index rather than the web.[^18][^19]
    - Runs as a single binary/service, which is small and straightforward compared to Elastic-style stacks.[^19][^18]


### tinysearch (Rust + WebAssembly, static)

- **What it is:** A tiny full-text search engine intended mainly for static sites, implemented in Rust and compiled to WebAssembly; the whole search bundle is about 50–100 kB gzipped.[^2][^20]
- **Why it fits you:**
    - Very small footprint and **completely static**—you generate a compact index and ship it alongside your site or app.[^2]
    - Good if you want an **offline, client-side** search for a small corpus (docs, notes) that your AI agent can read locally instead of making network calls.[^20][^2]


## Non‑Rust but extremely lightweight backends

### ZincSearch (Go)

- **What it is:** A lightweight, resource‑minimal alternative to Elasticsearch written in Go that does full‑text indexing and search.[^21][^17][^18]
- **Why it fits you:**
    - Designed specifically to be **minimal-resource**, making it attractive on small self-hosted boxes when Elastic is overkill.[^21][^17][^18]
    - Simple API and schema make it a decent candidate as a log/document search backend for an AI harness.


## Integration tips for an AI harness

- **HTTP-first:** All of these expose regular HTTP endpoints (some with JSON, some primarily HTML), so your harness can just call them over HTTP and parse either JSON or HTML to extract titles, URLs, and snippets.[^11][^3][^5][^9]
- **Preferred for JSON/API-style use:**
    - **SearXNG** (mature JSON API, MCP integration already being used in the wild).[^16][^15]
    - **Websurfx** and **LibreX** are meta-search engines designed to be self-hosted and configurable, with environment or Lua-configurable behavior that you can tune for agent use.[^4][^7][^3][^5][^9]
    - **Meilisearch** is ideal for your own content: Rust, HTTP+JSON, and easy to drive from any language.[^18][^19]

If you tell me whether you care more about “web metasearch” vs “local corpus search for RAG,” I can sketch a minimal docker-compose (or bare binary) setup and a small Rust/Python harness snippet to call one of these on ARM-friendly hardware.
<span style="display:none">[^22][^23][^24][^25][^26][^27][^28][^29]</span>

<div align="center">⁂</div>

[^1]: https://www.reddit.com/r/selfhosted/comments/1cinrk0/websurfx_a_selfhostable_meta_search_engine/

[^2]: https://endler.dev/2019/tinysearch/

[^3]: https://awesome-docker-compose.com/websurfx

[^4]: https://hostedsoftware.org/tools/librex/

[^5]: https://github.com/neon-mmd/websurfx

[^6]: https://github.com/hnhx/librex

[^7]: https://www.reddit.com/r/selfhosted/comments/134wm9k/websurfx_vs_searx_vs_searxng_comparison_of_the/

[^8]: https://forum.cloudron.io/topic/9247/librex-framework-and-javascript-free-privacy-respecting-meta-search-engine

[^9]: https://hub.docker.com/r/librex/librex

[^10]: https://www.youtube.com/watch?v=aq3mZrDbbYQ

[^11]: https://github.com/benbusby/whoogle-search

[^12]: https://zeabur.com/templates/8TBB9V

[^13]: https://labs.newpush.com/applications/whoogle.html

[^14]: https://sourceforge.net/projects/whoogle-search.mirror/

[^15]: https://www.youtube.com/watch?v=9pNMvfwQYE4

[^16]: https://akashrajpurohit.com/blog/selfhost-searxng-for-privacy-focused-search/

[^17]: https://www.facts.dev/self-hosted/search-engines/1/

[^18]: https://www.devopsschool.com/blog/list-of-top-free-open-source-self-hosted-application-for-search-engines/

[^19]: https://meilisearch.com/docs/resources/comparisons/alternatives

[^20]: https://news.ycombinator.com/item?id=23473303

[^21]: https://github.com/zincsearch/zincsearch

[^22]: https://alternativeto.net/software/searxng/?license=opensource

[^23]: https://dev.to/0thtachi/build-a-fast-and-lightweight-rust-vector-search-app-with-rig-lancedb-57h2

[^24]: https://www.xda-developers.com/replaced-google-with-self-hosted-search-aggregator-never-going-back/

[^25]: https://alternativeto.net/software/searxng/?feature=web-search\&license=opensource\&p=2

[^26]: https://jdrouet.github.io/posts/202503161800-search-engine-intro/

[^27]: https://github.com/mikeroyal/Self-Hosting-Guide

[^28]: https://www.reddit.com/r/rust/comments/12v5kpw/i_am_writing_a_meta_search_engine_in_rust/

[^29]: https://oneuptime.com/blog/post/2026-02-20-rust-web-server-actix/view

