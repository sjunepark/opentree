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

// Secondary drawer state
export const drawer = $state({
  open: false,
  activeTab: 'tree' as 'tree' | 'config' | 'docs',
});

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

// Toggle drawer
export function toggleDrawer(): void {
  drawer.open = !drawer.open;
}

// Set drawer tab
export function setDrawerTab(tab: 'tree' | 'config' | 'docs'): void {
  drawer.activeTab = tab;
  if (!drawer.open) {
    drawer.open = true;
  }
}
