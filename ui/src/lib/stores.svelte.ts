import type {
  Node,
  RunState,
  RunnerConfig,
  StreamEvent,
  TimelineEntry,
  IterationMeta,
  SelectionType,
} from './types';

// Connection & loading state
export const connection = $state({
  sseConnected: false,
  loading: true,
  error: null as string | null,
  staticMode: false,
  lastEventTime: null as number | null,
});

// Core data from the runner
export const data = $state({
  tree: null as Node | null,
  runState: null as RunState | null,
  config: null as RunnerConfig | null,
  assumptions: '',
  questions: '',
});

// Unified iteration timeline
export const iterations = $state({
  entries: [] as TimelineEntry[],
  metaCache: new Map<string, IterationMeta>(),
});

// Live event stream
export const stream = $state({
  events: [] as StreamEvent[],
  offset: 0,
  activeRunId: null as string | null,
  activeIter: null as number | null,
  autoScroll: true,
});

// Unified selection state
export const selection = $state({
  type: null as SelectionType,
  nodeId: null as string | null,
  node: null as Node | null,
  iterKey: null as string | null, // "run_id/iter"
});

// Timer state for the status bar
export const timer = $state({
  startTime: null as number | null,
  elapsed: 0,
});

// Right panel state
export const rightPanel = $state({
  activeTab: 'details' as 'details' | 'config' | 'docs',
});

function latestIterationForNode(
  nodeId: string,
  preferredRunId: string | null
): { runId: string; iter: number } | null {
  let bestRunId: string | null = null;
  let bestIter = -1;

  for (const entry of iterations.entries) {
    if (entry.node_id !== nodeId) continue;
    if (preferredRunId && entry.run_id !== preferredRunId) continue;

    if (
      bestRunId === null ||
      entry.run_id > bestRunId ||
      (entry.run_id === bestRunId && entry.iter > bestIter)
    ) {
      bestRunId = entry.run_id;
      bestIter = entry.iter;
    }
  }

  if (bestRunId === null) return null;
  return { runId: bestRunId, iter: bestIter };
}

function latestIterationOverall(): { runId: string; iter: number } | null {
  const runId = data.runState?.run_id ?? null;
  const nextIter = data.runState?.next_iter ?? 0;
  if (runId && nextIter > 0) {
    return { runId, iter: nextIter - 1 };
  }

  if (iterations.entries.length === 0) return null;

  let bestRunId = iterations.entries[0].run_id;
  let bestIter = iterations.entries[0].iter;

  for (const entry of iterations.entries.slice(1)) {
    if (
      entry.run_id > bestRunId ||
      (entry.run_id === bestRunId && entry.iter > bestIter)
    ) {
      bestRunId = entry.run_id;
      bestIter = entry.iter;
    }
  }

  return { runId: bestRunId, iter: bestIter };
}

// Helper to create iteration key
export function makeIterKey(runId: string, iter: number): string {
  return `${runId}/${iter}`;
}

// Helper to parse iteration key
export function parseIterKey(key: string): { runId: string; iter: number } | null {
  const parts = key.split('/');
  if (parts.length !== 2) return null;
  const iter = parseInt(parts[1], 10);
  if (isNaN(iter)) return null;
  return { runId: parts[0], iter };
}

// Select a node
export function selectNode(node: Node | null): void {
  selection.type = node ? 'node' : null;
  selection.nodeId = node?.id ?? null;
  selection.node = node;
  selection.iterKey = null;

  // When not actively streaming a live iteration, show logs for the last iteration
  // that worked on the selected node (or fall back to the most recent iteration).
  if (!node) return;
  if (timer.startTime !== null) return;

  const preferredRunId = data.runState?.run_id ?? null;
  const latestForNode =
    latestIterationForNode(node.id, preferredRunId) ?? latestIterationForNode(node.id, null);
  const latest = latestForNode ?? latestIterationOverall();
  if (!latest) return;

  if (stream.activeRunId === latest.runId && stream.activeIter === latest.iter) return;
  resetStream(latest.runId, latest.iter);
}

// Select an iteration
export function selectIteration(runId: string, iter: number): void {
  selection.type = 'iteration';
  selection.nodeId = null;
  selection.node = null;
  selection.iterKey = makeIterKey(runId, iter);

  // Switch stream to this iteration
  stream.activeRunId = runId;
  stream.activeIter = iter;
  stream.events = [];
  stream.offset = 0;
}

// Clear selection
export function clearSelection(): void {
  selection.type = null;
  selection.nodeId = null;
  selection.node = null;
  selection.iterKey = null;
}

// Add timeline entry (auto-selects if it's running)
export function addTimelineEntry(entry: TimelineEntry): void {
  iterations.entries.push(entry);

  // Auto-follow: select the new running iteration
  if (entry.status === 'running') {
    selectIteration(entry.run_id, entry.iter);
    // Reset timer for new iteration
    timer.startTime = Date.now();
    timer.elapsed = 0;
  }
}

// Update timeline entry status
export function updateTimelineEntry(
  runId: string,
  iter: number,
  status: TimelineEntry['status'],
  guard: TimelineEntry['guard']
): void {
  const entry = iterations.entries.find(
    (e) => e.run_id === runId && e.iter === iter
  );
  if (entry) {
    entry.status = status;
    entry.guard = guard;
  }
}

// Cache iteration metadata
export function cacheIterationMeta(meta: IterationMeta): void {
  const key = makeIterKey(meta.run_id, meta.iter);
  iterations.metaCache.set(key, meta);
}

// Get cached iteration metadata
export function getCachedMeta(runId: string, iter: number): IterationMeta | undefined {
  return iterations.metaCache.get(makeIterKey(runId, iter));
}

// Append stream events
export function appendStreamEvents(events: StreamEvent[]): void {
  stream.events.push(...events);
  stream.offset += events.length;
}

// Reset stream for new iteration
export function resetStream(runId: string, iter: number): void {
  stream.activeRunId = runId;
  stream.activeIter = iter;
  stream.events = [];
  stream.offset = 0;
  stream.autoScroll = true;
}

// Set right panel tab
export function setRightPanelTab(tab: 'details' | 'config' | 'docs'): void {
  rightPanel.activeTab = tab;
}
