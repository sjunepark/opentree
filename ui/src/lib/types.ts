// Types matching the runner's data structures

export interface Node {
  id: string;
  order: number;
  title: string;
  goal: string;
  acceptance: string[];
  next: 'execute' | 'decompose';
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

export interface RunnerConfig {
  max_iterations?: number;
  max_attempts_default?: number;
  iteration_timeout_secs?: number;
  iteration_output_limit?: number;
  guard_command?: string;
  [key: string]: unknown;
}

export interface StreamEvent {
  type?: string;
  [key: string]: unknown;
}

export interface StreamUsage {
  input_tokens?: number;
  cached_input_tokens?: number;
  output_tokens?: number;
  [key: string]: unknown;
}

export type UiStreamEvent =
  | {
      kind: 'turn_started';
      id: string;
      raw: StreamEvent;
    }
  | {
      kind: 'turn_completed';
      id: string;
      usage?: StreamUsage;
      raw: StreamEvent;
    }
  | {
      kind: 'command';
      id: string;
      status: 'running' | 'completed';
      command: string;
      output: string;
      exitCode: number | null;
      rawStart?: StreamEvent;
      rawEnd?: StreamEvent;
    }
  | {
      kind: 'reasoning';
      id: string;
      text: string;
      raw: StreamEvent;
    }
  | {
      kind: 'agent_message';
      id: string;
      text: string;
      raw: StreamEvent;
    }
  | {
      kind: 'raw';
      id: string;
      raw: StreamEvent;
    }
  | {
      kind: 'unknown';
      id: string;
      raw: StreamEvent;
    };

export type NormalizeMode = 'semantic' | 'raw';

// Unified timeline entry for the history view
export interface TimelineEntry {
  run_id: string;
  iter: number;
  node_id: string;
  status: 'done' | 'retry' | 'decomposed' | 'running';
  guard: 'pass' | 'fail' | 'skipped' | null;
}

// Selection state
export type SelectionType = 'node' | 'iteration' | null;
