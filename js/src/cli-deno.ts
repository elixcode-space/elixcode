/// <reference lib="deno.ns" />
import { Client } from './lib/api-deno.ts';
import { green, red, cyan, gray, bold } from './lib/colors.ts';
import { commands, cmd } from './lib/registry.ts';

const server = Deno.env.get("ELIXCODE_SERVER") || "https://api.elixcode.space";
const apiKey = Deno.env.get("ELIXCODE_API_KEY");
const client = new Client(server, apiKey);

cmd("config", "Show configuration");
cmd("login", "Login with API key to get JWT");
cmd("health", "Check gateway health");
cmd("models", "List available models");
cmd("ask", "Ask a single question", 1);
cmd("harnesses", "List harnesses");
cmd("loops", "List loop patterns");
cmd("sessions", "List sessions");
cmd("fleet", "List fleet workers");
cmd("obs", "Show agent summary");

commands["config"].action = async () => {
  console.log(`Server:     ${cyan(client.server)}`);
  console.log(`API Key:    ${apiKey ? green("***set***") : gray("not set")}`);
  console.log(`Token:      ${client.getToken() ? green("***set***") : gray("not set")}`);
};

commands["login"].action = async (args: string[]) => {
  const key = args[0] || client.apiKey;
  if (!key) {
    console.error(red("API key required"));
    Deno.exit(1);
  }
  const response = await client.login(key);
  console.log(green("Logged in successfully"));
  console.log(`JWT expires at: ${response.expires_at}`);
};

commands["health"].action = async () => {
  const result = await client.health();
  console.log(JSON.stringify(result, null, 2));
};

commands["models"].action = async () => {
  const models = await client.listModels();
  console.log(cyan(`Models (${models.length}):`));
  for (const m of models) {
    console.log(`  ${bold(m.id)} — ${m.provider}`);
  }
};

commands["ask"].action = async (args: string[], opts?: Record<string, unknown>) => {
  const input = args[0];
  if (!input) {
    console.error(red("Input required"));
    Deno.exit(1);
  }
  const model = (opts?.model as string) || "gpt-4o";
  const messages = [{ role: "user", content: input }];
  const result = await client.chat(model, messages);
  const msg = (result as { choices?: { message?: { content?: string } }[] }).choices?.[0]?.message?.content || "";
  console.log(msg);
};

commands["harnesses"].action = async () => {
  const harnesses = await client.listHarnesses();
  console.log(cyan(`Harnesses (${harnesses.length}):`));
  for (const h of harnesses) {
    console.log(`  ${bold(h.id)} — ${h.base}`);
  }
};

commands["loops"].action = async () => {
  const patterns = await client.listPatterns();
  console.log(cyan("Loop Patterns:"));
  for (const p of patterns) {
    console.log(`  ${bold(p.name)} [${p.tier}] — ${p.cadence}`);
  }
};

commands["sessions"].action = async () => {
  const sessions = await client.listSessions();
  console.log(cyan(`Sessions (${sessions.length}):`));
  for (const s of sessions) {
    console.log(`  ${bold(s.id)} — ${s.model}`);
  }
};

commands["fleet"].action = async () => {
  const workers = await client.fleetWorkers();
  console.log(cyan(`Workers (${workers.length}):`));
  for (const w of workers) {
    console.log(JSON.stringify(w));
  }
};

commands["obs"].action = async () => {
  const s = await client.agentSummary();
  console.log(`Agents: ${s.total_agents}, Tokens: ${s.total_tokens}`);
};

if (import.meta.main) {
  const args = Deno.args;
  if (args.length === 0) {
    console.log("elixcode v0.9.2 — Elixcode user CLI (Deno)");
    console.log(gray("\nCommands:"));
    for (const [name, def] of Object.entries(commands)) {
      console.log(`  ${cyan(name.padEnd(12))} ${def.description}`);
    }
    console.log(gray("\nSet ELIXCODE_API_KEY env var for authentication."));
    Deno.exit(0);
  }

  const cmdName = args[0];
  const c = commands[cmdName];
  if (!c) {
    console.error(red(`Unknown command: ${cmdName}`));
    console.log("Run without arguments for help.");
    Deno.exit(1);
  }

  const restArgs = args.slice(1);
  const opts: Record<string, unknown> = {};
  const positional: string[] = [];

  for (const arg of restArgs) {
    if (arg.startsWith("--")) {
      const [key, val] = arg.slice(2).split("=");
      opts[key] = val ?? true;
    } else {
      positional.push(arg);
    }
  }

  await c.action(positional, opts);
}
