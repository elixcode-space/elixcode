import { Client } from '../lib/api.ts';
import { Command } from 'commander';
import pc from 'picocolors';

export function setupLoops(program: Command, client: Client): void {
  const loop = program.command('loops').description('Manage automated loops');

  loop
    .command('patterns')
    .description('List loop patterns')
    .action(async () => {
      const patterns = await client.listPatterns();
      console.log(pc.cyan('Loop Patterns:'));
      for (const p of patterns) {
        console.log(`  ${pc.bold(p.name)} [${p.tier}] — ${p.cadence}`);
      }
    });

  loop
    .command('start <pattern>')
    .description('Start a loop')
    .option('-i, --interval <ms>', 'Interval in milliseconds', '60000')
    .action(async (pattern: string, opts) => {
      const result = await client.startLoop(pattern, parseInt(opts.interval));
      console.log(pc.green(`Loop started: ${result.id}`));
    });

  loop
    .command('stop <id>')
    .description('Stop a loop')
    .action(async (id: string) => {
      await client.stopLoop(id);
      console.log(pc.gray(`Loop ${id} stopped.`));
    });

  loop
    .command('status')
    .description('Show loop status')
    .action(async () => {
      const status = await client.loopStatus();
      console.log(JSON.stringify(status, null, 2));
    });

  loop
    .command('cost')
    .description('Show loop cost breakdown')
    .action(async () => {
      const cost = await client.loopCost();
      console.log(JSON.stringify(cost, null, 2));
    });
}
