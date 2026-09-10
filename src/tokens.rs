//! Token counting and cost estimation.
//!
//! Tracks:
//!   - Input/output tokens per request
//!   - Cumulative cost (OpenRouter pricing)
//!   - Daily/monthly limits

use crate::api::Usage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenCounter {
    pub requests: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost_usd: f64,
    pub by_model: HashMap<String, ModelStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelStats {
    pub requests: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_usd: f64,
}

impl TokenCounter {
    pub fn record(&mut self, model: &str, usage: &Usage, cost_per_million: f64) {
        let input = usage.prompt_tokens.unwrap_or(0) as u64;
        let output = usage.completion_tokens.unwrap_or(0) as u64;
        let total = input + output;
        let cost = (total as f64) * cost_per_million / 1_000_000.0;

        self.requests += 1;
        self.total_input_tokens += input;
        self.total_output_tokens += output;
        self.total_cost_usd += cost;

        let stats = self.by_model.entry(model.to_string()).or_default();
        stats.requests += 1;
        stats.input_tokens += input;
        stats.output_tokens += output;
        stats.cost_usd += cost;
    }

    pub fn summary(&self) -> String {
        format!(
            "Requests: {} | Input: {} tokens | Output: {} tokens | Cost: ${:.4}",
            self.requests,
            self.total_input_tokens,
            self.total_output_tokens,
            self.total_cost_usd
        )
    }
}

// OpenRouter-style pricing (approximate, USD per million tokens)
pub fn cost_per_million(model: &str) -> f64 {
    match model {
        // Anthropic
        m if m.contains("claude-3.5-sonnet") => 3.0,
        m if m.contains("claude-3-opus") => 15.0,
        m if m.contains("claude-3-sonnet") => 3.0,
        m if m.contains("claude-3-haiku") => 0.25,

        // OpenAI
        m if m.contains("gpt-4o") => 5.0,
        m if m.contains("gpt-4-turbo") => 10.0,
        m if m.contains("gpt-4") => 30.0,
        m if m.contains("gpt-3.5-turbo") => 0.5,

        // OpenRouter / others
        m if m.contains("claude-3.5-sonnet") => 3.0,
        m if m.contains("o1-preview") => 15.0,
        m if m.contains("o1-mini") => 3.0,

        // Self-hosted (electricity cost only)
        m if m.contains("llama") || m.contains("mistral") || m.contains("qwen") => 0.001,

        // Default
        _ => 1.0,
    }
}

pub fn estimate_tokens_for_text(text: &str) -> usize {
    // Rough estimate: ~4 chars per token for English
    (text.len() / 4).max(1)
}
