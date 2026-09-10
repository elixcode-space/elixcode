//! Agent monitoring commands (`elix monitor`).
//!
//! Wraps the `agent-top` terminal dashboard for local coding agents
//! (Claude Code, Codex, Gemini CLI, OpenCode): live dashboard, cost and
//! token reports, trace export (Perfetto/Jaeger), JSON snapshots, price
//! table and installation help.
//!
//! See: https://github.com/kannandreams/agent-top

use crate::config::Config;
use anyhow::{bail, Context, Result};
use colored::*;
use serde_json::Value;
use tokio::process::Command as TokioCmd;

const BINARY: &str = "agent-top";

#[derive(Debug, Clone, clap::Subcommand)]
pub enum MonitorAction {
    /// Launch the interactive agent-top dashboard
    Dashboard {
        /// Refresh interval in ms
        #[arg(long, default_value = "1000")]
        interval_ms: u64,
        /// Keep stopped sessions visible for N minutes
        #[arg(long, default_value = "30")]
        stopped_window_min: u64,
    },
    /// Print one snapshot and exit
    Snapshot {
        /// Output the snapshot as JSON
        #[arg(long)]
        json: bool,
        /// Refresh interval in ms
        #[arg(long, default_value = "1000")]
        interval_ms: u64,
    },
    /// Summarize all local coding agents from a JSON snapshot
    Agents,
    /// Totals from an agent-top JSON snapshot (sessions/tokens/cost/MCP)
    Summary,
    /// Cost and token report grouped by harness/model/project/day
    Report {
        /// Time range: 7d, 30d, all, or YYYY-MM-DD
        #[arg(long, default_value = "7d")]
        since: String,
        /// Group by: harness, model, project, or day
        #[arg(long, default_value = "harness")]
        by: String,
        /// Output the report as JSON
        #[arg(long)]
        json: bool,
    },
    /// Export a session trace for Perfetto (chrome) or Jaeger (otlp)
    Trace {
        /// Session id, unique prefix, or transcript path
        session: String,
        /// Export format: chrome or otlp
        #[arg(long, default_value = "chrome")]
        format: String,
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
        /// OTLP endpoint URL (posts the trace, e.g. to Jaeger)
        #[arg(long)]
        endpoint: Option<String>,
    },
    /// Render a saved --json snapshot (bug reports)
    Replay {
        /// Path to a saved snapshot JSON
        file: String,
    },
    /// Show the pricing table in use
    Prices,
    /// Show what's new in this build
    WhatsNew,
    /// Print shell completions for agent-top
    Completions {
        /// Shell: bash, zsh, fish
        shell: String,
    },
    /// Check whether agent-top is installed
    Available,
    /// Show installation instructions
    Install,
}

pub async fn dispatch(_cfg: Config, action: MonitorAction) -> Result<()> {
    match action {
        MonitorAction::Dashboard { interval_ms, stopped_window_min } => dashboard(interval_ms, stopped_window_min).await,
        MonitorAction::Snapshot { json, interval_ms } => snapshot(json, interval_ms).await,
        MonitorAction::Agents => agents().await,
        MonitorAction::Summary => summary().await,
        MonitorAction::Report { since, by, json } => report(&since, &by, json).await,
        MonitorAction::Trace { session, format, output, endpoint } => {
            trace(&session, &format, output.as_deref(), endpoint.as_deref()).await
        }
        MonitorAction::Replay { file } => replay(&file).await,
        MonitorAction::Prices => prices().await,
        MonitorAction::WhatsNew => whats_new().await,
        MonitorAction::Completions { shell } => completions(&shell).await,
        MonitorAction::Available => available().await,
        MonitorAction::Install => {
            install_instructions();
            Ok(())
        }
    }
}

