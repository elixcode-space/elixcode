//! Skills system: extensible, markdown-defined agent capabilities.
//!
//! Skills are markdown files in ~/.elixcode/skills/ that define:
//!   - What the skill does
//!   - Instructions for the LLM
//!   - Example usage
//!   - Tools available

use crate::config::Config;
use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub content: String,
    pub tags: Vec<String>,
    pub enabled: bool,
}

impl Skill {
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("reading skill {}", path.display()))?;

        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let first_line = content.lines().next().unwrap_or("").trim();
        let description = if first_line.starts_with("# ") {
            first_line[2..].to_string()
        } else {
            first_line.to_string()
        };

        let tags: Vec<String> = content
            .lines()
            .filter(|l| l.starts_with("tag:"))
            .map(|l| l.trim_start_matches("tag:").trim().to_string())
            .collect();

        Ok(Self {
            name,
            description,
            content,
            tags,
            enabled: true,
        })
    }
}

pub fn skills_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("no home dir")?;
    Ok(home.join(".elixcode").join("skills"))
}

pub async fn handle(_cfg: Config, action: Option<crate::SkillsAction>, _json: bool) -> Result<()> {
    let skills_path = skills_dir()?;

    match action {
        Some(crate::SkillsAction::List) => {
            list_skills(&skills_path)?;
        }
        Some(crate::SkillsAction::Load { name }) => {
            load_skill(&skills_path, &name)?;
        }
        Some(crate::SkillsAction::Search { query }) => {
            search_skills(&skills_path, &query)?;
        }
        Some(crate::SkillsAction::Create { name }) => {
            create_skill(&skills_path, &name)?;
        }
        None => {
            list_skills(&skills_path)?;
        }
    }
    Ok(())
}

fn list_skills(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
        println!("Skills directory created at {}", path.display());
        println!("Add markdown skills to this directory.");
        return Ok(());
    }

    let entries: Vec<_> = fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md" || ext == "mdx"))
        .collect();

    if entries.is_empty() {
        println!("No skills found in {}", path.display());
        return Ok(());
    }

    println!("{}", "Skills".cyan().bold());
    println!("{}", "─".repeat(60).cyan());
    for entry in entries {
            if let Ok(skill) = Skill::load(&entry.path()) {
            println!("  {:<20} {}", skill.name.green(), skill.description);
            if !skill.tags.is_empty() {
                println!("    tags: {}", skill.tags.join(", ").bright_black());
            }
        }
    }
    println!();

    Ok(())
}

fn load_skill(path: &PathBuf, name: &str) -> Result<()> {
    let skill_path = path.join(format!("{}.md", name));
    if !skill_path.exists() {
        anyhow::bail!("skill '{}' not found at {}", name, skill_path.display());
    }

    let skill = Skill::load(&skill_path)?;
    println!("{}", "─".repeat(60).cyan());
    println!("Skill: {}  ({})", skill.name.green(), skill.description);
    println!("{}", "─".repeat(60).cyan());
    println!("{}", skill.content);
    Ok(())
}

fn search_skills(path: &PathBuf, query: &[String]) -> Result<()> {
    let query_str = query.join(" ").to_lowercase();

    if !path.exists() {
        println!("No skills directory found.");
        return Ok(());
    }

    let entries: Vec<_> = fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md" || ext == "mdx"))
        .filter(|e| {
            if let Ok(skill) = Skill::load(&e.path()) {
                skill.description.to_lowercase().contains(&query_str)
                    || skill.content.to_lowercase().contains(&query_str)
                    || skill.tags.iter().any(|t| t.to_lowercase().contains(&query_str))
            } else {
                false
            }
        })
        .collect();

    if entries.is_empty() {
        println!("No skills match '{}'", query_str);
    } else {
        for entry in entries {
        if let Ok(skill) = Skill::load(&entry.path()) {
                println!("{}: {}", skill.name.green(), skill.description);
            }
        }
    }
    Ok(())
}

fn create_skill(path: &PathBuf, name: &str) -> Result<()> {
    fs::create_dir_all(path)?;
    let skill_path = path.join(format!("{}.md", name));

    if skill_path.exists() {
        anyhow::bail!("skill '{}' already exists at {}", name, skill_path.display());
    }

    let content = format!(
        "# {}\n\nDescribe what this skill does.\n\n## Instructions\n\nDetailed instructions for the agent.\n\n## Examples\n\nExample usage.\n",
        name
    );

    fs::write(&skill_path, &content)?;
    println!("{} Created skill '{}' at {}", "✓".green(), name, skill_path.display());
    Ok(())
}
