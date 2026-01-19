/**
 * Test utilities for store testing.
 * Provides reset functions and factories for mock data.
 */

import type { Node, TimelineEntry, IterationMeta } from '../types';
import {
  connection,
  data,
  iterations,
  stream,
  selection,
  timer,
  rightPanel,
} from '../stores.svelte';

/**
 * Resets all stores to their initial state.
 * Call this in beforeEach() to ensure test isolation.
 */
export function resetAllStores(): void {
  // Connection
  connection.sseConnected = false;
  connection.loading = true;
  connection.error = null;
  connection.staticMode = false;
  connection.lastEventTime = null;

  // Data
  data.tree = null;
  data.runState = null;
  data.config = null;
  data.assumptions = '';
  data.questions = '';

  // Iterations
  iterations.entries = [];
  iterations.metaCache.clear();

  // Stream
  stream.events = [];
  stream.offset = 0;
  stream.activeRunId = null;
  stream.activeIter = null;
  stream.autoScroll = true;

  // Selection
  selection.type = null;
  selection.nodeId = null;
  selection.node = null;
  selection.iterKey = null;

  // Timer
  timer.startTime = null;
  timer.elapsed = 0;

  // Right panel
  rightPanel.activeTab = 'details';
}

let nodeCounter = 0;

/**
 * Creates a mock Node with default values.
 * All properties can be overridden via the partial argument.
 */
export function createMockNode(partial: Partial<Node> = {}): Node {
  nodeCounter++;
  return {
    id: partial.id ?? `node-${nodeCounter}`,
    order: partial.order ?? nodeCounter,
    title: partial.title ?? `Test Node ${nodeCounter}`,
    goal: partial.goal ?? 'Test goal',
    acceptance: partial.acceptance ?? [],
    passes: partial.passes ?? false,
    attempts: partial.attempts ?? 0,
    max_attempts: partial.max_attempts ?? 3,
    children: partial.children ?? [],
  };
}

/**
 * Creates a mock TimelineEntry with default values.
 * All properties can be overridden via the partial argument.
 */
export function createMockEntry(partial: Partial<TimelineEntry> = {}): TimelineEntry {
  return {
    run_id: partial.run_id ?? 'run-1',
    iter: partial.iter ?? 0,
    node_id: partial.node_id ?? 'node-1',
    status: partial.status ?? 'done',
    guard: partial.guard ?? 'pass',
  };
}

/**
 * Creates a mock IterationMeta with default values.
 * All properties can be overridden via the partial argument.
 */
export function createMockIterationMeta(partial: Partial<IterationMeta> = {}): IterationMeta {
  return {
    run_id: partial.run_id ?? 'run-1',
    iter: partial.iter ?? 0,
    node_id: partial.node_id ?? 'node-1',
    status: partial.status ?? 'done',
    guard: partial.guard ?? 'pass',
    started_at: partial.started_at ?? null,
    ended_at: partial.ended_at ?? null,
    duration_ms: partial.duration_ms ?? null,
  };
}

/**
 * Resets the internal node counter. Useful for deterministic test output.
 */
export function resetNodeCounter(): void {
  nodeCounter = 0;
}
