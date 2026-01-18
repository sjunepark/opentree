import type { Node, RunState, RunEntry, IterationMeta, AgentOutput } from './stores';

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
