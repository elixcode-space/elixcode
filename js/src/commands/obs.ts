import { Client } from '../lib/api.ts';
import { Command } from 'commander';
import pc from 'picocolors';

export function setupObs(program: Command, client: Client): void {
  const obs = program.command('obs').description('Observability (agent-top)');

  obs
    .command('agents')
    .description('List all local agents')
    .action(async () => {
      const agents = await client.agentAgents();
      console.log(pc.cyan(`Agents (${(agents as unknown[]).length}):`));
      for (const a of agents as unknown[]) {
        console.log(JSON.stringify(a));
      }
    });

  obs
    .command('summary')
    .description('Show agent summary')
    .action(async () => {
      const s = await client.agentSummary();
      console.log(`Total agents:   ${pc.bold(s.total_agents.toString())}`);
      console.log(`Total sessions: ${s.total_sessions}`);
      console.log(`Total tokens:   ${s.total_tokens.toLocaleString()}`);
      console.log(`Total cost:     $${s.total_cost_usd.toFixed(4)}`);
      console.log(`Orphaned MCP:   ${s.orphaned_mcp}`);
    });

  obs
    .command('report')
    .description('Cost/token report')
    .option('-s, --since <period>', 'Time period', '7d')
    .option('-b, --by <field>', 'Group by', 'model')
    .option('-j, --json', 'JSON output')
    .action(async (opts) => {
      const report = await client.agentReport(opts.since, opts.by);
      if (opts.json) {
        console.log(JSON.stringify(report, null, 2));
      } else {
        console.log(pc.cyan('Report:'));
        console.log(JSON.stringify(report, null, 2));
      }
    });

  obs
    .command('trace <id>')
    .description('Trace a session')
    .action(async (id: string) => {
      const trace = await client.agentTrace(id);
      console.log(JSON.stringify(trace, null, 2));
    });

  obs
    .command('prices')
    .description('Show model pricing')
    .action(async () => {
      const models = await client.listModels();
      console.log(pc.cyan('Pricing:'));
      for (const m of models) {
        if (m.pricing) {
          console.log(`  ${m.id}: $${(m.pricing.prompt * 1000).toFixed(3)}/1k prompt`);
        }
      }
    });
}
