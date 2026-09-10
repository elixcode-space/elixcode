//! Configuration management for.elixcode (Rust user CLI).
//!
//! Config is loaded from (in priority order):
//!   1. Environment variables (ELIXCODE_*, ELIXCODE_*)
//!   2. CLI flags
//!   3. ~/.elixcodecode/config.toml

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: String,
    pub api_key: Option<String>,
    pub default_model: String,
    pub workspace: PathBuf,
    pub providers: HashMap<String, ProviderConfig>,
    pub mcp_servers: HashMap<String, McpConfig>,
    pub auth: AuthConfig,
    pub ui: UiConfig,
    pub streaming: StreamingConfig,
    pub sandbox: SandboxConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub default_model: Option<String>,
    pub enabled: bool,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub name: String,
    pub transport: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
    pub env: HashMap<String, String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub vim_mode: bool,
    pub syntax_theme: String,
    pub font: Option<String>,
    pub font_size: u32,
    pub stream_responses: bool,
    pub show_timestamps: bool,
    pub show_token_count: bool,
    pub show_model: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    pub enabled: bool,
    pub buffer_size: usize,
    pub chunk_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub allowed_paths: Vec<String>,
    pub blocked_paths: Vec<String>,
    pub max_file_size_mb: u32,
    pub exec_timeout_secs: u32,
    pub shell_enabled: bool,
    pub network_allowed: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: default_server(),
            api_key: None,
            default_model: "anthropic/claude-3.5-sonnet".to_string(),
            workspace: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            providers: default_providers(),
            mcp_servers: HashMap::new(),
            auth: AuthConfig {
                jwt: None,
                refresh_token: None,
                expires_at: None,
                user_id: None,
            },
            ui: UiConfig {
                theme: "dark".to_string(),
                vim_mode: true,
                syntax_theme: "dracula".to_string(),
                font: None,
                font_size: 14,
                stream_responses: true,
                show_timestamps: true,
                show_token_count: true,
                show_model: true,
            },
            streaming: StreamingConfig {
                enabled: true,
                buffer_size: 64,
                chunk_delay_ms: 0,
            },
            sandbox: SandboxConfig {
                allowed_paths: vec![".".to_string()],
                blocked_paths: vec![
                    "/etc".to_string(),
                    "/root".to_string(),
                    "/.ssh".to_string(),
                    "/var/log".to_string(),
                ],
                max_file_size_mb: 100,
                exec_timeout_secs: 300,
                shell_enabled: true,
                network_allowed: true,
            },
        }
    }
}

fn default_server() -> String {
    std::env::var("ELIXCODE_SERVER")
        .or_else(|_| std::env::var("ELIXCODE_SERVER"))
        .unwrap_or_else(|_| "https://api.elixcodecode.space".to_string())
}

fn default_providers() -> HashMap<String, ProviderConfig> {
    let mut m = HashMap::new();
    m.insert("openrouter".to_string(), ProviderConfig {
        name: "OpenRouter".to_string(),
        api_key: std::env::var("OPENROUTER_API_KEY").ok(),
        base_url: None,
        default_model: Some("anthropic/claude-3.5-sonnet".to_string()),
        enabled: true,
        priority: 10,
    });
    m.insert("anthropic".to_string(), ProviderConfig {
        name: "Anthropic".to_string(),
        api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
        base_url: Some("https://api.anthropic.com".to_string()),
        default_model: Some("claude-3.5-sonnet-20241022".to_string()),
        enabled: true,
        priority: 20,
    });
    m.insert("openai".to_string(), ProviderConfig {
        name: "OpenAI".to_string(),
        api_key: std::env::var("OPENAI_API_KEY").ok(),
        base_url: None,
        default_model: Some("gpt-4o".to_string()),
        enabled: true,
        priority: 30,
    });
    m.insert("self_hosted".to_string(), ProviderConfig {
        name: "Self-hosted".to_string(),
        api_key: None,
        base_url: Some("http://localhost:11434".to_string()),
        default_model: Some("llama3.1:8b".to_string()),
        enabled: true,
        priority: 5,
    });
    m
}

