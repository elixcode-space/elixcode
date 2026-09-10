use clap::{Parser, Subcommand, ValueHint};
use std::path::PathBuf;

mod api;
mod auth;
mod config;
mod session;
mod chat;
mod tools;
mod skills;
mod agents;
mod share;
mod ui;
mod streaming;
mod model;
mod search;
mod providers;
mod mcp;
mod editor;
mod plan;
mod tokens;
mod fleet;
mod monitor;

use anyhow::Result;

#[derive(Parser, Debug)]
#[command(
    name = "elixcode",
    version,
    about = "Elixcode user CLI — OpenCode/Cursor/Claude Code-style AI coding agent",
    long_about = "elixcode is the user-facing CLI for the Elixcode platform. \
                  Talk to LLMs, edit code, run agents, share sessions, and \
                  work across providers (OpenRouter, Anthropic, OpenAI, self-hosted)."
)]
pub struct Cli {
    /// Elixcode server URL (gateway). Defaults to ~/.elixcode/config.toml
    #[arg(long, env = "ELIXCODE_SERVER", global = true)]
    pub server: Option<String>,

    /// API key (overrides config)
    #[arg(long, env = "ELIXCODE_API_KEY", global = true)]
    pub api_key: Option<String>,

    /// Default model to use
    #[arg(long, env = "ELIXCODE_MODEL", global = true)]
    pub model: Option<String>,

    /// Working directory
    #[arg(long, env = "ELIXCODE_WORKSPACE", global = true, value_hint = ValueHint::DirPath)]
    pub workspace: Option<PathBuf>,

    /// Output as JSON
    #[arg(long, global = true)]
    pub json: bool,

    /// Verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Interactive chat session (default if no command)
    Chat {
        /// Starting prompt
        prompt: Vec<String>,
        /// Resume a saved session
        #[arg(long)]
        session: Option<String>,
        /// Use a specific agent (coder, planner, reviewer, ask)
        #[arg(long, default_value = "coder")]
        agent: String,
        /// Plan mode — only plan, don't edit files
        #[arg(long)]
        plan: bool,
    },

    /// Send a single non-interactive prompt
    Ask {
        /// The prompt text
        prompt: Vec<String>,
        /// Model to use
        #[arg(long)]
        model: Option<String>,
        /// Session ID for context
        #[arg(long)]
        session: Option<String>,
    },

    /// Edit a file with the agent (Claude Code-style)
    Edit {
        /// File to edit
        #[arg(value_hint = ValueHint::FilePath)]
        file: PathBuf,
        /// Optional instruction
        instruction: Option<String>,
    },

    /// Run an autonomous agent task
    Run {
        /// The task description
        task: Vec<String>,
        /// Agent profile
        #[arg(long, default_value = "coder")]
        agent: String,
        /// Max iterations
        #[arg(long, default_value = "50")]
        max_iter: u32,
    },

    /// List sessions
    Sessions {
        #[command(subcommand)]
        action: Option<SessionsAction>,
    },

    /// Share a session (returns a URL)
    Share {
        /// Session ID
        session: String,
    },

    /// Search across the workspace (semantic + regex)
    Search {
        /// Search query
        query: Vec<String>,
        /// Only file paths (regex mode)
        #[arg(long)]
        path: bool,
    },

    /// Manage skills (extensibility)
    Skills {
        #[command(subcommand)]
        action: Option<SkillsAction>,
    },

    /// List available models
    Models {
        /// Filter by provider
        #[arg(long)]
        provider: Option<String>,
    },

    /// Manage providers (OpenRouter, Anthropic, etc.)
    Providers {
        #[command(subcommand)]
        action: Option<ProvidersAction>,
    },

    /// Manage MCP (Model Context Protocol) servers
    Mcp {
        #[command(subcommand)]
        action: Option<McpAction>,
    },

    /// Auth: login, logout, whoami
    Auth {
        #[command(subcommand)]
        action: Option<AuthAction>,
    },

