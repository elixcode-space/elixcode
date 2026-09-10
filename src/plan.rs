//! Plan mode: structured task breakdown without file changes.
//!
//! The planner agent outputs a numbered step-by-step plan, then waits
//! for user confirmation before executing each step.

use crate::api::{ApiClient, ChatMessage};
use crate::config::Config;
use anyhow::Result;
use colored::*;
use futures_util::StreamExt;
use std::io::Write;

pub async fn plan(cfg: Config, task: Vec<String>, _json: bool) -> Result<()> {
    let client = ApiClient::new(cfg.clone())?;
    let task_text = task.join(" ");

    println!("{}", "Planning mode".cyan().bold());
    println!("Task: {}\n", task_text);
    println!("{}", "─".repeat(60).cyan());

    let messages = vec![
        ChatMessage::system(
            "You are a strategic planner. Break down the user's task into clear, numbered steps. \
            For each step, describe what needs to be done and what file/command is involved. \
            Do NOT make any changes — just plan."
        ),
        ChatMessage::user(&task_text),
    ];

    let stream = client.chat_stream(&client.cfg.default_model, &messages, Some(0.3), Some(1024)).await?;
    tokio::pin!(stream);

    let mut plan_text = String::new();
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(c) => {
                print!("{}", c.delta);
                std::io::stdout().flush()?;
                plan_text.push_str(&c.delta);
                if c.finish_reason.is_some() {
                    break;
                }
            }
            Err(e) => {
                eprintln!("{} {}", "✗".red(), e);
                break;
            }
        }
    }

    println!("\n\n{}", "─".repeat(60).cyan());
    println!("{} Review the plan above. To execute step N, run:", "→".cyan());
    println!("  elixcode run --step N");
    println!("{} To execute all steps:", "→".cyan());
    println!("  elixcode run --confirm");

    Ok(())
}
