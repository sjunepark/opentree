import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  resetAllStores,
  createMockNode,
  createMockEntry,
  createMockIterationMeta,
  resetNodeCounter,
} from './__test-utils__/store-helpers';
import {
  makeIterKey,
  parseIterKey,
  selectNode,
  selectIteration,
  clearSelection,
  addTimelineEntry,
  updateTimelineEntry,
  cacheIterationMeta,
  getCachedMeta,
  appendStreamEvents,
  resetStream,
  setRightPanelTab,
  selection,
  iterations,
  stream,
  timer,
  rightPanel,
  data,
} from './stores.svelte';

beforeEach(() => {
  resetAllStores();
  resetNodeCounter();
  vi.restoreAllMocks();
});

describe('makeIterKey', () => {
  it('creates "runId/iter" format', () => {
    expect(makeIterKey('run-abc', 5)).toBe('run-abc/5');
  });

  it('handles zero iteration', () => {
    expect(makeIterKey('run-1', 0)).toBe('run-1/0');
  });

  it('handles complex run IDs', () => {
    expect(makeIterKey('2024-01-15T10:30:00Z', 42)).toBe('2024-01-15T10:30:00Z/42');
  });
});

describe('parseIterKey', () => {
  it('parses valid key back to runId and iter', () => {
    const result = parseIterKey('run-abc/5');
    expect(result).toEqual({ runId: 'run-abc', iter: 5 });
  });

  it('handles zero iteration', () => {
    const result = parseIterKey('run-1/0');
    expect(result).toEqual({ runId: 'run-1', iter: 0 });
  });

  it('returns null for invalid format - missing slash', () => {
    expect(parseIterKey('run-abc5')).toBeNull();
  });

  it('returns null for invalid format - too many slashes', () => {
    expect(parseIterKey('run/abc/5')).toBeNull();
  });

  it('returns null for non-numeric iter', () => {
    expect(parseIterKey('run-abc/five')).toBeNull();
  });

  it('returns null for empty string', () => {
    expect(parseIterKey('')).toBeNull();
  });

  it('handles complex run IDs with colons and dashes', () => {
    const result = parseIterKey('2024-01-15T10:30:00Z/42');
    expect(result).toEqual({ runId: '2024-01-15T10:30:00Z', iter: 42 });
  });
});

describe('selectNode', () => {
  it('sets type/nodeId/node when selecting a node', () => {
    const node = createMockNode({ id: 'test-node', title: 'Test' });

    selectNode(node);

    expect(selection.type).toBe('node');
    expect(selection.nodeId).toBe('test-node');
    expect(selection.node).toStrictEqual(node);
    expect(selection.iterKey).toBeNull();
  });

  it('clears iterKey when selecting a node', () => {
    selection.iterKey = 'run-1/5';

    const node = createMockNode({ id: 'test-node' });
    selectNode(node);

    expect(selection.iterKey).toBeNull();
  });

  it('clears all selection state when selecting null', () => {
    const node = createMockNode({ id: 'test-node' });
    selectNode(node);

    selectNode(null);

    expect(selection.type).toBeNull();
    expect(selection.nodeId).toBeNull();
    expect(selection.node).toBeNull();
    expect(selection.iterKey).toBeNull();
  });

  it('preserves existing stream when timer is running', () => {
    timer.startTime = Date.now();
    stream.activeRunId = 'run-1';
    stream.activeIter = 5;
    stream.events = [{ type: 'test' }];

    const node = createMockNode({ id: 'test-node' });
    selectNode(node);

    // Stream should not be reset when timer is running
    expect(stream.activeRunId).toBe('run-1');
    expect(stream.activeIter).toBe(5);
    expect(stream.events).toHaveLength(1);
  });

  it('loads latest iteration for node when timer is not running', () => {
    // Add some timeline entries
    iterations.entries = [
      createMockEntry({ run_id: 'run-1', iter: 0, node_id: 'node-a' }),
      createMockEntry({ run_id: 'run-1', iter: 1, node_id: 'test-node' }),
      createMockEntry({ run_id: 'run-1', iter: 2, node_id: 'node-b' }),
    ];
    data.runState = { run_id: 'run-1', next_iter: 3, last_status: null, last_summary: null, last_guard: null };

    const node = createMockNode({ id: 'test-node' });
    selectNode(node);

    // Should load iter 1 which is the latest for test-node
    expect(stream.activeRunId).toBe('run-1');
    expect(stream.activeIter).toBe(1);
  });

  it('falls back to overall latest when node has no entries', () => {
    iterations.entries = [
      createMockEntry({ run_id: 'run-1', iter: 0, node_id: 'other-node' }),
      createMockEntry({ run_id: 'run-1', iter: 1, node_id: 'other-node' }),
    ];
    data.runState = { run_id: 'run-1', next_iter: 2, last_status: null, last_summary: null, last_guard: null };

    const node = createMockNode({ id: 'nonexistent-node' });
    selectNode(node);

    // Should fall back to latest iteration overall
    expect(stream.activeRunId).toBe('run-1');
    expect(stream.activeIter).toBe(1);
  });
});

