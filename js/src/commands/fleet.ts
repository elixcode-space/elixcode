import { Client } from '../lib/api.ts';
import { Command } from 'commander';
import pc from 'picocolors';

export function setupFleet(program: Command, client: Client): void {
  const fleet = program.command('fleet').description('Manage fleet');

  fleet
    .command('workers')
    .description('List fleet workers')
    .action(async () => {
      const workers = await client.fleetWorkers();
      console.log(pc.cyan(`Workers (${workers.length}):`));
      for (const w of workers as unknown[]) {
        console.log(JSON.stringify(w));
      }
    });

  fleet
    .command('hardware')
    .description('Show hardware info')
    .action(async () => {
      const hw = await client.fleetHardware();
      console.log(JSON.stringify(hw, null, 2));
    });

  fleet
    .command('metrics')
    .description('Show fleet metrics')
    .action(async () => {
      const m = await client.fleetMetrics();
      console.log(JSON.stringify(m, null, 2));
    });
}
