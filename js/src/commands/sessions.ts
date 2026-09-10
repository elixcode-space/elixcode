import { Client } from '../lib/api.ts';
import { Command } from 'commander';
import pc from 'picocolors';

export function setupSessions(program: Command, client: Client): void {
  const session = program.command('sessions').description('Manage sessions');

  session
    .command('ls')
    .description('List sessions')
    .action(async () => {
      const sessions = await client.listSessions();
      console.log(pc.cyan(`Sessions (${sessions.length}):`));
      for (const s of sessions) {
        console.log(`  ${pc.bold(s.id)} — ${s.model} (${s.message_count} messages)`);
      }
    });

  session
    .command('save [id]')
    .description('Save session')
    .action(async (id?: string) => {
      const result = await client.saveSession(id);
      console.log(pc.green(`Saved: ${result.id}`));
    });

  session
    .command('show <id>')
    .description('Show a session')
    .action(async (id: string) => {
      const s = await client.loadSession(id);
      console.log(JSON.stringify(s, null, 2));
    });

  session
    .command('export <id>')
    .description('Export session')
    .option('-f, --format <fmt>', 'Format: json or md', 'json')
    .action(async (id: string, opts) => {
      const result = await client.exportSession(id, opts.format as 'json' | 'md');
      console.log(JSON.stringify(result, null, 2));
    });

  session
    .command('rm <id>')
    .description('Delete session')
    .action(async (id: string) => {
      await client.deleteSession(id);
      console.log(pc.gray(`Deleted session ${id}.`));
    });
}
