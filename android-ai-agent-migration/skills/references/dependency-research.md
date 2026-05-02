# Dependency Research Workflow

When documenting a repo (especially a spec/design doc), researching and pinning current versions of all dependencies adds significant value. This workflow emerged from the `android-ai-app` session where we pinned 20+ dependency versions across Rust, Kotlin, and external APIs.

## When to Use

- Spec/architecture repos that list technologies but not specific versions
- Implementation-plan repos where version accuracy matters for build files
- Any repo documentation where the user says "pin down all versions"

## Workflow

### Phase 1: Inventory

Extract every dependency mentioned in the spec/repo:
- Language libraries (crates, Maven artifacts, npm packages)
- External services/APIs (endpoints, pricing models)
- Platform/built-in components (Android SDK, system services)

### Phase 2: Live Search

For each dependency, run `web_search` targeting the canonical source:
```
# Rust crates
web_search("<crate-name> crate latest version crates.io 2026")

# Android/Kotlin
web_search("<artifact> latest stable version 2026 developer.android.com")

# Maven Central
web_search("<artifact> latest version 2026 maven central")

# External APIs
web_search("<api-name> API pricing documentation 2026")
```

### Phase 3: Doc Extraction

For key dependencies, use `web_extract` to pull API reference from docs.rs:
```
web_extract(urls=["https://docs.rs/<crate>/latest/<crate>/"])
```

Extract: key types, feature flags, usage examples, module structure.

### Phase 4: Write Manifest

Consolidate into a single `Dependencies.md` note with these sections:
1. **Android/Kotlin Dependencies** — artifact, version, release date, status, `build.gradle.kts` snippet
2. **Rust Dependencies** — crate, version, MSRV, release date, `Cargo.toml` snippet
3. **External Services** — endpoint, auth, cost model, rate limits
4. **Crate Docs (Key API Reference)** — extracted docs for the 3-5 most critical crates with code examples
5. **Quick Reference** — copy-pasteable version blocks for both build systems

### Phase 5: Cross-link

Link the dependencies note from the master index via `[[wikilinks]]`.

## Example Output

From `android-ai-app`: a 360-line dependencies note covering:
- Jetpack Compose BOM `2026.04.01` (1.11.0 stable)
- Kotlin Coroutines 1.10.2
- Room 2.8.4, OkHttp 5.3.2, CameraX 1.6.0
- Rust: tokio 1.52.1, reqwest 0.13.3, serde 1.0.228, jni 0.22.4, toml 1.1.2
- OpenRouter pricing, Websurfx (self-hosted search)
- Extracted docs.rs API references with code examples for reqwest, tokio, jni, serde, toml

## Pitfalls

- **Stale results**: Some search results lag by days. Cross-check against the official release page (GitHub releases, Maven Central metadata, crates.io version history).
- **Edition/API compatibility**: Rust edition 2024 is very new — check MSRV requirements for each crate.
- **Missing deps**: Specs often omit transitive/platform deps. Check the spec's module list for crates that aren't in the dependency table.