async fn dashboard(interval_ms: u64, stopped_window_min: u64) -> Result<()> {
    let bin = find_binary().context(not_installed_help())?;
    println!("{} Starting agent-top dashboard", "→".cyan());
    println!("{} Press 'q' to quit, '?' for help", "".bold());
    let status = TokioCmd::new(bin)
        .arg("--interval-ms")
        .arg(interval_ms.to_string())
        .arg("--stopped-window-min")
        .arg(stopped_window_min.to_string())
        .status()
        .await?;
    ensure_success(status)
}

async fn snapshot(json: bool, interval_ms: u64) -> Result<()> {
    let bin = find_binary().context(not_installed_help())?;
    let mut cmd = TokioCmd::new(bin);
    cmd.arg("--once");
    if json {
        cmd.arg("--json");
    }
    cmd.arg("--interval-ms").arg(interval_ms.to_string());
    let status = cmd.status().await?;
    ensure_success(status)
}

async fn agents() -> Result<()> {
    let bin = find_binary().context(not_installed_help())?;
    let output = TokioCmd::new(bin)
        .arg("--json")
        .output()
        .await
        .context("failed to run agent-top --json")?;
    if !output.status.success() {
        bail!("agent-top exited with {}", output.status);
    }
    let parsed: Value = serde_json::from_slice(&output.stdout).context("failed to parse agent-top --json output")?;

    let sessions = num(&parsed, &["sessions", "total"]);
    let tokens = string_or(&parsed, &["tokens", "total"], "0");
    let cost = string_or(&parsed, &["cost", "total"], "0.00");
    let orphaned = num(&parsed, &["mcp", "orphaned"]);

    println!("{}", "Local coding agents:".bold().cyan());
    println!("  sessions:      {}", sessions.to_string().yellow());
    println!("  total tokens:  {}", tokens.yellow());
    println!("  total cost:    ${}", cost.green());
    println!("  orphaned MCP:  {}", orphaned.to_string().red());

    let rows = parsed
        .get("sessions_list")
        .or_else(|| parsed.get("sessions"))
        .and_then(Value::as_array);

    if let Some(rows) = rows {
        if !rows.is_empty() {
            println!();
            println!(
                "{} {} {} {}",
                pad_to("SESSION", 16).dimmed(),
                pad_to("STATE", 10).dimmed(),
                pad_to("TOKENS", 14).dimmed(),
                "COST".dimmed()
            );
            println!("{}", "─".repeat(52).dimmed());
            for row in rows {
                let id = row.get("id").and_then(Value::as_str).unwrap_or("?").chars().take(12).collect::<String>();
                let state = row.get("state").and_then(Value::as_str).unwrap_or("?");
                let tokens = row.get("tokens").map(display).unwrap_or_else(|| "?".to_string());
                let cost = row.get("cost").map(display).unwrap_or_else(|| "?".to_string());
                println!(
                    "{} {} {} {}",
                    pad_to(&id, 16),
                    pad_to(state, 10),
                    pad_to(&tokens, 14),
                    cost
                );
            }
        }
    }
    Ok(())
}

async fn summary() -> Result<()> {
    let bin = find_binary().context(not_installed_help())?;
    let output = TokioCmd::new(bin)
        .arg("--json")
        .output()
        .await
        .context("failed to run agent-top --json")?;
    if !output.status.success() {
        bail!("agent-top exited with {}", output.status);
    }
    let parsed: Value = serde_json::from_slice(&output.stdout).context("failed to parse agent-top --json output")?;

    println!("{}", "Agent monitor summary:".bold().cyan());
    println!("  sessions:      {}", num(&parsed, &["sessions", "total"]).to_string().yellow());
    println!("  total tokens:  {}", string_or(&parsed, &["tokens", "total"], "0").yellow());
    println!("  total cost:    ${}", string_or(&parsed, &["cost", "total"], "0.00").green());
    println!("  orphaned MCP:  {}", num(&parsed, &["mcp", "orphaned"]).to_string().red());
    Ok(())
}

