//! Share sessions via the Elixcode share pages API.

use crate::api::ApiClient;
use crate::config::Config;
use anyhow::Result;
use colored::*;

pub async fn create(cfg: Config, session_id: String, _json: bool) -> Result<()> {
    let client = ApiClient::new(cfg)?;

    let url = client.share_session(&session_id).await?;

    println!();
    println!("{} Session shared!", "✓".green().bold());
    println!();
    println!("  Share URL: {}", url.bright_white().underline());
    println!();
    println!("  View at: {}/share/{}", client.cfg.api_base(), session_id.split('/').last().unwrap_or(&session_id));
    println!();

    Ok(())
}
