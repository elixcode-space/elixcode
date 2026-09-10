//! Autonomous agent task runner (Claude Code / Cursor / kilo code style).
//!
//! The agent loop:
//!   1. Analyze the task
//!   2. Plan steps
//!   3. Execute with tools (read, edit, bash, grep)
//!   4. Review result
//!   5. Iterate until done or max_iter hit

use crate::api::{ApiClient, ChatMessage};
use crate::config::Config;
use anyhow::Result;
use colored::*;
use futures_util::StreamExt;
use std::io::Write;

pub async fn run(
    cfg: Config,
    task: Vec<String>,
    agent: String,
    max_iter: u32,
    _json: bool,
) -> Result<()> {
    let client = ApiClient::new(cfg)?;
    let task_text = task.join(" ");

    println!("{} Running agent '{}' on: {}", "→".cyan(), agent, task_text);
    println!("{}", "─".repeat(60).cyan());

    let system_prompt = match agent.as_str() {
        "coder" => "You are an expert autonomous coding agent. Given a task, break it down, execute file edits, run shell commands, and verify results. Be thorough but efficient.",
        "planner" => "You are a strategic planner. Analyze the task, create a detailed execution plan with numbered steps. Do not execute — just plan.",
        "reviewer" => "You are a code reviewer. Analyze the codebase, find bugs, security issues, and improvements. Be specific with file paths and line numbers.",
        "debugger" => "You are an expert debugger. Given an error, find the root cause and fix it. Use the minimum necessary changes.",
        _ => "You are an autonomous coding agent. Break down the task, execute it step by step, and verify the result.",
    };

    let mut messages = vec![
        ChatMessage::system(system_prompt),
        ChatMessage::user(&format!("Task: {}", task_text)),
    ];

    let model = &client.cfg.default_model;

    for iter in 1..=max_iter {
        println!("\n[Iteration {}/{}]", iter, max_iter);

        // Stream response
        let stream = client.chat_stream(model, &messages, Some(0.3), Some(2048)).await?;
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
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "✗".red(), e);
                    break;
                }
            }
        }
        println!();

        messages.push(ChatMessage::assistant(&response));

        // Check for completion signals
        let lower = response.to_lowercase();
        if lower.contains("task complete")
            || lower.contains("done")
            || lower.contains("finished")
            || lower.contains("all steps completed")
        {
            println!("\n{} Task complete", "✓".green().bold());
            break;
        }

        if iter == max_iter {
            println!("\n{} Max iterations ({}) reached", "⚠".yellow(), max_iter);
        }
    }

    Ok(())
}
