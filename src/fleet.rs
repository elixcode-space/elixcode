//! Fleet management commands (workers, hardware, deploy, warmup, router).

use crate::api::{ApiClient, DeploymentResponse, HardwareInfo, WorkerInfo};
use crate::config::Config;
use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};

impl ApiClient {
    /// GET /fleet/workers
    pub async fn list_workers(&self) -> Result<Vec<WorkerInfo>> {
        let url = format!("{}/fleet/workers", self.cfg.api_base());
        let resp = self
            .http
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .with_context(|| format!("GET {}", url))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("HTTP {}: {}", status, body);
        }
        Ok(resp.json().await?)
    }

    /// POST /fleet/workers (worker self-registration)
    pub async fn register_worker(&self, name: &str, port: u16) -> Result<WorkerInfo> {
        let url = format!("{}/fleet/workers", self.cfg.api_base());
        let body = serde_json::json!({
            "name": name,
            "target": "worker",
            "port": port,
        });
        let resp = self
            .http
            .post(&url)
            .headers(self.auth_headers())
            .json(&body)
            .send()
            .await
            .with_context(|| format!("POST {}", url))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("HTTP {}: {}", status, body);
        }
        Ok(resp.json().await?)
    }

    /// POST /fleet/deploy
    pub async fn deploy(
        &self,
        target: &str,
        version: &str,
        canary_pct: u8,
    ) -> Result<DeploymentResponse> {
        let url = format!("{}/fleet/deploy", self.cfg.api_base());
        let body = serde_json::json!({
            "target": target,
            "version": version,
            "canary_percent": canary_pct,
        });
        let resp = self
            .http
            .post(&url)
            .headers(self.auth_headers())
            .json(&body)
            .send()
            .await
            .with_context(|| format!("POST {}", url))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("HTTP {}: {}", status, body);
        }
        Ok(resp.json().await?)
    }

    /// GET /hardware
    pub async fn hardware(&self) -> Result<HardwareInfo> {
        let url = format!("{}/hardware", self.cfg.api_base());
        let resp = self
            .http
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .with_context(|| format!("GET {}", url))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("HTTP {}: {}", status, body);
        }
        Ok(resp.json().await?)
    }

    /// GET /warmup/{model}
    pub async fn warmup(&self, model: &str) -> Result<serde_json::Value> {
        let url = format!("{}/warmup/{}", self.cfg.api_base(), model);
        let resp = self
            .http
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .with_context(|| format!("GET {}", url))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("HTTP {}: {}", status, body);
        }
        Ok(resp.json().await?)
    }

    /// POST /router/decide
    pub async fn router_decide(&self, model: &str) -> Result<serde_json::Value> {
        let url = format!("{}/router/decide", self.cfg.api_base());
        let body = serde_json::json!({ "model": model });
        let resp = self
            .http
            .post(&url)
            .headers(self.auth_headers())
            .json(&body)
            .send()
            .await
            .with_context(|| format!("POST {}", url))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("HTTP {}: {}", status, body);
        }
        Ok(resp.json().await?)
    }

    /// GET /metrics (raw text)
    pub async fn metrics(&self) -> Result<String> {
        let url = format!("{}/metrics", self.cfg.api_base());
        let resp = self
            .http
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .with_context(|| format!("GET {}", url))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("HTTP {}: {}", status, body);
        }
        Ok(resp.text().await?)
    }
}

// ─── CLI commands ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, clap::Subcommand, Serialize, Deserialize)]
pub enum FleetAction {
    /// List all fleet workers
    Workers,
    /// Worker self-registration
    Register {
        /// Worker name
        name: String,
        /// Worker port
        #[arg(long, default_value = "0")]
        port: u16,
    },
    /// Trigger a deployment
    Deploy {
        /// Target (gateway|worker|edge)
        #[arg(long, default_value = "gateway")]
        target: String,
        /// Version string
        #[arg(long)]
        version: String,
        /// Canary percentage
        #[arg(long, default_value = "0")]
        canary: u8,
    },
    /// Show hardware info
    Hardware,
    /// Warmup a model
    Warmup {
        /// Model id
        model: String,
    },
    /// Routing decision
    Router {
        /// Model id
        model: String,
    },
    /// Prometheus metrics
    Metrics,
}

