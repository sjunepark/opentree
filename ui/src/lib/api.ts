import type { Node, RunState, RunEntry, IterationMeta, AgentOutput, RunnerConfig } from './stores';

const API_BASE = '/api';

async function fetchJson<T>(path: string): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`);
  if (!response.ok) {
    throw new Error(`API error: ${response.status} ${response.statusText}`);
  }
  return response.json();
}

export async function fetchTree(): Promise<Node> {
  return fetchJson<Node>('/tree');
}

export async function fetchRunState(): Promise<RunState> {
  return fetchJson<RunState>('/run-state');
}

export async function fetchIterations(): Promise<{ runs: RunEntry[] }> {
  return fetchJson<{ runs: RunEntry[] }>('/iterations');
}

export interface IterationDetail {
  meta: IterationMeta;
  output: AgentOutput;
}

export async function fetchIteration(runId: string, iter: number): Promise<IterationDetail> {
  return fetchJson<IterationDetail>(`/iterations/${runId}/${iter}`);
}

export async function fetchGuardLog(runId: string, iter: number): Promise<string> {
  const response = await fetch(`${API_BASE}/iterations/${runId}/${iter}/guard.log`);
  if (!response.ok) {
    throw new Error(`API error: ${response.status} ${response.statusText}`);
  }
  return response.text();
}

export interface StreamEvent {
  type?: string;
  [key: string]: unknown;
}

export async function fetchStream(runId: string, iter: number, offset?: number): Promise<StreamEvent[]> {
  const url = offset !== undefined
    ? `${API_BASE}/iterations/${runId}/${iter}/stream?offset=${offset}`
    : `${API_BASE}/iterations/${runId}/${iter}/stream`;
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`API error: ${response.status} ${response.statusText}`);
  }
  const text = await response.text();
  if (!text.trim()) {
    return [];
  }
  return text.trim().split('\n').map(line => JSON.parse(line));
}

export async function fetchConfig(): Promise<RunnerConfig> {
  return fetchJson<RunnerConfig>('/config');
}

async function fetchText(path: string): Promise<string> {
  const response = await fetch(`${API_BASE}${path}`);
  if (!response.ok) {
    throw new Error(`API error: ${response.status} ${response.statusText}`);
  }
  return response.text();
}

export async function fetchAssumptions(): Promise<string> {
  return fetchText('/assumptions');
}

export async function fetchQuestions(): Promise<string> {
  return fetchText('/questions');
}
