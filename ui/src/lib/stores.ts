import { writable, type Writable } from 'svelte/store';

// Types matching the runner's data structures
export interface Node {
  id: string;
  order: number;
  title: string;
  goal: string;
  acceptance: string[];
  passes: boolean;
  attempts: number;
  max_attempts: number;
  children: Node[];
}

export interface RunState {
  run_id: string | null;
  next_iter: number;
  last_status: 'done' | 'retry' | 'decomposed' | null;
  last_summary: string | null;
  last_guard: 'pass' | 'fail' | 'skipped' | null;
}

export interface IterationMeta {
  run_id: string;
  iter: number;
  node_id: string;
  status: 'done' | 'retry' | 'decomposed';
  guard: 'pass' | 'fail' | 'skipped';
  started_at: string | null;
  ended_at: string | null;
  duration_ms: number | null;
}

export interface AgentOutput {
  status: 'done' | 'retry' | 'decomposed';
  summary: string;
}

export interface RunEntry {
  run_id: string;
  iterations: number[];
}

// Stores
export const tree: Writable<Node | null> = writable(null);
export const runState: Writable<RunState | null> = writable(null);
export const runs: Writable<RunEntry[]> = writable([]);
export const selectedNode: Writable<Node | null> = writable(null);
export const selectedIteration: Writable<{ run_id: string; iter: number } | null> = writable(null);
export const sseConnected: Writable<boolean> = writable(false);
export const loading: Writable<boolean> = writable(true);
export const error: Writable<string | null> = writable(null);
