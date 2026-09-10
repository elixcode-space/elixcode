//! Provider management: OpenRouter, Anthropic, OpenAI, self-hosted.

use crate::api::ApiClient;
use crate::config::Config;
use anyhow::Result;
use colored::*;
use std::io::Write;

pub async fn handle(cfg: Config, action: Option<crate::ProvidersAction>, _json: bool) -> Result<()> {
    let client = ApiClient::new(cfg)?;

    match action {
        Some(crate::ProvidersAction::List) => {
            list_providers(&client).await?;
        }
        Some(crate::ProvidersAction::Add { name, key, base }) => {
            add_provider(&client, &name, &key, base.as_deref()).await?;
        }
        Some(crate::ProvidersAction::Remove { name }) => {
            remove_provider(&client, &name).await?;
        }
        Some(crate::ProvidersAction::Test { name }) => {
            test_provider(&client, &name).await?;
        }
        None => {
            list_providers(&client).await?;
        }
    }
    Ok(())
}

async fn list_providers(client: &ApiClient) -> Result<()> {
    let providers = client.list_providers().await?;

    if providers.is_empty() {
        // Fall back to local config
        println!("{}", "Configured providers:".cyan().bold());
        for (name, prov) in &client.cfg.providers {
            println!(
                "  {:<15} {}  model={}",
                name.green(),
                if prov.enabled { "✓ enabled " } else { "✗ disabled" },
                prov.default_model.as_deref().unwrap_or("—")
            );
        }
        return Ok(());
    }

    println!("{}", "Providers".cyan().bold());
    println!("{}", "─".repeat(60).cyan());
    for p in providers {
        println!(
            "  {:<15} {}  {}",
            p.name.green(),
            if p.enabled { "✓ enabled " } else { "✗ disabled" },
            p.default_model.as_deref().unwrap_or("—").bright_black()
        );
    }
    Ok(())
}

async fn add_provider(
    client: &ApiClient,
    name: &str,
    key: &str,
    base: Option<&str>,
) -> Result<()> {
    println!("{} Adding provider '{}'...", "→".cyan(), name);

    let mut cfg = client.cfg.clone();
    cfg.providers.insert(name.to_string(), crate::config::ProviderConfig {
        name: name.to_string(),
        api_key: Some(key.to_string()),
        base_url: base.map(String::from),
        default_model: None,
        enabled: true,
        priority: 50,
    });

    cfg.save()?;
    println!("{} Provider '{}' added and saved to config", "✓".green(), name);
    Ok(())
}

async fn remove_provider(client: &ApiClient, name: &str) -> Result<()> {
    let mut cfg = client.cfg.clone();
    if cfg.providers.remove(name).is_none() {
        anyhow::bail!("provider '{}' not found", name);
    }
    cfg.save()?;
    println!("{} Provider '{}' removed", "✓".green(), name);
    Ok(())
}

async fn test_provider(client: &ApiClient, name: &str) -> Result<()> {
    print!("{} Testing '{}'... ", "→".cyan(), name);
    std::io::stdout().flush()?;

    let providers = client.list_providers().await?;
    let found = providers.iter().find(|p| p.name == name);

    match found {
        Some(p) if p.enabled => {
            println!("{}", "✓ OK".green());
        }
        _ => {
            println!("{}", "✗ Failed or disabled".red());
        }
    }
    Ok(())
}
