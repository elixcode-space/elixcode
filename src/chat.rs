//! Main chat loop: interactive REPL with streaming + tool use.

use crate::api::{ApiClient, ChatMessage};
use crate::config::Config;
use anyhow::Result;
use colored::*;
use futures_util::StreamExt;
use rustyline::Editor;
use std::io::Write;

const HISTORY_FILE: &str = ".elixcode/chat_history";

pub async fn run(
    cfg: Config,
    prompt: Vec<String>,
    session_id: Option<String>,
    agent: String,
    plan: bool,
    _json: bool,
) -> Result<()> {
    let mut client = ApiClient::new(cfg.clone())?;

    // Load or create session
    let session = if let Some(ref sid) = session_id {
        client.get_session(sid).await.ok()
    } else {
        None
    };

    let mut messages: Vec<ChatMessage> = if let Some(ref s) = session {
        s.messages.clone()
    } else {
        vec![]
    };

    // System prompt based on agent type
    let system_msg = match agent.as_str() {
        "planner" => "You are a strategic planner. Break down complex tasks into actionable steps. Output a numbered plan.",
        "reviewer" => "You are a code reviewer. Be thorough, constructive, and cite specific lines. Look for bugs, style issues, and improvements.",
        "ask" => "You are a helpful coding assistant. Be concise and accurate.",
        _ => "You are an expert coding agent (like Claude Code, Cursor, OpenCode). You have file editing, bash, grep, and web search tools. Think step by step.",
    };

    // Prepend system prompt if not present
    if messages.is_empty() || messages[0].role != "system" {
        messages.insert(0, ChatMessage::system(system_msg));
    }

    // Check server health
    if !client.health().await? {
        eprintln!("{} Server unreachable at {}", "✗".red(), client.cfg.server);
        eprintln!("  Start the server:  elix --gateway");
        std::process::exit(1);
    }

    // Build initial user prompt
    let user_input = if !prompt.is_empty() {
        prompt.join(" ")
    } else {
        // Interactive REPL
        run_repl(&mut messages, &mut client, &cfg, agent.as_str(), plan).await?;
        return Ok(());
    };

    messages.push(ChatMessage::user(&user_input));

    // Run agent loop
    if plan {
        run_agent(&mut messages, &mut client, plan).await?;
    } else {
        run_agent(&mut messages, &mut client, plan).await?;
    }

    Ok(())
}

pub async fn ask(
    cfg: Config,
    prompt: Vec<String>,
    model: Option<String>,
    session_id: Option<String>,
    _json: bool,
) -> Result<()> {
    let client = ApiClient::new(cfg)?;
    let model = model.unwrap_or_else(|| "anthropic/claude-3.5-sonnet".to_string());

    let mut messages = vec![];
    if let Some(sid) = session_id {
        if let Ok(s) = client.get_session(&sid).await {
            messages = s.messages;
        }
    }

    messages.push(ChatMessage::user(prompt.join(" ")));

    print!("{} ", " Assistant".cyan().bold());

    let stream = client.chat_stream(&model, &messages, None, None).await?;

    tokio::pin!(stream);
    let mut response = String::new();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(c) => {
                if !c.delta.is_empty() {
                    print!("{}", c.delta);
                    std::io::stdout().flush()?;
                    response.push_str(&c.delta);
                }
                if c.finish_reason.is_some() {
                    println!();
        break;
                }
            }
            Err(e) => {
                eprintln!("{} stream error: {}", "✗".red(), e);
                break;
            }
        }
    }

    Ok(())
}

pub async fn slash_command(
    cfg: Config,
    command: String,
    args: Vec<String>,
    _json: bool,
) -> Result<()> {
    let _client = ApiClient::new(cfg)?;

    match command.as_str() {
        "commit" => {
            let msg = args.join(" ");
            println!("{} Commit: {}", "→".cyan(), msg);
            // TODO: git add + commit
        }
        "test" => {
            println!("{} Running tests...", "→".cyan());
            // TODO: run test command
        }
        "review" => {
            println!("{} Code review mode...", "→".cyan());
        }
        "plan" => {
            println!("{} Planning...", "→".cyan());
        }
        "search" => {
            let query = args.join(" ");
            println!("{} Searching: {}", "→".cyan(), query);
            // TODO: search
        }
        "lsp" => {
            println!("{} LSP mode: {}", "→".cyan(), args.join(" "));
        }
        _ => {
            println!("{} Unknown command: /{}", "✗".red(), command);
        }
    }
    Ok(())
}