async fn report(since: &str, by: &str, json: bool) -> Result<()> {
    let bin = find_binary().context(not_installed_help())?;
    let mut cmd = TokioCmd::new(bin);
    cmd.arg("report").arg("--since").arg(since).arg("--by").arg(by);
    if json {
        cmd.arg("--json");
    }
    let output = cmd.output().await.context("failed to run agent-top report")?;
    if !output.status.success() {
        bail!("agent-top report exited with {}", output.status);
    }
    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

async fn trace(session: &str, format: &str, output: Option<&str>, endpoint: Option<&str>) -> Result<()> {
    let bin = find_binary().context(not_installed_help())?;
    let mut cmd = TokioCmd::new(bin);
    cmd.arg("trace").arg("--session").arg(session).arg("--format").arg(format);
    if let Some(o) = output {
        cmd.arg("-o").arg(o);
    }
    if let Some(e) = endpoint {
        cmd.arg("--endpoint").arg(e);
    }
    let status = cmd.status().await?;
    ensure_success(status)
}

async fn replay(file: &str) -> Result<()> {
    let bin = find_binary().context(not_installed_help())?;
    let status = TokioCmd::new(bin).arg("--replay").arg(file).status().await?;
    ensure_success(status)
}

async fn prices() -> Result<()> {
    let bin = find_binary().context(not_installed_help())?;
    let status = TokioCmd::new(bin).arg("--prices").status().await?;
    ensure_success(status)
}

async fn whats_new() -> Result<()> {
    let bin = find_binary().context(not_installed_help())?;
    let status = TokioCmd::new(bin).arg("--whats-new").status().await?;
    ensure_success(status)
}

async fn completions(shell: &str) -> Result<()> {
    let bin = find_binary().context(not_installed_help())?;
    let status = TokioCmd::new(bin).arg("--completions").arg(shell).status().await?;
    ensure_success(status)
}

async fn available() -> Result<()> {
    match find_binary() {
        Some(bin) => {
            println!("{} agent-top is available at: {}", "✓".green().bold(), bin);
            Ok(())
        }
        None => {
            println!("{} agent-top not found in PATH", "✗".red());
            install_instructions();
            Ok(())
        }
    }
}

fn install_instructions() {
    println!(
        "\n{}\n\n  brew install kannandreams/tap/agent-top\n  cargo binstall agent-top\n  cargo install --locked agent-top\n\n  See: https://github.com/kannandreams/agent-top/releases\n",
        format!(
            "⚠  agent-top is not installed. Install it with:"
        ).yellow().bold()
    );
    println!("  After installation, verify with: elix monitor available");
}

fn not_installed_help() -> String {
    "agent-top is not installed — run `elix monitor install` for setup instructions".to_string()
}

/// Find `agent-top` on PATH without spawning (checked before running it).
fn find_binary() -> Option<String> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(BINARY);
        if candidate.is_file() {
            // On Unix, also require the executable bit.
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = std::fs::metadata(&candidate) {
                    if meta.permissions().mode() & 0o111 != 0 {
                        return Some(candidate.display().to_string());
                    }
                }
            }
            #[cfg(not(unix))]
            {
                return Some(candidate.display().to_string());
            }
        }
    }
    None
}

fn ensure_success(status: std::process::ExitStatus) -> Result<()> {
    if status.success() {
        Ok(())
    } else {
        bail!("agent-top exited with {}", status)
    }
}

fn num(v: &Value, keys: &[&str]) -> i64 {
    keys.iter()
        .fold(v.clone(), |acc, k| acc.get(k).cloned().unwrap_or(Value::Null))
        .as_i64()
        .unwrap_or(0)
}

fn string_or(v: &Value, keys: &[&str], default: &str) -> String {
    keys.iter()
        .fold(v.clone(), |acc, k| acc.get(k).cloned().unwrap_or(Value::Null))
        .as_str()
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| default.to_string())
}

fn display(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Null => "?".to_string(),
        other => other.to_string(),
    }
}

fn pad_to(s: &str, width: usize) -> String {
    let mut out = String::from(s);
    if out.len() < width {
        out.push_str(&" ".repeat(width - out.len()));
    }
    out
}