//! Model listing with filtering by provider.

use crate::api::{ApiClient, ModelInfo};
use crate::config::Config;
use anyhow::Result;
use colored::*;

pub async fn list(cfg: Config, provider_filter: Option<String>, _json: bool) -> Result<()> {
    let client = ApiClient::new(cfg)?;

    let models = client.list_models().await?;

    if let Some(ref pf) = provider_filter {
        let filtered: Vec<ModelInfo> = models.into_iter()
            .filter(|m| m.id.to_lowercase().contains(&pf.to_lowercase()))
            .collect();

        print_models(&filtered);
    } else {
        print_models(&models);
    }

    Ok(())
}

fn print_models(models: &[ModelInfo]) {
    println!("{}", format!("{} models", models.len()).cyan().bold());
    println!("{}", "─".repeat(60).cyan());

    for m in models {
        let owned_by = m.owned_by.as_deref().unwrap_or("—");
        let created = m.created.map(|ts| {
            chrono::DateTime::from_timestamp(ts as i64, 0)
                .map(|dt| dt.format("%Y-%m").to_string())
                .unwrap_or_default()
        }).unwrap_or_default();

        println!(
            "  {:<45} {:>8}  {}",
            m.id.green(),
            created.bright_black(),
            owned_by.bright_black()
        );
    }
}
