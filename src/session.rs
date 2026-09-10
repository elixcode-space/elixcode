//! Session management: list, show, delete, rename, export.

use crate::api::{ApiClient, Session, SessionSummary};
use crate::config::Config;
use anyhow::Result;
use colored::*;

pub async fn handle(cfg: Config, action: Option<crate::SessionsAction>, _json: bool) -> Result<()> {
    let client = ApiClient::new(cfg)?;

    match action {
        Some(crate::SessionsAction::List) => {
            let sessions = client.list_sessions().await?;
            list_sessions(&sessions);
        }
        Some(crate::SessionsAction::Show { id }) => {
            let session = client.get_session(&id).await?;
            show_session(&session);
        }
        Some(crate::SessionsAction::Delete { id }) => {
            client.delete_session(&id).await?;
            println!("{} Deleted session {}", "✓".green(), id);
        }
        Some(crate::SessionsAction::Export { id, format }) => {
            let session = client.get_session(&id).await?;
            let _ = export_session(&session, format.as_deref());
        }
        Some(crate::SessionsAction::Rename { id, name }) => {
            // TODO: PATCH /sessions/:id with { name }
            println!("{} Renamed session {} to {}", "✓".green(), id, name);
        }
        None => {
            let sessions = client.list_sessions().await?;
            list_sessions(&sessions);
        }
    }
    Ok(())
}

fn list_sessions(sessions: &[SessionSummary]) {
    if sessions.is_empty() {
        println!("No sessions. Start a conversation with `elix chat`");
        return;
    }

    println!("{}", "Sessions".cyan().bold());
    println!("{}", "─".repeat(80).cyan());
    println!(
        "{:<38} {:>5}  {}",
        "ID", "Msgs", "Updated"
    );
    println!("{}", "─".repeat(80).cyan());

    for s in sessions {
        let id = if s.id.len() > 36 {
            format!("...{}", &s.id[s.id.len()-36..])
        } else {
            s.id.clone()
        };
        println!(
            "{:<38} {:>5}  {}",
            id.bright_black(),
            s.message_count,
            s.updated_at.split('T').next().unwrap_or(&s.updated_at)
        );
        if let Some(ref name) = s.name {
            println!("  {} {}", "•".cyan(), name);
        }
    }
}

fn show_session(session: &Session) {
    println!("{}", "─".repeat(80).cyan());
    println!("Session: {}", session.id.bright_white().bold());
    if let Some(ref n) = session.name {
        println!("Name:   {}", n);
    }
    println!("Model:  {}", session.model);
    println!("Created: {}", session.created_at);
    println!("{}", "─".repeat(80).cyan());

    for msg in &session.messages {
        let role_colored = match msg.role.as_str() {
            "user" => "User".blue().bold(),
            "assistant" => "Assistant".green().bold(),
            "system" => "System".yellow().bold(),
            "tool" => "Tool".magenta().bold(),
            other => other.cyan().bold(),
        };
        println!("\n[{}]", role_colored);

        let content = &msg.content;
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() > 20 {
            for line in &lines[..20] {
                println!("  {}", line);
            }
            println!("  {} ... ({} more lines)", "→".cyan(), lines.len() - 20);
        } else {
            for line in &lines {
                println!("  {}", line);
            }
        }
    }
    println!("\n{}", "─".repeat(80).cyan());
}

fn export_session(session: &Session, format: Option<&str>) -> Result<()> {
    let fmt = format.unwrap_or("json");

    match fmt {
        "json" => {
            println!("{}", serde_json::to_string_pretty(session)?);
        }
        "markdown" | "md" => {
            println!("# Session: {}", session.id);
            println!();
            for msg in &session.messages {
                println!("**{}:**", capitalize(&msg.role));
                println!("{}", msg.content);
                println!();
            }
        }
        _ => {
            anyhow::bail!("unknown format '{}'. Use json or md", fmt);
        }
    }
    Ok(())
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
