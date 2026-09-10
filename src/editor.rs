//! File editor (Claude Code / Cursor style).
//!
//! Supports:
//!   - Read file and submit for editing
//!   - Apply edits via LLM suggestions
//!   - Show diff before applying

use crate::api::{ApiClient, ChatMessage};
use crate::config::Config;
use anyhow::{Context, Result};
use colored::*;
use std::fs;

pub async fn run(
    cfg: Config,
    file: std::path::PathBuf,
    instruction: Option<String>,
    _json: bool,
) -> Result<()> {
    let client = ApiClient::new(cfg)?;

    if !file.exists() {
        anyhow::bail!("file not found: {}", file.display());
    }

    let content = fs::read_to_string(&file)
        .with_context(|| format!("reading {}", file.display()))?;

    let instruction_text = instruction.unwrap_or_else(|| {
        String::from("Review and improve this file. Suggest edits.")
    });

    let messages = vec![
        ChatMessage::system("You are a code editing agent. Read the file content, then suggest edits as a unified patch or by describing changes."),
        ChatMessage::user(&format!(
            "File: {}\n\n```\n{}\n```\n\nTask: {}",
            file.display(),
            content,
            instruction_text
        )),
    ];

    print!("Reviewing {}... ", file.display().to_string().green());

    let resp: crate::api::ChatCompletionsResponse = client
        .chat_completions(&client.cfg.default_model, &messages, false, Some(0.7), Some(2048))
        .await?;

    let response = resp.choices.first()
        .and_then(|c| c.message.as_ref())
        .map(|m| m.content.clone())
        .unwrap_or_default();

    println!("done");

    println!("\n{}", "─".repeat(60).cyan());
    println!("{}", response);
    println!("{}", "─".repeat(60).cyan());
    println!("\nEdit session complete. Use `elixcode edit {} -- \"instruction\"` to make specific changes.", file.display());

    Ok(())
}
