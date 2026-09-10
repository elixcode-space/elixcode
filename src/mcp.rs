//! MCP (Model Context Protocol) server management.
//!
//! MCP servers extend the agent with tools via stdio or HTTP/SSE transport.

use crate::config::Config;
use anyhow::{Context, Result};
use colored::*;
use std::path::PathBuf;

pub async fn handle(cfg: Config, action: Option<crate::McpAction>, _json: bool) -> Result<()> {
    match action {
        Some(crate::McpAction::List) => {
            list_mcp_servers(&cfg);
        }
        Some(crate::McpAction::Add { name, transport, command, url }) => {
            add_mcp_server(&cfg, &name, &transport, command.as_deref(), url.as_deref())?;
        }
        Some(crate::McpAction::Remove { name }) => {
            remove_mcp_server(&cfg, &name)?;
        }
        Some(crate::McpAction::Tools { name }) => {
            list_mcp_tools(&cfg, &name)?;
        }
        None => {
            list_mcp_servers(&cfg);
        }
    }
    Ok(())
}

fn mcp_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("no home dir")?;
    Ok(home.join(".elixcode").join("mcp.toml"))
}

fn list_mcp_servers(cfg: &Config) {
    if cfg.mcp_servers.is_empty() {
        println!("No MCP servers configured.");
        println!("Add one: elix mcp add <name> --transport stdio --command <cmd>");
        return;
    }

    println!("{}", "MCP Servers".cyan().bold());
    println!("{}", "─".repeat(60).cyan());
    for (name, server) in &cfg.mcp_servers {
        println!(
            "  {:<20} {:<10} {}",
            name.green(),
            server.transport.yellow(),
            if server.enabled { "✓".green() } else { "✗".red() }
        );
        if let Some(ref cmd) = server.command {
            println!("    cmd: {}", cmd);
        }
        if let Some(ref url) = server.url {
            println!("    url: {}", url);
        }
    }
}

fn add_mcp_server(
    cfg: &Config,
    name: &str,
    transport: &str,
    command: Option<&str>,
    url: Option<&str>,
) -> Result<()> {
    let mut new_cfg = cfg.clone();
    new_cfg.mcp_servers.insert(name.to_string(), crate::config::McpConfig {
        name: name.to_string(),
        transport: transport.to_string(),
        command: command.map(String::from),
        args: None,
        url: url.map(String::from),
        env: Default::default(),
        enabled: true,
    });
    new_cfg.save()?;
    println!("{} MCP server '{}' added ({})", "✓".green(), name, transport);
    Ok(())
}

fn remove_mcp_server(cfg: &Config, name: &str) -> Result<()> {
    let mut new_cfg = cfg.clone();
    if new_cfg.mcp_servers.remove(name).is_none() {
        anyhow::bail!("MCP server '{}' not found", name);
    }
    new_cfg.save()?;
    println!("{} MCP server '{}' removed", "✓".green(), name);
    Ok(())
}

fn list_mcp_tools(cfg: &Config, name: &str) -> Result<()> {
    let server = cfg.mcp_servers.get(name)
        .with_context(|| format!("MCP server '{}' not found", name))?;

    println!("MCP server: {}", name);
    println!("Transport: {}", server.transport);
    println!();
    println!("Tools (connect to server to discover):");
    println!("  Connect to {} at {} to enumerate tools.", name, server.transport);

    Ok(())
}