impl Config {
    /// Load config: CLI flags override config file, env vars override both
    pub fn load(
        server_flag: Option<String>,
        api_key_flag: Option<String>,
        model_flag: Option<String>,
        workspace_flag: Option<PathBuf>,
    ) -> Result<Self> {
        let config_path = config_file_path()?;

        // Start from file or defaults
        let mut cfg = if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("reading config {}", config_path.display()))?;
            toml::from_str(&content)
                .with_context(|| format!("parsing config {}", config_path.display()))?
        } else {
            Config::default()
        };

        // Override with env vars
        if let Ok(val) = std::env::var("ELIXCODE_SERVER") {
            cfg.server = val;
        }
        if let Ok(val) = std::env::var("ELIXCODE_API_KEY") {
            cfg.api_key = Some(val);
        }
        if let Ok(val) = std::env::var("ELIXCODE_MODEL") {
            cfg.default_model = val;
        }
        if let Ok(val) = std::env::var("ELIXCODE_WORKSPACE") {
            cfg.workspace = PathBuf::from(val);
        }

        // Override with CLI flags (highest priority)
        if let Some(s) = server_flag {
            cfg.server = s;
        }
        if let Some(k) = api_key_flag {
            cfg.api_key = Some(k);
        }
        if let Some(m) = model_flag {
            cfg.default_model = m;
        }
        if let Some(w) = workspace_flag {
            cfg.workspace = w;
        }

        Ok(cfg)
    }

    /// Save config to ~/.elixcodecode/config.toml
    pub fn save(&self) -> Result<()> {
        let path = config_file_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating config dir {}", parent.display()))?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Effective API key (from config, providers, or env)
    pub fn effective_api_key(&self) -> Option<&str> {
        // First check the top-level key
        if let Some(ref k) = self.api_key {
            return Some(k);
        }
        // Then check providers
        for prov in self.providers.values() {
            if prov.enabled {
                if let Some(ref k) = prov.api_key {
                    return Some(k);
                }
            }
        }
        None
    }

    /// Build base URL for a request
    pub fn api_base(&self) -> String {
        self.server.trim_end_matches('/').to_string()
    }
}

pub fn config_file_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .context("no home directory found")?;
    Ok(home.join(".elixcodecode").join("config.toml"))
}

/// Handle config subcommands
pub async fn handle(cfg: Config, action: Option<crate::ConfigAction>, _json: bool) -> anyhow::Result<()> {
    use crate::ConfigAction;
    match action {
        Some(ConfigAction::Show) => {
            println!("{}", toml::to_string_pretty(&cfg)?);
        }
        Some(ConfigAction::Set { key, value }) => {
            let mut new_cfg = cfg.clone();
            match key.as_str() {
                "server" => new_cfg.server = value,
                "model" => new_cfg.default_model = value,
                "default_model" => new_cfg.default_model = value,
                "workspace" => new_cfg.workspace = PathBuf::from(value),
                _ => {
                    println!("Setting custom key '{}' = '{}'", key, value);
                }
            }
            new_cfg.save()?;
            println!("Config updated");
        }
        Some(ConfigAction::Unset { key }) => {
            println!("Unset '{}' (not persisted in this version)", key);
        }
        Some(ConfigAction::Edit) => {
            let path = config_file_path()?;
            if let Some(editor) = std::env::var_os("EDITOR") {
                std::process::Command::new(editor)
                    .arg(&path)
                    .status()?;
            } else {
                println!("No $EDITOR set. Config at: {}", path.display());
            }
        }
        Some(ConfigAction::Reset) => {
            let default = Config::default();
            default.save()?;
            println!("Config reset to defaults");
        }
        None => {
            println!("Config at {}:", config_file_path()?.display());
            println!("{}", toml::to_string_pretty(&cfg)?);
        }
    }
    Ok(())
}
