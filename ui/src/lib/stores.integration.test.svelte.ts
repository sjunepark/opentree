/**
 * Integration tests for cross-store side effects.
 *
 * These tests verify that store mutations cascade correctly
 * to related stores. Since Svelte 5 $state objects can be
 * mutated directly, we can test the imperative side effects
 * without rendering components.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  resetAllStores,
  createMockNode,
  createMockEntry,
  resetNodeCounter,
} from './__test-utils__/store-helpers';
import {
  addTimelineEntry,
  selectNode,
  selectIteration,
  selection,
  iterations,
  stream,
  timer,
  data,
} from './stores.svelte';

beforeEach(() => {
  resetAllStores();
  resetNodeCounter();
  vi.restoreAllMocks();
});

describe('addTimelineEntry cascades', () => {
  it('running entry selects iteration and starts timer', () => {
    vi.spyOn(Date, 'now').mockReturnValue(5000);

    const entry = createMockEntry({
      run_id: 'run-1',
      iter: 3,
      node_id: 'node-a',
      status: 'running',
    });

    addTimelineEntry(entry);

    // Selection should change to iteration
    expect(selection.type).toBe('iteration');
    expect(selection.iterKey).toBe('run-1/3');
    expect(selection.nodeId).toBeNull();
    expect(selection.node).toBeNull();

    // Timer should start
    expect(timer.startTime).toBe(5000);
    expect(timer.elapsed).toBe(0);

    // Stream should be reset to this iteration
    expect(stream.activeRunId).toBe('run-1');
    expect(stream.activeIter).toBe(3);
    expect(stream.events).toHaveLength(0);
  });

  it('completed entry does not change selection', () => {
    // Set up existing selection
    const node = createMockNode({ id: 'selected-node' });
    selection.type = 'node';
    selection.nodeId = 'selected-node';
    selection.node = node;

    const entry = createMockEntry({
      run_id: 'run-1',
      iter: 0,
      status: 'done',
    });

    addTimelineEntry(entry);

    // Selection should remain unchanged
    expect(selection.type).toBe('node');
    expect(selection.nodeId).toBe('selected-node');
    expect(timer.startTime).toBeNull();
  });

  it('decomposed entry does not change selection', () => {
    const entry = createMockEntry({
      run_id: 'run-1',
      iter: 0,
      status: 'decomposed',
    });

    addTimelineEntry(entry);

    expect(selection.type).toBeNull();
    expect(timer.startTime).toBeNull();
  });

  it('retry entry does not change selection', () => {
    const entry = createMockEntry({
      run_id: 'run-1',
      iter: 0,
      status: 'retry',
    });

    addTimelineEntry(entry);

    expect(selection.type).toBeNull();
    expect(timer.startTime).toBeNull();
  });

  it('consecutive running entries update selection to latest', () => {
    vi.spyOn(Date, 'now').mockReturnValueOnce(1000).mockReturnValueOnce(2000);

    addTimelineEntry(createMockEntry({ run_id: 'run-1', iter: 0, status: 'running' }));
    expect(selection.iterKey).toBe('run-1/0');
    expect(timer.startTime).toBe(1000);

    addTimelineEntry(createMockEntry({ run_id: 'run-1', iter: 1, status: 'running' }));
    expect(selection.iterKey).toBe('run-1/1');
    expect(timer.startTime).toBe(2000);
  });
});

describe('selectNode triggers stream loading', () => {
  it('loads latest iteration for node from current run', () => {
    data.runState = {
      run_id: 'run-1',
      next_iter: 5,
      last_status: null,
      last_summary: null,
      last_guard: null,
    };

    iterations.entries = [
      createMockEntry({ run_id: 'run-1', iter: 0, node_id: 'node-a' }),
      createMockEntry({ run_id: 'run-1', iter: 1, node_id: 'target-node' }),
      createMockEntry({ run_id: 'run-1', iter: 2, node_id: 'target-node' }),
      createMockEntry({ run_id: 'run-1', iter: 3, node_id: 'node-b' }),
    ];

    const node = createMockNode({ id: 'target-node' });
    selectNode(node);

    // Should load iter 2 (latest for target-node in run-1)
    expect(stream.activeRunId).toBe('run-1');
    expect(stream.activeIter).toBe(2);
  });

  it('falls back to any run if current run has no entries for node', () => {
    data.runState = {
      run_id: 'run-2',
      next_iter: 1,
      last_status: null,
      last_summary: null,
      last_guard: null,
    };

    iterations.entries = [
      createMockEntry({ run_id: 'run-1', iter: 0, node_id: 'target-node' }),
      createMockEntry({ run_id: 'run-1', iter: 1, node_id: 'target-node' }),
      createMockEntry({ run_id: 'run-2', iter: 0, node_id: 'other-node' }),
    ];

    const node = createMockNode({ id: 'target-node' });
    selectNode(node);

    // Should fall back to run-1 which has entries for target-node
    expect(stream.activeRunId).toBe('run-1');
    expect(stream.activeIter).toBe(1);
  });

  it('falls back to overall latest if node has no entries anywhere', () => {
    data.runState = {
      run_id: 'run-1',
      next_iter: 3,
      last_status: null,
      last_summary: null,
      last_guard: null,
    };

    iterations.entries = [
      createMockEntry({ run_id: 'run-1', iter: 0, node_id: 'other-a' }),
      createMockEntry({ run_id: 'run-1', iter: 1, node_id: 'other-b' }),
    ];

    const node = createMockNode({ id: 'target-node' });
    selectNode(node);

    // Should fall back to latest overall (run_id=run-1, iter=2 from next_iter-1)
    expect(stream.activeRunId).toBe('run-1');
    expect(stream.activeIter).toBe(2);
  });

  it('does not load stream if no entries exist at all', () => {
    iterations.entries = [];
    data.runState = null;

    stream.activeRunId = 'existing';
    stream.activeIter = 99;

    const node = createMockNode({ id: 'target-node' });
    selectNode(node);

    // Stream should remain unchanged when there's nothing to load
    expect(stream.activeRunId).toBe('existing');
    expect(stream.activeIter).toBe(99);
  });

  it('does not reload if already showing the right iteration', () => {
    data.runState = {
      run_id: 'run-1',
      next_iter: 2,
      last_status: null,
      last_summary: null,
      last_guard: null,
    };

    iterations.entries = [
      createMockEntry({ run_id: 'run-1', iter: 0, node_id: 'target-node' }),
    ];

    // Already showing the right iteration
    stream.activeRunId = 'run-1';
    stream.activeIter = 0;
    stream.events = [{ type: 'existing-event' }];
    stream.offset = 1;

    const node = createMockNode({ id: 'target-node' });
    selectNode(node);

    // Stream should NOT be reset since we're already showing the right iteration
    expect(stream.events).toHaveLength(1);
    expect(stream.offset).toBe(1);
  });

  it('does not load stream when timer is running (live iteration)', () => {
    timer.startTime = Date.now();

    data.runState = {
      run_id: 'run-1',
      next_iter: 5,
      last_status: null,
      last_summary: null,
      last_guard: null,
    };

    iterations.entries = [
      createMockEntry({ run_id: 'run-1', iter: 0, node_id: 'target-node' }),
    ];

    stream.activeRunId = 'live-run';
    stream.activeIter = 99;
    stream.events = [{ type: 'live-event' }];

    const node = createMockNode({ id: 'target-node' });
    selectNode(node);

    // Stream should remain unchanged when timer is running
    expect(stream.activeRunId).toBe('live-run');
    expect(stream.activeIter).toBe(99);
    expect(stream.events).toHaveLength(1);
  });
});

describe('selectIteration resets stream', () => {
  it('clears existing events and sets new activeRunId/activeIter', () => {
    stream.activeRunId = 'old-run';
    stream.activeIter = 5;
    stream.events = [{ type: 'old1' }, { type: 'old2' }];
    stream.offset = 10;
    stream.autoScroll = false;

    selectIteration('new-run', 3);

    expect(stream.activeRunId).toBe('new-run');
    expect(stream.activeIter).toBe(3);
    expect(stream.events).toHaveLength(0);
    expect(stream.offset).toBe(0);
  });

  it('also updates selection state', () => {
    // Set up node selection
    const node = createMockNode({ id: 'some-node' });
    selection.type = 'node';
    selection.nodeId = 'some-node';
    selection.node = node;
    selection.iterKey = null;

    selectIteration('run-1', 7);

    expect(selection.type).toBe('iteration');
    expect(selection.iterKey).toBe('run-1/7');
    expect(selection.nodeId).toBeNull();
    expect(selection.node).toBeNull();
  });
});

describe('cross-store consistency', () => {
  it('maintains consistency through a full iteration lifecycle', () => {
    // 1. New running iteration starts
    vi.spyOn(Date, 'now').mockReturnValue(1000);
    addTimelineEntry(createMockEntry({
      run_id: 'run-1',
      iter: 0,
      node_id: 'root',
      status: 'running',
    }));

    expect(selection.type).toBe('iteration');
    expect(selection.iterKey).toBe('run-1/0');
    expect(timer.startTime).toBe(1000);
    expect(stream.activeRunId).toBe('run-1');
    expect(stream.activeIter).toBe(0);

    // 2. User manually selects a node while iteration runs
    const node = createMockNode({ id: 'some-node' });
    selectNode(node);

    // Since timer is running, stream should NOT change
    expect(selection.type).toBe('node');
    expect(selection.nodeId).toBe('some-node');
    expect(stream.activeRunId).toBe('run-1');
    expect(stream.activeIter).toBe(0);

    // 3. User switches back to the running iteration
    selectIteration('run-1', 0);

    expect(selection.type).toBe('iteration');
    expect(selection.iterKey).toBe('run-1/0');
    expect(stream.events).toHaveLength(0); // Reset clears events
  });

  it('handles run transitions correctly', () => {
    // First run
    addTimelineEntry(createMockEntry({
      run_id: 'run-1',
      iter: 0,
      status: 'done',
    }));

    // New run starts
    vi.spyOn(Date, 'now').mockReturnValue(2000);
    addTimelineEntry(createMockEntry({
      run_id: 'run-2',
      iter: 0,
      status: 'running',
    }));

    expect(iterations.entries).toHaveLength(2);
    expect(selection.iterKey).toBe('run-2/0');
    expect(stream.activeRunId).toBe('run-2');
    expect(timer.startTime).toBe(2000);
  });
});
