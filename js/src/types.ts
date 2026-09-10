export interface AuthResponse {
  jwt: string;
  expires_at: string;
  refresh_token?: string;
}

export interface HealthResponse {
  status: string;
  gateway: string;
  version: string;
  cache?: { hits: number };
}

export interface ModelInfo {
  id: string;
  name: string;
  provider: string;
  owner?: string;
  context_length?: number;
  pricing?: { prompt: number; completion: number };
}

export interface APIKey {
  id: string;
  name: string;
  created_at: string;
  last_used: string | null;
  scopes: string[];
  secret?: string;
}

export interface Harness {
  id: string;
  name: string;
  base: string;
  type: string;
  model: string;
  installed: boolean;
  enabled: boolean;
  instructions: string;
  limits: { max_turns: number; max_tokens: number };
}

export interface Turn {
  turn_id: string;
  status: 'in_progress' | 'completed' | 'failed' | 'error';
}

export interface LoopPattern {
  name: string;
  cadence: string;
  tier: 'L0' | 'L1' | 'L2' | 'L3';
  harness: string;
  cost: string;
  description: string;
}

export interface Session {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
  model: string;
  provider: string;
  message_count: number;
}

export interface Usage {
  total_tokens: number;
  total_cost_usd: number;
  period: string;
}

export interface AgentSummary {
  total_agents: number;
  total_sessions: number;
  total_tokens: number;
  total_cost_usd: number;
  orphaned_mcp: number;
}
