import readline from 'node:readline';
import { Client, APIError } from '../lib/api.ts';
import { Command } from 'commander';
import ora from 'ora';
import pc from 'picocolors';

export function setupChat(program: Command, client: Client): void {
  program
    .command('chat')
    .description('Start an interactive chat session')
    .option('-m, --model <model>', 'Model to use', 'gpt-4o')
    .option('-s, --server <url>', 'Server URL (overrides ELIXCODE_SERVER)')
    .action(async (opts) => {
      const server = opts.server || client.server;
      console.log(pc.cyan(`Elixcode Chat — ${server}`));
      console.log(pc.gray('Type /help for commands, /exit to quit.\n'));

      const messages: { role: string; content: string }[] = [];
      const rl = readline.createInterface({
        input: process.stdin,
        output: process.stdout,
      });

      const ask = (prompt: string): Promise<string> => {
        return new Promise((resolve) => rl.question(prompt, resolve));
      };

      while (true) {
        const input = await ask(pc.green('> '));

        if (input === '/exit') break;
        if (input === '/help') {
          console.log('Commands: /exit, /help, /status, /reset');
          continue;
        }
        if (input === '/reset') {
          messages.length = 0;
          console.log(pc.gray('Conversation reset.'));
          continue;
        }
        if (input === '/status') {
          console.log(`Model: ${opts.model}`);
          continue;
        }

        messages.push({ role: 'user', content: input });
        const spinner = ora('Thinking...').start();
        try {
          const result = await client.chat(opts.model, messages, true);
          spinner.stop();
          const content = (result as { choices?: { delta?: { content?: string } }[] }).choices?.[0]?.delta?.content || '';
          if (content) {
            process.stdout.write(pc.blue(content));
            messages.push({ role: 'assistant', content });
          }
        } catch (err) {
          spinner.stop();
          if (err instanceof APIError) {
            console.error(pc.red(err.message));
          } else {
            console.error(pc.red('An error occurred'));
          }
        }
      }
      rl.close();
      console.log(pc.gray('\nGoodbye!'));
    });
}