    /// Configuration
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },

    /// Fleet management (workers, hardware, warmup, deploy)
    Fleet {
        #[command(subcommand)]
        action: fleet::FleetAction,
    },

    /// Monitor local coding agents (agent-top): dashboard, cost, traces
    Monitor {
        #[command(subcommand)]
        action: monitor::MonitorAction,
    },

    /// Alias for `monitor`
    Top {
        #[command(subcommand)]
        action: monitor::MonitorAction,
    },

    /// Show version + system info
    Version,

    /// OpenCode-style: run a slash command (e.g. /commit, /test, /review)
    Cmd {
        /// Slash command (without leading /)
        command: String,
        /// Args
        args: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum SessionsAction {
    List,
    Show { id: String },
    Delete { id: String },
    Export { id: String, format: Option<String> },
    Rename { id: String, name: String },
}

#[derive(Subcommand, Debug)]
pub enum SkillsAction {
    List,
    Load { name: String },
    Search { query: Vec<String> },
    Create { name: String },
}

#[derive(Subcommand, Debug)]
pub enum ProvidersAction {
    List,
    Add {
        /// Provider name
        name: String,
        /// API key
        #[arg(long)]
        key: String,
        /// API base URL
        #[arg(long)]
        base: Option<String>,
    },
    Remove { name: String },
    Test { name: String },
}

#[derive(Subcommand, Debug)]
pub enum McpAction {
    List,
    Add {
        name: String,
        /// Transport type
        #[arg(long, default_value = "stdio")]
        transport: String,
        /// Command (for stdio)
        #[arg(long)]
        command: Option<String>,
        /// URL (for http/sse)
        #[arg(long)]
        url: Option<String>,
    },
    Remove { name: String },
    Tools { name: String },
}

#[derive(Subcommand, Debug)]
pub enum AuthAction {
    /// Login with API key or OAuth
    Login,
    /// Logout and clear credentials
    Logout,
    /// Show current user
    Whoami,
    /// Refresh token
    Refresh,
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    Show,
    Set { key: String, value: String },
    Unset { key: String },
    /// Open config file in editor
    Edit,
    /// Reset to defaults
    Reset,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Warn)
            .init();
    }

    // Load config
    let cfg = config::Config::load(cli.server, cli.api_key, cli.model, cli.workspace)?;

    // Dispatch
    match cli.command {
        Commands::Chat { prompt, session, agent, plan } => {
            chat::run(cfg, prompt, session, agent, plan, cli.json).await
        }
        Commands::Ask { prompt, model, session } => {
            chat::ask(cfg, prompt, model, session, cli.json).await
        }
        Commands::Edit { file, instruction } => {
            editor::run(cfg, file, instruction, cli.json).await
        }
        Commands::Run { task, agent, max_iter } => {
            agents::run(cfg, task, agent, max_iter, cli.json).await
        }
        Commands::Sessions { action } => {
            session::handle(cfg, action, cli.json).await
        }
        Commands::Share { session } => {
            share::create(cfg, session, cli.json).await
        }
        Commands::Search { query, path } => {
            search::run(cfg, query, path, cli.json).await
        }
        Commands::Skills { action } => {
            skills::handle(cfg, action, cli.json).await
        }
        Commands::Models { provider } => {
            model::list(cfg, provider, cli.json).await
        }
        Commands::Providers { action } => {
            providers::handle(cfg, action, cli.json).await
        }
        Commands::Mcp { action } => {
            mcp::handle(cfg, action, cli.json).await
        }
        Commands::Auth { action } => {
            auth::handle(cfg, action, cli.json).await
        }
        Commands::Config { action } => {
            config::handle(cfg, action, cli.json).await
        }
        Commands::Fleet { action } => {
            fleet::dispatch(cfg, action, cli.json).await
        }
        Commands::Monitor { action } => {
            monitor::dispatch(cfg, action).await
        }
        Commands::Top { action } => {
            monitor::dispatch(cfg, action).await
        }
        Commands::Version => {
            version_info();
            Ok(())
        }
        Commands::Cmd { command, args } => {
            chat::slash_command(cfg, command, args, cli.json).await
        }
    }
}

fn version_info() {
    println!("elixcode {}", env!("CARGO_PKG_VERSION"));
    println!("rustc {}", rustc_version_runtime());
    println!("target {}", std::env::consts::ARCH);
    println!();
    println!("Elixcode platform — full production system");
    println!("User CLI in Rust, platform CLI in Zig, Phoenix backend in Elixir");
}

fn rustc_version_runtime() -> String {
    let version = std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string());
    version
}
