import { Client } from '../lib/api.ts';
import { Command } from 'commander';
import pc from 'picocolors';

export function setupHarnesses(program: Command, client: Client): void {
  const harness = program.command('harnesses').description('Manage agent harnesses');

  harness
    .command('ls')
    .description('List all harnesses')
    .action(async () => {
      const harnesses = await client.listHarnesses();
      console.log(pc.cyan(`Harnesses (${harnesses.length}):`));
      for (const h of harnesses) {
        const status = h.enabled ? pc.green('enabled') : pc.gray('disabled');
        const inst = h.installed ? pc.green('installed') : pc.red('not installed');
        console.log(`  ${pc.bold(h.id)} — ${h.base} [${status}, ${inst}]`);
      }
    });

  harness
    .command('create <name> <base>')
    .description('Create a harness config')
    .action(async (name: string, base: string) => {
      const result = await client.createHarness({ name, base });
      console.log(pc.green(`Created harness: ${result.id}`));
    });

  harness
    .command('get <id>')
    .description('Get harness details')
    .action(async (id: string) => {
      const h = await client.getHarness(id);
      console.log(JSON.stringify(h, null, 2));
    });

  harness
    .command('install <id>')
    .description('Check installation status')
    .action(async (id: string) => {
      const status = await client.checkInstall(id);
      console.log(`Installed: ${status.installed}`);
    });
}