describe('selectIteration', () => {
  it('sets type to iteration and clears node selection', () => {
    selection.nodeId = 'some-node';
    selection.node = createMockNode();

    selectIteration('run-1', 5);

    expect(selection.type).toBe('iteration');
    expect(selection.nodeId).toBeNull();
    expect(selection.node).toBeNull();
    expect(selection.iterKey).toBe('run-1/5');
  });

  it('resets stream state', () => {
    stream.events = [{ type: 'old-event' }];
    stream.offset = 10;
    stream.activeRunId = 'old-run';
    stream.activeIter = 99;

    selectIteration('run-2', 3);

    expect(stream.activeRunId).toBe('run-2');
    expect(stream.activeIter).toBe(3);
    expect(stream.events).toHaveLength(0);
    expect(stream.offset).toBe(0);
  });
});

describe('clearSelection', () => {
  it('nullifies all selection state', () => {
    const node = createMockNode();
    selection.type = 'node';
    selection.nodeId = 'test';
    selection.node = node;
    selection.iterKey = 'run-1/5';

    clearSelection();

    expect(selection.type).toBeNull();
    expect(selection.nodeId).toBeNull();
    expect(selection.node).toBeNull();
    expect(selection.iterKey).toBeNull();
  });
});

describe('addTimelineEntry', () => {
  it('appends entry to iterations.entries', () => {
    const entry = createMockEntry({ run_id: 'run-1', iter: 0 });

    addTimelineEntry(entry);

    expect(iterations.entries).toHaveLength(1);
    expect(iterations.entries[0]).toStrictEqual(entry);
  });

  it('appends multiple entries in order', () => {
    addTimelineEntry(createMockEntry({ iter: 0 }));
    addTimelineEntry(createMockEntry({ iter: 1 }));
    addTimelineEntry(createMockEntry({ iter: 2 }));

    expect(iterations.entries).toHaveLength(3);
    expect(iterations.entries.map(e => e.iter)).toEqual([0, 1, 2]);
  });

  it('auto-selects running entry', () => {
    const entry = createMockEntry({ run_id: 'run-1', iter: 5, status: 'running' });

    addTimelineEntry(entry);

    expect(selection.type).toBe('iteration');
    expect(selection.iterKey).toBe('run-1/5');
  });

  it('starts timer for running entry', () => {
    vi.spyOn(Date, 'now').mockReturnValue(1000);

    const entry = createMockEntry({ status: 'running' });
    addTimelineEntry(entry);

    expect(timer.startTime).toBe(1000);
    expect(timer.elapsed).toBe(0);
  });

  it('does not auto-select completed entries', () => {
    const entry = createMockEntry({ status: 'done' });

    addTimelineEntry(entry);

    expect(selection.type).toBeNull();
    expect(timer.startTime).toBeNull();
  });

  it('does not auto-select retry entries', () => {
    const entry = createMockEntry({ status: 'retry' });

    addTimelineEntry(entry);

    expect(selection.type).toBeNull();
  });
});

describe('updateTimelineEntry', () => {
  it('updates existing entry status and guard', () => {
    addTimelineEntry(createMockEntry({ run_id: 'run-1', iter: 0, status: 'running', guard: null }));

    updateTimelineEntry('run-1', 0, 'done', 'pass');

    expect(iterations.entries[0].status).toBe('done');
    expect(iterations.entries[0].guard).toBe('pass');
  });

  it('finds correct entry among multiple', () => {
    addTimelineEntry(createMockEntry({ run_id: 'run-1', iter: 0 }));
    addTimelineEntry(createMockEntry({ run_id: 'run-1', iter: 1 }));
    addTimelineEntry(createMockEntry({ run_id: 'run-1', iter: 2 }));

    updateTimelineEntry('run-1', 1, 'retry', 'fail');

    expect(iterations.entries[0].status).toBe('done'); // unchanged
    expect(iterations.entries[1].status).toBe('retry'); // updated
    expect(iterations.entries[2].status).toBe('done'); // unchanged
  });

  it('no-op when entry not found', () => {
    addTimelineEntry(createMockEntry({ run_id: 'run-1', iter: 0 }));

    // Should not throw
    updateTimelineEntry('run-1', 99, 'done', 'pass');
    updateTimelineEntry('nonexistent-run', 0, 'done', 'pass');

    expect(iterations.entries[0].status).toBe('done');
  });
});

