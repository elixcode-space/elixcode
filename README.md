# elixcode

**elixcode/** — Rust CLI repository for the ElixCode platform.

 🌐 **Homepage:** [https://elixcode.space](https://elixcode.space) · 🐙 **GitHub:** [https://github.com/elixcode-space/elixcode](https://github.com/elixcode-space/elixcode)

## What is ElixCode?

ElixCode is an Agentic AI coding agent platform that combines:

- **LLM Gateway** — Route to 400+ OpenRouter models + self-hosted inference
- **Agentic TUI** — Interactive terminal interface (Claude Code-style)
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

This repository contains:
- **Rust CLI** — the primary end-user CLI binary
- **JS CLI** — TypeScript source for Node.js, Deno, and Bun variants (in `js/`)

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

### Download Release Binary

Download the appropriate binary for your platform from [GitHub Releases](https://github.com/elixcode-space/elixcode/releases/latest):

| Platform | File |
|----------|------|
| macOS (ARM) | `elixcode-aarch64-apple-darwin.tar.gz` |
| macOS (Intel) | `elixcode-x86_64-apple-darwin.tar.gz` |
| Linux (ARM) | `elixcode-aarch64-unknown-linux-gnu.tar.gz` |
| Linux (Intel) | `elixcode-x86_64-unknown-linux-gnu.tar.gz` |
| Windows | `elixcode-x86_64-pc-windows-msvc.zip` |

### Build from Source (Rust)

```bash
# Prerequisites: Rust toolchain (rustup)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Build
cargo build --release
# binary at target/release/elixcode
```

### JavaScript Variants

This repository also contains TypeScript source for Node.js, Deno, and Bun
variants of the same CLI in the `js/` directory. See [JS CLI](#javascript-cli-variants) for details.

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

| Command | Description |
|---------|-------------|
| `chat` | Interactive streaming chat session |
| `ask <prompt>` | Single non-interactive prompt |
| `edit <file> <description>` | Edit a file with agent assistance |
| `run <task>` | Run an autonomous agent task |
| `models` | List available LLM models |
| `harnesses ls` | List agent harness configs |
| `loops patterns` | List loop engineering patterns |
| `sessions ls` | List saved sessions |
| `fleet workers` | List fleet workers |
| `obs agents` | Monitor local coding agents |
| `auth login` | Authenticate with API key |
| `config` | Show/set configuration |
| `health` | Check gateway health |

### chat

```bash
elixcode chat                        # default model
elixcode chat --model claude-3-5-sonnet
elixcode chat --session <id>         # resume a saved session
elixcode chat --plan                 # plan-only mode (no file edits)
```

### ask

```bash
elixcode ask "what does 2+2 equal?"
```

### run

```bash
elixcode run "implement OAuth2 login" --agent coder --max-iter 100
```

### sessions

```bash
elixcode sessions list
elixcode sessions show <id>
elixcode sessions delete <id>
elixcode sessions export <id> --format json
```

### harnesses

```bash
elixcode harnesses ls
elixcode harnesses get claude
```

### loops

```bash
elixcode loops patterns
elixcode loops start daily-triage --interval 120000
```

### fleet

```bash
elixcode fleet workers
elixcode fleet hardware
elixcode fleet metrics
```

### obs / top

```bash
elixcode obs agents
elixcode obs summary
elixcode obs report --since 7d --by model
elixcode monitor dashboard
```

### auth

```bash
elixcode auth login
elixcode auth logout
elixcode auth whoami
```

### config

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

## JavaScript CLI Variants (Node.js, Deno, Bun)

This repo also contains TypeScript source for Node.js, Deno, and Bun variants of
the same CLI in the `js/` directory. All three runtimes share a single codebase;
only the entry point and packaging configuration differ.

### Runtime Variants

| Runtime | Entry point | Package | Registry |
|---------|-------------|---------|----------|
| Node.js | `src/cli-entry.ts` | `elixcode` | npm |
| Deno | `src/cli-deno.ts` | `@elixcode/elixcode` | JSR |
| Bun | `src/cli-entry.ts` | `@elixcode/elixcode-bun` | npm |

### Quick Start (JS CLIs)

```bash
export ELIXCODE_API_KEY="your-api-key"

# Node.js
cd js && npm install && npm run build
npx elixcode chat
npx elixcode ask "what does 2+2 equal?"

# Deno
deno run -A src/cli-deno.ts chat
deno run -A @elixcode/elixcode chat

# Bun
bun install
bun run src/cli-entry.ts chat
```

### Build (JS CLIs)

```bash
# Node.js / npm
cd js && npm install && npm run build

# Deno
deno run -A src/cli-deno.ts

# Bun
bun install && bun run src/cli-entry.ts
```

### Commands (JS CLIs)

All runtimes support the same command surface: `chat`, `ask`, `models`,
`harnesses ls`, `loops patterns`, `sessions ls`, `fleet workers`, `obs agents`,
`login [key]`, `config`, `health`.

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

## License

MIT License — see [LICENSE](LICENSE).