pub async fn dispatch(cfg: Config, action: FleetAction, json: bool) -> Result<()> {
    match action {
        FleetAction::Workers => list_workers(cfg, json).await,
        FleetAction::Register { name, port } => register_worker(cfg, name, port, json).await,
        FleetAction::Deploy { target, version, canary } => {
            deploy(cfg, target, version, canary, json).await
        }
        FleetAction::Hardware => hardware(cfg, json).await,
        FleetAction::Warmup { model } => warmup(cfg, model, json).await,
        FleetAction::Router { model } => router(cfg, model, json).await,
        FleetAction::Metrics => metrics(cfg).await,
    }
}

pub async fn list_workers(cfg: Config, json: bool) -> Result<()> {
    let client = ApiClient::new(cfg)?;
    let workers = client.list_workers().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&workers)?);
        return Ok(());
    }

    println!(
        "{} {}",
        "Fleet workers:".bold(),
        workers.len().to_string().yellow()
    );
    println!("{}", "─".repeat(60).dimmed());
    for w in workers {
        print_worker(&w);
    }
    Ok(())
}

pub async fn register_worker(cfg: Config, name: String, port: u16, json: bool) -> Result<()> {
    let client = ApiClient::new(cfg)?;
    let w = client.register_worker(&name, port).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&w)?);
        return Ok(());
    }
    println!("{} Worker \"{}\" registered", "✓".green().bold(), name);
    println!("  id:    {}", w.id);
    Ok(())
}

pub async fn deploy(cfg: Config, target: String, version: String, canary: u8, json: bool) -> Result<()> {
    let client = ApiClient::new(cfg)?;
    let resp = client.deploy(&target, &version, canary).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&resp)?);
        return Ok(());
    }
    println!(
        "{} Deploying {} v{} (canary={}%)",
        "✓".green().bold(),
        target.cyan(),
        version.yellow(),
        canary
    );
    println!("  deployment_id: {}", resp.deployment_id.cyan());
    println!("  status:        {}", resp.status);
    println!("  started_at:    {}", resp.started_at.dimmed());
    Ok(())
}

pub async fn hardware(cfg: Config, json: bool) -> Result<()> {
    let client = ApiClient::new(cfg)?;
    let h = client.hardware().await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&h)?);
        return Ok(());
    }
    println!("{}", "Hardware:".bold());
    if let Some(r) = h.recommended {
        println!("  recommended: {}", r.green());
    }
    for b in h.backends {
        let name = b.get("name").and_then(|v| v.as_str()).unwrap_or("?");
        let available = b.get("available").and_then(|v| v.as_bool()).unwrap_or(false);
        let dev_count = b.get("device_count").and_then(|v| v.as_u64()).unwrap_or(0);
        let vram = b.get("vram_mb").and_then(|v| v.as_u64()).unwrap_or(0);
        let icon = if available { "✓".green().bold() } else { "✗".red() };
        println!("  {} {:<6} devices={} vram_mb={}", icon, name, dev_count, vram);
    }
    Ok(())
}

pub async fn warmup(cfg: Config, model: String, json: bool) -> Result<()> {
    let client = ApiClient::new(cfg)?;
    let resp = client.warmup(&model).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&resp)?);
        return Ok(());
    }
    println!("{} Warmup state for {}", "•".cyan(), model.yellow());
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

pub async fn router(cfg: Config, model: String, json: bool) -> Result<()> {
    let client = ApiClient::new(cfg)?;
    let resp = client.router_decide(&model).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&resp)?);
        return Ok(());
    }
    println!("{} Routing decision for {}", "•".cyan(), model.yellow());
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

pub async fn metrics(cfg: Config) -> Result<()> {
    let client = ApiClient::new(cfg)?;
    let text = client.metrics().await?;
    print!("{}", text);
    Ok(())
}

fn print_worker(w: &WorkerInfo) {
    println!("  {} {}", "id:".dimmed(), w.id.cyan());
    if let Some(name) = &w.name {
        println!("  {}    {}", "name:".dimmed(), name);
    }
    if let Some(status) = &w.status {
        let icon = if status == "healthy" { "●".green() } else { "●".yellow() };
        println!("  {}  {} {}", "status:".dimmed(), icon, status);
    }
    if let Some(t) = &w.target {
        println!("  {}  {}", "target:".dimmed(), t);
    }
    if let Some(v) = &w.version {
        println!("  {} {}", "version:".dimmed(), v);
    }
    if let (Some(host), Some(port)) = (&w.host, &w.port) {
        println!("  {}    {}:{}", "addr:".dimmed(), host, port);
    }
    println!("{}", "─".repeat(60).dimmed());
}