describe('cacheIterationMeta / getCachedMeta', () => {
  it('round-trips metadata correctly', () => {
    const meta = createMockIterationMeta({
      run_id: 'run-1',
      iter: 5,
      status: 'done',
      guard: 'pass',
      duration_ms: 1234,
    });

    cacheIterationMeta(meta);
    const retrieved = getCachedMeta('run-1', 5);

    expect(retrieved).toBe(meta);
  });

  it('returns undefined for uncached entry', () => {
    expect(getCachedMeta('nonexistent', 0)).toBeUndefined();
  });

  it('overwrites existing cache entry', () => {
    const meta1 = createMockIterationMeta({ run_id: 'run-1', iter: 0, duration_ms: 100 });
    const meta2 = createMockIterationMeta({ run_id: 'run-1', iter: 0, duration_ms: 200 });

    cacheIterationMeta(meta1);
    cacheIterationMeta(meta2);

    expect(getCachedMeta('run-1', 0)?.duration_ms).toBe(200);
  });

  it('stores multiple entries independently', () => {
    const meta1 = createMockIterationMeta({ run_id: 'run-1', iter: 0 });
    const meta2 = createMockIterationMeta({ run_id: 'run-1', iter: 1 });
    const meta3 = createMockIterationMeta({ run_id: 'run-2', iter: 0 });

    cacheIterationMeta(meta1);
    cacheIterationMeta(meta2);
    cacheIterationMeta(meta3);

    expect(getCachedMeta('run-1', 0)).toBe(meta1);
    expect(getCachedMeta('run-1', 1)).toBe(meta2);
    expect(getCachedMeta('run-2', 0)).toBe(meta3);
  });
});

describe('appendStreamEvents', () => {
  it('appends events to stream.events', () => {
    const events = [{ type: 'event1' }, { type: 'event2' }];

    appendStreamEvents(events);

    expect(stream.events).toHaveLength(2);
    expect(stream.events[0]).toEqual({ type: 'event1' });
    expect(stream.events[1]).toEqual({ type: 'event2' });
  });

  it('increments offset by event count', () => {
    expect(stream.offset).toBe(0);

    appendStreamEvents([{ type: 'e1' }, { type: 'e2' }]);
    expect(stream.offset).toBe(2);

    appendStreamEvents([{ type: 'e3' }]);
    expect(stream.offset).toBe(3);
  });

  it('handles empty array', () => {
    appendStreamEvents([]);

    expect(stream.events).toHaveLength(0);
    expect(stream.offset).toBe(0);
  });

  it('appends to existing events', () => {
    stream.events = [{ type: 'existing' }];
    stream.offset = 1;

    appendStreamEvents([{ type: 'new1' }, { type: 'new2' }]);

    expect(stream.events).toHaveLength(3);
    expect(stream.offset).toBe(3);
  });
});

describe('resetStream', () => {
  it('clears events and resets offset', () => {
    stream.events = [{ type: 'e1' }, { type: 'e2' }];
    stream.offset = 5;

    resetStream('run-new', 10);

    expect(stream.events).toHaveLength(0);
    expect(stream.offset).toBe(0);
  });

  it('sets activeRunId and activeIter', () => {
    resetStream('run-123', 7);

    expect(stream.activeRunId).toBe('run-123');
    expect(stream.activeIter).toBe(7);
  });

  it('resets autoScroll to true', () => {
    stream.autoScroll = false;

    resetStream('run-1', 0);

    expect(stream.autoScroll).toBe(true);
  });
});

describe('setRightPanelTab', () => {
  it('updates activeTab to config', () => {
    expect(rightPanel.activeTab).toBe('details');

    setRightPanelTab('config');

    expect(rightPanel.activeTab).toBe('config');
  });

  it('updates activeTab to docs', () => {
    setRightPanelTab('docs');

    expect(rightPanel.activeTab).toBe('docs');
  });

  it('updates activeTab back to details', () => {
    setRightPanelTab('config');
    setRightPanelTab('details');

    expect(rightPanel.activeTab).toBe('details');
  });
});
