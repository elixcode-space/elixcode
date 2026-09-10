# @elixcode/elixcode-js — Unified JS CLI

Source code for the TypeScript/Node.js, Deno, and Bun variants of the Elixcode
user CLI. All three runtimes share the same source code; only the entry point
and packaging configuration differ.

## Runtime Variants

| Runtime | Entry point | Package | Registry |
|---------|-------------|---------|----------|
| Node.js | `src/cli-entry.ts` | `elixcode` | npm |
| Deno | `src/cli-deno.ts` | `@elixcode/elixcode` | JSR |
| Bun | `src/cli-entry.ts` | `@elixcode/elixcode-bun` | npm |

## Build

### Node.js / npm

```bash
npm install
npm run build   # outputs to dist/
npm link        # or: npm install -g .
```

### Deno

```bash
deno run -A src/cli-deno.ts
# or install:
deno install -A src/cli-deno.ts
```

### Bun

```bash
bun install
bun run src/cli-entry.ts
# or: bun build src/cli-entry.ts --outdir dist --format=esm
```

## Usage

```bash
export ELIXCODE_API_KEY="your-api-key"

# Node.js
npx elixcode chat
npx elixcode ask "what does 2+2 equal?"

# Deno
deno run -A elixcode chat
deno run -A @elixcode/elixcode ask "what does 2+2 equal?"

# Bun
bunx elixcode-bun chat
```

## Commands

All runtimes support the same command surface:

- `chat` — interactive streaming chat
- `ask <prompt>` — single question
- `models` — list models
- `harnesses ls` — list agent harnesses
- `loops patterns` — list loop patterns
- `sessions ls` — list sessions
- `fleet workers` — list fleet workers
- `obs agents` — monitor local agents
- `login [key]` — authenticate
- `config` — show configuration
- `health` — check gateway health

## License

MIT