async fn run_repl(
    messages: &mut Vec<ChatMessage>,
    client: &mut ApiClient,
    cfg: &Config,
    agent: &str,
    _plan: bool,
) -> Result<()> {
    let mut rl: rustyline::DefaultEditor = Editor::new()?;
    let hist_path = dirs::home_dir()
        .map(|h| h.join(HISTORY_FILE))
        .unwrap_or_else(|| std::path::PathBuf::from(HISTORY_FILE));

    let _ = rl.load_history(&hist_path);

    println!("{} Elixcode — {} mode, {} model", "elix".green().bold(),
        agent, cfg.default_model);
    println!("{}", "─".repeat(60).cyan());
    println!("Type your message, or /help for commands. Ctrl+D to exit.\n");

    loop {
        let prompt = match rl.readline(&format!("{} ", "↪".cyan())) {
            Ok(line) => line,
            Err(rustyline::error::ReadlineError::Eof) => break,
            Err(rustyline::error::ReadlineError::Interrupted) => break,
            Err(e) => {
                eprintln!("readline error: {}", e);
                break;
            }
        };

        let trimmed = prompt.trim();
        if trimmed.is_empty() {
            continue;
        }

        let _ = rl.add_history_entry(trimmed);
        let _ = rl.save_history(&hist_path);

        if trimmed == "/exit" || trimmed == "/quit" {
            break;
        }

        if trimmed == "/help" {
            print_help();
            continue;
        }

        if trimmed.starts_with('/') {
            let parts: Vec<&str> = trimmed[1..].splitn(2, ' ').collect();
            let cmd = parts[0];
            let args = parts.get(1).map(|s| s.split(' ').map(String::from).collect::<Vec<_>>()).unwrap_or_default();
            slash_command(cfg.clone(), cmd.to_string(), args, false).await?;
            continue;
        }

        // Build user message
        let user_msg = ChatMessage::user(trimmed);
        messages.push(user_msg.clone());

        print!("{} ", " Assistant".cyan().bold());
        std::io::stdout().flush()?;

        // Stream response
        let stream = client.chat_stream(&cfg.default_model, messages, None, None).await?;

        tokio::pin!(stream);
        let mut assistant_content = String::new();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(c) => {
                    if !c.delta.is_empty() {
                        print!("{}", c.delta);
                        std::io::stdout().flush()?;
                        assistant_content.push_str(&c.delta);
                    }
                    if c.finish_reason.is_some() {
                        println!();
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("{} error: {}", "✗".red(), e);
                    break;
                }
            }
        }

        // Append assistant response
        messages.push(ChatMessage::assistant(&assistant_content));
    }

    Ok(())
}

async fn run_agent(
    messages: &mut Vec<ChatMessage>,
    client: &mut ApiClient,
    plan: bool,
) -> Result<()> {
    let model = &client.cfg.default_model;

    let stream = client.chat_stream(model, messages, None, None).await?;
    tokio::pin!(stream);

    let mut assistant_content = String::new();
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(c) => {
                if !c.delta.is_empty() {
                    if plan {
                        print!("{}", c.delta);
                    }
                    assistant_content.push_str(&c.delta);
                    std::io::stdout().flush()?;
                }
                if c.finish_reason.is_some() {
                    if plan {
                        println!();
                    }
                    break;
                }
            }
            Err(e) => {
                eprintln!("{} {}", "✗".red(), e);
                break;
            }
        }
    }

    messages.push(ChatMessage::assistant(&assistant_content));
    Ok(())
}

fn print_help() {
    println!(r#"
Slash commands:
  /commit <msg>   — git commit
  /test           — run tests
  /review         — code review mode
  /plan           — planning mode
  /search <q>     — search workspace
  /lsp <cmd>      — LSP commands
  /model <name>   — switch model
  /provider <p>   — switch provider
  /exit, /quit    — exit

Config:
  ELIXCODE_SERVER  — gateway URL
  ELIXCODE_API_KEY — your API key
  ELIXCODE_MODEL   — default model
"#);
}
