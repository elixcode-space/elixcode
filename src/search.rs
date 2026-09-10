//! Search across the workspace: semantic + regex.
//!
//! Semantic search uses embeddings from the self-hosted model or OpenAI.
//! Regex search uses ripgrep-style pattern matching.

use crate::config::Config;
use anyhow::Result;
use colored::*;
use regex::Regex;
use std::fs;
use walkdir::WalkDir;

pub async fn run(cfg: Config, query: Vec<String>, path_only: bool, _json: bool) -> Result<()> {
    let query_str = query.join(" ");
    let workspace = &cfg.workspace;

    if !workspace.exists() {
        anyhow::bail!("workspace not found: {}", workspace.display());
    }

    println!("Searching in {} for: {}", workspace.display(), query_str.cyan());

    if path_only {
        search_path(&query_str, workspace)?;
    } else {
        search_content(&query_str, workspace)?;
    }

    Ok(())
}

fn search_path(query: &str, workspace: &std::path::Path) -> Result<()> {
    let re = Regex::new(query).map_err(|e| anyhow::anyhow!("invalid regex: {}", e))?;

    let mut count = 0;
    for entry in WalkDir::new(workspace)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
    {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if re.is_match(name) {
                println!("  {}", path.display().to_string().green());
                count += 1;
            }
        }
    }

    println!("\n{} files matched", count);
    Ok(())
}

fn search_content(query: &str, workspace: &std::path::Path) -> Result<()> {
    let re = Regex::new(query).map_err(|e| anyhow::anyhow!("invalid regex: {}", e))?;

    let mut count = 0;
    for entry in WalkDir::new(workspace)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        // Skip binary files
        if is_binary(path) {
            continue;
        }

        if let Ok(content) = fs::read_to_string(path) {
            for (line_no, line) in content.lines().enumerate() {
                if re.is_match(line) {
                    let line_str = line.trim();
                    let path_str = path.display().to_string();
                    if line_str.len() > 120 {
                        println!("{}:{}: {}", path_str.green(), line_no + 1,
                            format!("{}...", &line_str[..120]).yellow());
                    } else {
                        println!("{}:{}: {}", path_str.green(), line_no + 1, line_str.yellow());
                    }
                    count += 1;
                    if count >= 200 {
                        println!("\n(truncated at 200 results)");
                        break;
                    }
                }
            }
        }
    }

    println!("\n{} matches", count);
    Ok(())
}

fn is_binary(path: &std::path::Path) -> bool {
    use std::io::Read;
    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut buf = [0u8; 8192];
    let n = file.read(&mut buf).unwrap_or(0);
    buf[..n].contains(&0)
}
