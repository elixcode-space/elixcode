type RequestOpts = {
  method?: string;
  headers?: Record<string, string>;
  body?: string;
  stream?: boolean;
};

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

export class APIError extends Error {
  constructor(message: string, public status: number) {
    super(message);
    this.name = 'APIError';
  }
}

export class Client {
  private token: string | null = null;
  private refreshTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(
    public server: string,
    public apiKey: string | undefined,
  ) {}

  setToken(token: string | null): void {
    this.token = token;
  }

  getToken(): string | null {
    return this.token;
  }

  private getAuthHeader(): Record<string, string> {
    if (this.token) return { Authorization: `Bearer ${this.token}` };
    if (this.apiKey) return { Authorization: `Bearer ${this.apiKey}` };
    return {};
  }

  private async request<T>(path: string, opts: RequestOpts = {}): Promise<T> {
    const url = `${this.server}${path}`;
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...this.getAuthHeader(),
      ...(opts.headers as Record<string, string> | undefined),
    };

    const response = await fetch(url, { ...opts, headers });

    if (response.status === 401) {
      this.token = null;
      throw new APIError('Unauthorized — check your API key', 401);
    }

    if (!response.ok) {
      const error = await response.json().catch(() => ({})) as { error?: { message?: string }; message?: string };
      throw new APIError(
        error.error?.message || error.message || response.statusText,
        response.status,
      );
    }

    if (response.status === 204) return {} as T;
    return response.json() as Promise<T>;
  }

  private post<T>(path: string, body: unknown = {}): Promise<T> {
    return this.request<T>(path, {
      method: 'POST',
      body: JSON.stringify(body),
    });
  }

  private get<T>(path: string): Promise<T> {
    return this.request<T>(path, { method: 'GET' });
  }

  del<T>(path: string): Promise<T> {
    return this.request<T>(path, { method: 'DELETE' });
  }

  async login(apiKey: string): Promise<AuthResponse> {
    const response = await this.post<AuthResponse>('/v1/auth/login', {
      api_key: apiKey,
    });
    this.token = response.jwt;
    this.apiKey = undefined;
    if (response.expires_at) {
      this.setupRefresh(response.expires_at);
    }
    return response;
  }

  async refresh(refreshToken: string): Promise<{ jwt: string; expires_at: string }> {
    const response = await this.post<{ jwt: string; expires_at: string }>(
      '/v1/auth/refresh',
      { refresh_token: refreshToken },
    );
    this.token = response.jwt;
    this.setupRefresh(response.expires_at);
    return response;
  }

  private setupRefresh(expiresAt: string): void {
    if (this.refreshTimer) clearTimeout(this.refreshTimer);
    const exp = new Date(expiresAt).getTime();
    const delay = exp - Date.now() - 60_000;
    if (delay > 0) {
      this.refreshTimer = setTimeout(() => {
        void this.refresh('');
      }, delay);
    }
  }

  async health(): Promise<HealthResponse> {
    return this.get('/health');
  }

  async ready(): Promise<HealthResponse> {
    return this.get('/ready');
  }

  async listModels(): Promise<ModelInfo[]> {
    const response = await this.get<{ object: string; data: ModelInfo[] }>('/v1/models');
    return response.data;
  }

  async listProviders(): Promise<{ name: string; models: string[] }[]> {
    const response = await this.get<{ providers: { name: string; models: string[] }[] }>('/providers');
    return response.providers;
  }

  async usage(): Promise<Usage> {
    return this.get('/usage');
  }

  async listAPIKeys(): Promise<APIKey[]> {
    const response = await this.get<{ keys: APIKey[] }>('/v1/api-keys');
    return response.keys;
  }

  async createAPIKey(name: string, scopes: string[] = ['read', 'write']): Promise<APIKey> {
    return this.post<APIKey>('/v1/api-keys', { name, scopes });
  }

  async deleteAPIKey(id: string): Promise<void> {
    await this.del(`/v1/api-keys/${id}`);
  }

  async listHarnesses(): Promise<Harness[]> {
    const response = await this.get<{ harnesses: Harness[] }>('/v1/harnesses');
    return response.harnesses;
  }

  async createHarness(config: Record<string, unknown>): Promise<{ id: string }> {
    return this.post('/v1/harnesses', config);
  }

  async getHarness(id: string): Promise<Harness> {
    return this.get(`/v1/harnesses/${id}`);
  }

  async runTurn(harness: string, input: string, stream = false): Promise<Turn> {
    return this.post(`/v1/harnesses/${harness}/turn`, { input, stream });
  }

  async checkInstall(harness: string): Promise<{ installed: boolean }> {
    return this.get(`/v1/harnesses/${harness}/install`);
  }

  async routeTask(input: string): Promise<{ route: string }> {
    return this.post('/v1/harness/route', { input });
  }

  async decideTask(input: string): Promise<unknown> {
    return this.post('/v1/harness/decide', { input });
  }

  async listPatterns(): Promise<LoopPattern[]> {
    return this.get('/v1/loops/patterns');
  }

  async startLoop(pattern: string, interval?: number): Promise<{ id: string }> {
    return this.post('/v1/loops', { pattern, interval });
  }

  async stopLoop(id: string): Promise<void> {
    await this.post(`/v1/loops/${id}/stop`);
  }

  async loopStatus(): Promise<unknown> {
    return this.get('/v1/loops/status');
  }

  async loopCost(): Promise<unknown> {
    return this.get('/v1/loops/cost');
  }

  async listSessions(): Promise<Session[]> {
    return this.get('/v1/sessions');
  }

  async saveSession(id?: string): Promise<{ id: string }> {
    return this.post('/v1/sessions/save', { id });
  }

  async loadSession(id: string): Promise<Session> {
    return this.get(`/v1/sessions/${id}`);
  }

  async exportSession(id: string, format: 'json' | 'md' = 'json'): Promise<unknown> {
    return this.post(`/v1/sessions/${id}/export`, { format });
  }

  async deleteSession(id: string): Promise<void> {
    await this.del(`/v1/sessions/${id}`);
  }

  async fleetWorkers(): Promise<unknown[]> {
    return this.get('/v1/fleet/workers');
  }

  async fleetHardware(): Promise<unknown> {
    return this.get('/v1/fleet/hardware');
  }

  async fleetMetrics(): Promise<unknown> {
    return this.get('/v1/fleet/metrics');
  }

  async agentAgents(): Promise<unknown[]> {
    return this.get('/agents');
  }

  async agentSummary(): Promise<AgentSummary> {
    return this.get('/agents/summary');
  }

  async agentReport(since = '7d', by = 'model'): Promise<unknown> {
    return this.get(`/agents/report?since=${since}&by=${by}`);
  }

  async agentTrace(id: string): Promise<unknown> {
    return this.get(`/agents/trace/${id}`);
  }

  async chat(model: string, messages: unknown[], stream = false): Promise<unknown> {
    return this.post('/v1/chat/completions', { model, messages, stream });
  }
}
