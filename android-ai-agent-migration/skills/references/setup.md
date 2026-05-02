# Machine Setup for obsidian-github-docgen

## Prerequisites

### 1. Python Virtual Environment
```bash
uv venv /home/$USER/crawl-venv
source /home/$USER/crawl-venv/bin/activate
uv pip install crawl4ai
```

### 2. Playwright Browsers
```bash
source /home/$USER/crawl-venv/bin/activate
playwright install chromium
```

### 3. Ollama (Local LLM)
```bash
# Install
curl -fsSL https://ollama.com/install.sh | sh

# Pull model
ollama pull gemma4:e4b

# Verify
curl http://localhost:11434/api/tags
```

If Ollama runs on a different machine, set `OLLAMA_HOST`:
```bash
export OLLAMA_HOST=http://100.70.230.26:11434
```

### 4. Obsidian Vault
```bash
mkdir -p "$HOME/Documents/Obsidian Vault"
# Or set custom path:
export OBSIDIAN_VAULT_PATH="/path/to/vault"
```

## Environment (add to ~/.hermes/.env)
```bash
OLLAMA_HOST=http://100.70.230.26:11434
OLLAMA_MODEL=gemma4:e4b
OBSIDIAN_VAULT_PATH=$HOME/Documents/Obsidian Vault
```

## Portability Notes

- The crawler script is self-contained (single Python file, stdlib + crawl4ai only)
- Ollama URL and model are configurable via environment variables
- Vault path defaults to `~/Documents/Obsidian Vault`
- All paths use `os.path.expanduser()` for portability
- Concurrency and page limits are tunable in the script
