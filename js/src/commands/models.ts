import { Client, APIError } from '../lib/api.ts';
import { Command } from 'commander';
import pc from 'picocolors';

export function setupModels(program: Command, client: Client): void {
  program
    .command('models')
    .description('List available models and providers')
    .option('-p, --providers', 'Show providers instead of models')
    .action(async (opts) => {
      if (opts.providers) {
        const providers = await client.listProviders();
        console.log(pc.cyan('Providers:'));
        for (const p of providers) {
          console.log(`  ${pc.bold(p.name)}: ${p.models.join(', ')}`);
        }
      } else {
        const models = await client.listModels();
        console.log(pc.cyan(`Models (${models.length}):`));
        for (const m of models) {
          const price = m.pricing
            ? ` ($${(m.pricing.prompt * 1000).toFixed(3)}/$${(m.pricing.completion * 1000).toFixed(3)})`
            : '';
          console.log(`  ${pc.bold(m.id)} — ${m.provider}${price}`);
        }
      }
    });
}

export function setupAuth(program: Command, client: Client): void {
  program
    .command('login')
    .description('Login with API key to get JWT')
    .argument('[apiKey]', 'API key')
    .action(async (apiKey?: string) => {
      const key = apiKey || client.apiKey;
      if (!key) {
        console.error(pc.red('API key required — set ELIXCODE_API_KEY or pass as argument'));
        process.exit(1);
      }
      const response = await client.login(key);
      console.log(pc.green('Logged in successfully'));
      console.log(`JWT expires at: ${response.expires_at}`);
      if (response.refresh_token) {
        console.log(pc.gray('Refresh token saved.'));
      }
    });

  program
    .command('logout')
    .description('Clear stored JWT')
    .action(() => {
      client.setToken(null);
      console.log(pc.gray('Logged out.'));
    });
}

export function setupConfig(program: Command, client: Client): void {
  program
    .command('config')
    .description('Show configuration')
    .action(() => {
      console.log(`Server:     ${pc.cyan(client.server)}`);
      console.log(`API Key:    ${client.apiKey ? pc.green('***set***') : pc.gray('not set')}`);
      console.log(`Token:      ${client.getToken() ? pc.green('***set***') : pc.gray('not set')}`);
    });

  program
    .command('config set <key> <value>')
    .description('Set a configuration value')
    .action((key: string, value: string) => {
      const envMap: Record<string, string> = {
        server: 'ELIXCODE_SERVER',
        api_key: 'ELIXCODE_API_KEY',
        model: 'ELIXCODE_MODEL',
      };
      const env = envMap[key] || key.toUpperCase();
      console.log(`Set ${env}=${value} as environment variable.`);
      console.log(pc.gray(`export ${env}="${value}"`));
    });
}
