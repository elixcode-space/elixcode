//! Tool registry mirroring the Elixir ToolRegistry.
//! Tools are dispatched by the Elixcode backend; this is the client-side
//! model that flows over the wire.

use crate::api::ToolCall;
use anyhow::Result;
use colored::*;
use std::process::Command;

/// A tool definition sent to the LLM so it can decide to call it
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub requires_confirmation: bool,
}

/// Standard tool catalog (mirrors Elixir ToolRegistry)
pub fn standard_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "bash".to_string(),
            description: "Run a shell command. Returns stdout, stderr, exit code.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string", "description": "The command to run" },
                    "timeout_secs": { "type": "integer", "default": 30 }
                },
                "required": ["command"]
            }),
            requires_confirmation: true,
        },
        Tool {
            name: "read".to_string(),
            description: "Read a file from the workspace.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "start_line": { "type": "integer", "default": 0 },
                    "end_line": { "type": "integer" }
                },
                "required": ["path"]
            }),
            requires_confirmation: false,
        },
        Tool {
            name: "write".to_string(),
            description: "Create or overwrite a file.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "content": { "type": "string" }
                },
                "required": ["path", "content"]
            }),
            requires_confirmation: true,
        },
        Tool {
            name: "edit".to_string(),
            description: "Edit a file by replacing a specific string.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "old_text": { "type": "string" },
                    "new_text": { "type": "string" }
                },
                "required": ["path", "old_text", "new_text"]
            }),
            requires_confirmation: true,
        },
        Tool {
            name: "grep".to_string(),
            description: "Search for a regex pattern in files.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string" },
                    "path": { "type": "string" },
                    "glob": { "type": "string" }
                },
                "required": ["pattern"]
            }),
            requires_confirmation: false,
        },
        Tool {
            name: "glob".to_string(),
            description: "Find files matching a glob pattern.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string" }
                },
                "required": ["pattern"]
            }),
            requires_confirmation: false,
        },
        Tool {
            name: "fetch".to_string(),
            description: "Fetch a URL and return its text content.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": { "type": "string" }
                },
                "required": ["url"]
            }),
            requires_confirmation: true,
        },
    ]
}

/// Format a tool call for display
pub fn format_tool_call(call: &ToolCall) -> String {
    let args: serde_json::Value = serde_json::from_str(&call.function.arguments)
        .unwrap_or_else(|_| serde_json::json!({}));

    let arg_str = match call.function.name.as_str() {
        "bash" | "fetch" => args.to_string(),
        _ => args.to_string(),
    };

    format!("{}{}", call.function.name.cyan().bold(), arg_str)
}

/// Execute a tool call locally (for tools that don't need server roundtrip)
pub fn execute_local(tool: &str, args: &serde_json::Value) -> Result<String> {
    use anyhow::Context;
    match tool {
        "bash" => {
            let cmd = args["command"].as_str()
                .context("missing 'command' arg")?;
            let _timeout = args["timeout_secs"].as_u64().unwrap_or(30);

            let output = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            let result = if !stderr.is_empty() {
                format!("{}\nstderr: {}", stdout, stderr)
            } else {
                stdout
            };
            let truncated = if result.len() > 10_000 { format!("{}...", &result[..10_000]) } else { result };
            Ok(truncated)
        }
        "read" => {
            let path = args["path"].as_str().context("missing 'path' arg")?;
            std::fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("read failed: {}", e))
        }
        _ => anyhow::bail!("tool '{}' not executable locally — use server", tool),
    }
}
