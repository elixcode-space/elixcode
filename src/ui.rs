//! TUI components for the interactive REPL — ratatui-based.

use crate::api::ChatMessage;
use colored::*;

pub fn render_message(msg: &ChatMessage, indent: usize) -> String {
    let prefix = " ".repeat(indent);
    let role_str = match msg.role.as_str() {
        "user" => "User".blue().bold().to_string(),
        "assistant" => "Assistant".green().bold().to_string(),
        "system" => "System".yellow().bold().to_string(),
        "tool" => "Tool".magenta().bold().to_string(),
        other => other.bright_white().bold().to_string(),
    };

    format!("{}{}\n{}{}", prefix, role_str, prefix, msg.content)
}

pub fn render_streaming_delta(delta: &str) -> String {
    // Colorize partial code blocks
    delta.to_string()
}

pub fn render_box(title: &str, content: &str) -> String {
    let width = content.lines().map(|l| l.len()).max().unwrap_or(0).max(title.len()) + 4;
    let bar = "─".repeat(width);

    format!(
        "┌─ {} {}\n{}\n└{}",
        title.cyan().bold(),
        "─".repeat(width - title.len() - 4).cyan(),
        content,
        bar
    )
}

pub fn render_diff(old: &str, new: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("{}{}\n", "─ Old:".red().bold(), "".red()));
    for line in old.lines() {
        out.push_str(&format!("  - {}\n", line.red()));
    }
    out.push_str(&format!("{}{}\n", "─ New:".green().bold(), "".green()));
    for line in new.lines() {
        out.push_str(&format!("  + {}\n", line.green()));
    }
    out
}
