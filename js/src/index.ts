import { Client } from './lib/api.ts';
import { setupChat } from './commands/chat.ts';
import { setupModels, setupAuth, setupConfig } from './commands/models.ts';
import { setupHarnesses } from './commands/harnesses.ts';
import { setupLoops } from './commands/loops.ts';
import { setupSessions } from './commands/sessions.ts';
import { setupFleet } from './commands/fleet.ts';
import { setupObs } from './commands/obs.ts';
import { Command } from 'commander';

const server = process.env.ELIXCODE_SERVER || 'https://api.elixcode.space';
const apiKey = process.env.ELIXCODE_API_KEY;

const client = new Client(server, apiKey);

const program = new Command();

program
  .name('elixcode')
  .description('Elixcode — OpenCode alternative (TypeScript CLI)')
  .version('0.9.2');

setupConfig(program, client);
setupAuth(program, client);
setupChat(program, client);
setupModels(program, client);
setupHarnesses(program, client);
setupLoops(program, client);
setupSessions(program, client);
setupFleet(program, client);
setupObs(program, client);

program.parseAsync(process.argv).catch((err: Error) => {
  if (err.message && !err.message.includes('Aborted')) {
    console.error(err.message);
  }
  process.exit(1);
});
