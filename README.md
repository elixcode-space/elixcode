# elixcode

**elixcode/** — Rust CLI repository for the ElixCode platform.

 🌐 **Homepage:** [https://elixcode.space](https://elixcode.space) · 🐙 **GitHub:** [https://github.com/elixcode-space/elixcode](https://github.com/elixcode-space/elixcode)

## What is ElixCode?

ElixCode is an Agentic AI coding agent platform that combines:
- **LLM Gateway** — Route to 400+ OpenRouter models + self-hosted inference
- **Agentic TUI** — Interactive terminal interface
- **Fleet orchestration** — Concurrent multi-agent workflows
- **Loop Engineering** — Automated repeated tasks (daily triage, CI sweeping)
- **Sandbox execution** — Secure code execution
- **Skills system** — Learnable domain knowledge
- **Observability** — Token cost tracking, agent monitoring

## Overview

`elixcode` is the end-user agentic CLI for the ElixCode platform. It provides an
interactive TUI, single-prompt agentic mode, model listing, harness management,
loop engineering, and more — all connecting to the public gateway at
`https://api.elixcode.space` by default.

## Installation

### Homebrew (macOS + Linux)

```bash
brew tap elixcode-space/homebrew-tap
brew install elixcode
```

### Scoop (Windows)

```powershell
scoop bucket add elixcode https://github.com/elixcode-space/scoop-bucket
scoop install elixcode
```

### Build from Source

```bash
cargo build --release
# binary at target/release/elixcode
```

### JavaScript Variants

This repository also contains TypeScript source for Node.js, Deno, and Bun
variants of the same CLI in the `js/` directory. See `js/README.md` for details.

## Quick Start

```bash
# Get an API key from https://api.elixcode.space
export ELIXCODE_API_KEY="your-api-key"

# Interactive chat (Claude Code style)
elixcode chat

# Single prompt
elixcode ask "explain Elixir GenServers"

# Use a specific model
elixcode ask --model claude-3-5-sonnet "write a Go HTTP server"

# Run an agent task
elixcode run "add a README to this repo" --agent coder --max-iter 50
```

## Commands

### chat
Start an interactive streaming chat session in the TUI.

```bash
elixcode chat                        # default model
elixcode chat --model claude-3-5-sonnet
elixcode chat --session <id>         # resume a saved session
elixcode chat --plan                 # plan-only mode (no file edits)
```

### ask
Send a single non-interactive prompt.

```bash
elixcode ask "what does 2+2 equal?"
```

### edit
Edit a file with agent assistance (Claude Code-style).

```bash
elixcode edit src/main.rs "add error handling"
```

### run
Run an autonomous agent task with a specified profile.

```bash
elixcode run "implement OAuth2 login" --agent coder --max-iter 100
```

### sessions
Manage saved chat sessions.

```bash
elixcode sessions list
elixcode sessions show <id>
elixcode sessions delete <id>
elixcode sessions export <id> --format json
```

### models
List available LLM models.

```bash
elixcode models
elixcode models --provider openrouter
```

### harnesses
Manage agent harness configurations.

```bash
elixcode harnesses ls
elixcode harnesses get claude
```

### loops
Manage automated loop engineering.

```bash
elixcode loops patterns
elixcode loops start daily-triage --interval 120000
```

### fleet
Manage fleet workers and hardware.

```bash
elixcode fleet workers
elixcode fleet hardware
elixcode fleet metrics
```

### obs / top
Monitor local coding agents (requires [agent-top](https://github.com/kannandreams/agent-top)).

```bash
elixcode obs agents
elixcode obs summary
elixcode obs report --since 7d --by model
elixcode monitor dashboard
```

### auth
Manage authentication.

```bash
elixcode auth login
elixcode auth logout
elixcode auth whoami
```

### config
Manage configuration.

```bash
elixcode config show
elixcode config set model gpt-4o
```

## Global Options

| Flag | Env | Description |
|------|-----|-------------|
| `--server <url>` | `ELIXCODE_SERVER` | Gateway URL |
| `--api-key <key>` | `ELIXCODE_API_KEY` | API key for auth |
| `--model <name>` | `ELIXCODE_MODEL` | Default model |
| `--workspace <dir>` | `ELIXCODE_WORKSPACE` | Working directory |
| `--json` | — | Output as JSON |
| `-v, --verbose` | — | Debug logging |

## Public API Endpoints

The gateway runs at `https://api.elixcode.space`. Common endpoints:

```bash
# Health
curl https://api.elixcode.space/health

# List models
curl https://api.elixcode.space/v1/models

# Chat completion
curl -X POST https://api.elixcode.space/v1/chat/completions \
  -H "Authorization: Bearer $ELIXCODE_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt-4o","messages":[{"role":"user","content":"Hello"}]}'

# OIDC discovery
curl https://api.elixcode.space/.well-known/openid-configuration

# OpenAPI spec
curl https://api.elixcode.space/openapi.json

# Interactive docs
open https://api.elixcode.space/docs
```

## JavaScript CLI Variants

This repo also contains TypeScript source for Node.js, Deno, and Bun in `js/`:

```bash
# Node.js
cd js && npm install && npm run build
npx elixcode chat

# Deno
deno run -A js/src/cli-deno.ts chat

# Bun
cd js && bun install
bun run src/cli-entry.ts chat
```

| Runtime | Package | Registry |
|---------|---------|----------|
| Node.js | `elixcode` | npm |
| Deno | `@elixcode/elixcode` | JSR |
| Bun | `@elixcode/elixcode-bun` | npm |

## License

MIT License — see [LICENSE](LICENSE).
