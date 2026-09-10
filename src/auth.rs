//! Authentication: JWT login, API key, OAuth, token refresh.
//!
//! Supports:
//!   - API key auth (via ELIXCODE_API_KEY)
//!   - JWT Bearer tokens (from login)
//!   - Token auto-refresh
//!   - Stored in ~/.elixcode/auth.toml

use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const AUTH_VERSION: &str = "1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthState {
    pub version: String,
    pub api_key: Option<String>,
    pub jwt: Option<String>,
    pub refresh_token: Option<String>,
    pub user: Option<UserInfo>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub plan: String,
    pub token_limit: i64,
    pub tokens_used_today: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub plan: String,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
}

impl AuthState {
    /// Load auth state from ~/.elixcode/auth.toml
    pub fn load() -> Result<Self> {
        let path = auth_file_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("reading auth file {}", path.display()))?;
            let state: AuthState = toml::from_str(&content)
                .with_context(|| format!("parsing auth file {}", path.display()))?;
            Ok(state)
        } else {
            Ok(AuthState {
                version: AUTH_VERSION.to_string(),
                api_key: None,
                jwt: None,
                refresh_token: None,
                user: None,
                expires_at: None,
            })
        }
    }

    /// Save auth state
    pub fn save(&self) -> Result<()> {
        let path = auth_file_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Create a JWT token (HS256) for local testing / self-hosted
    pub fn create_jwt(&mut self, user_id: &str, email: &str, plan: &str, secret: &str) -> Result<()> {
        let now = Utc::now();
        let exp = now + Duration::hours(24);

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            plan: plan.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            iss: "elixcode".to_string(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )?;

        self.jwt = Some(token.clone());
        self.user = Some(UserInfo {
            id: user_id.to_string(),
            email: email.to_string(),
            name: None,
            plan: plan.to_string(),
            token_limit: 1_000_000,
            tokens_used_today: 0,
        });
        self.expires_at = Some(exp.to_rfc3339());
        self.save()?;
        Ok(())
    }

    /// Validate a JWT token
    pub fn validate_jwt(token: &str, secret: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .context("invalid JWT token")?;

        Ok(token_data.claims)
    }

    /// Refresh JWT token using refresh token
    pub async fn refresh(&mut self, server: &str, _secret: &str) -> Result<()> {
        let refresh = self.refresh_token
            .as_ref()
            .context("no refresh token available")?;

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/auth/refresh", server))
            .json(&serde_json::json!({ "refresh_token": refresh }))
            .send()
            .await
            .context("refresh request failed")?;

        if resp.status().is_success() {
            let body: serde_json::Value = resp.json().await?;
            if let Some(new_jwt) = body.get("token") {
                self.jwt = Some(new_jwt.as_str().unwrap().to_string());
                self.expires_at = Some(
                    (Utc::now() + Duration::hours(24)).to_rfc3339()
                );
                self.save()?;
            }
            Ok(())
        } else {
            anyhow::bail!("token refresh failed: {}", resp.status());
        }
    }

    /// Check if JWT is expired
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = &self.expires_at {
            if let Ok(exp_dt) = chrono::DateTime::parse_from_rfc3339(exp) {
                return exp_dt.with_timezone(&Utc) < Utc::now();
            }
        }
        false
    }
}

fn auth_file_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("no home dir")?;
    Ok(home.join(".elixcode").join("auth.toml"))
}

/// Handle auth subcommands
pub async fn handle(
    cfg: crate::config::Config,
    action: Option<crate::AuthAction>,
    _json: bool,
) -> anyhow::Result<()> {
    use crate::AuthAction;
    match action {
        Some(AuthAction::Login) => login(&cfg).await,
        Some(AuthAction::Logout) => {
            logout();
            Ok(())
        }
        Some(AuthAction::Whoami) => {
            whoami(&cfg);
            Ok(())
        }
        Some(AuthAction::Refresh) => {
            let mut auth = AuthState::load()?;
            auth.refresh(&cfg.server, "your-jwt-secret").await?;
            println!("Token refreshed");
            Ok(())
        }
        None => {
            whoami(&cfg);
            Ok(())
        }
    }
}

async fn login(cfg: &crate::config::Config) -> anyhow::Result<()> {
    // Try API key login first
    if let Some(key) = &cfg.api_key {
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("{}/auth/verify", cfg.api_base()))
            .header("Authorization", format!("Bearer {}", key))
            .send()
            .await?;

        if resp.status().is_success() {
            let mut auth = AuthState::load()?;
            auth.api_key = Some(key.clone());
            auth.save()?;
            println!("Logged in with API key");
            return Ok(());
        }
    }

    // Interactive: prompt for API key
    println!("Enter your API key:");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let key = input.trim().to_string();

    if key.is_empty() {
        anyhow::bail!("API key cannot be empty");
    }

    let mut auth = AuthState::load()?;
    auth.api_key = Some(key);
    auth.save()?;
    println!("Logged in successfully");
    Ok(())
}

fn logout() {
    let mut auth = AuthState::load().unwrap_or_else(|_| AuthState {
        version: AUTH_VERSION.to_string(),
        api_key: None,
        jwt: None,
        refresh_token: None,
        user: None,
        expires_at: None,
    });
    auth.api_key = None;
    auth.jwt = None;
    auth.refresh_token = None;
    auth.user = None;
    auth.expires_at = None;
    let _ = auth.save();
    println!("Logged out");
}

fn whoami(cfg: &crate::config::Config) {
    let auth = AuthState::load().unwrap_or_else(|_| AuthState {
        version: AUTH_VERSION.to_string(),
        api_key: None,
        jwt: None,
        refresh_token: None,
        user: None,
        expires_at: None,
    });

    if let Some(ref user) = auth.user {
        println!("User: {}", user.email);
        println!("Plan: {}", user.plan);
        println!("ID: {}", user.id);
    } else if auth.api_key.is_some() {
        println!("Logged in with API key (server: {})", cfg.server);
    } else {
        println!("Not logged in. Run `elix auth login`");
    }
}
