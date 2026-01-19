<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import StatusBar from './lib/StatusBar.svelte';
  import LeftPanel from './lib/LeftPanel.svelte';
  import RightPanel from './lib/RightPanel.svelte';
  import {
    connection,
    data,
    iterations,
    stream,
    timer,
    addTimelineEntry,
    updateTimelineEntry,
    appendStreamEvents,
    resetStream,
    selectNode,
  } from './lib/stores.svelte';
  import { findAutoSelectNode } from './lib/tree-utils';
  import type { TimelineEntry } from './lib/types';
  import {
    fetchTree,
    fetchRunState,
    fetchIterations,
    fetchConfig,
    fetchAssumptions,
    fetchQuestions,
    fetchStream,
    fetchIteration,
  } from './lib/api';
  import { connect, disconnect, subscribe } from './lib/sse';

  // Load initial data
  async function loadData() {
    connection.loading = true;
    connection.error = null;

    try {
      const [treeData, runStateData, iterationsData, configData, assumptionsData, questionsData] =
        await Promise.all([
          fetchTree().catch(() => null),
          fetchRunState().catch(() => null),
          fetchIterations().catch(() => ({ runs: [] })),
          fetchConfig().catch(() => null),
          fetchAssumptions().catch(() => ''),
          fetchQuestions().catch(() => ''),
        ]);

      if (treeData) data.tree = treeData;
      if (runStateData) data.runState = runStateData;
      if (configData) data.config = configData;
      data.assumptions = assumptionsData;
      data.questions = questionsData;

      // Convert runs to timeline entries
      await buildTimeline(iterationsData.runs);

      // Auto-select the current working node or last passing node
      if (data.tree) {
        const nodeToSelect = findAutoSelectNode(data.tree);
        selectNode(nodeToSelect);
      }
    } catch (e) {
      connection.error = e instanceof Error ? e.message : 'Failed to load data';
    } finally {
      connection.loading = false;
    }
  }

  // Build timeline from run entries
  async function buildTimeline(runs: Array<{ run_id: string; iterations: number[] }>) {
    const entries: TimelineEntry[] = [];

    for (const run of runs) {
      for (const iter of run.iterations) {
        try {
          const detail = await fetchIteration(run.run_id, iter);
          entries.push({
            run_id: run.run_id,
            iter,
            node_id: detail.meta.node_id,
            status: detail.meta.status,
            guard: detail.meta.guard,
          });
        } catch {
          // If we can't fetch the iteration, create a placeholder
          entries.push({
            run_id: run.run_id,
            iter,
            node_id: 'unknown',
            status: 'done',
            guard: null,
          });
        }
      }
    }

    iterations.entries = entries;
  }

  // Handle SSE events
  function handleChange(event: {
    type: string;
    run_id?: string;
    iter?: number;
    node_id?: string;
  }) {
    switch (event.type) {
      case 'tree_changed':
        fetchTree()
          .then((d) => (data.tree = d))
          .catch(console.error);
        break;

      case 'run_state_changed':
        fetchRunState()
          .then((d) => (data.runState = d))
          .catch(console.error);
        break;

      case 'iteration_added':
        if (event.run_id && event.iter !== undefined) {
          // Add a running entry to timeline
          const entry: TimelineEntry = {
            run_id: event.run_id,
            iter: event.iter,
            node_id: event.node_id || 'unknown',
            status: 'running',
            guard: null,
          };
          addTimelineEntry(entry);

          // Reset timer
          timer.startTime = Date.now();
          timer.elapsed = 0;

          // Start streaming for this iteration
          resetStream(event.run_id, event.iter);
        }
        break;

      case 'stream_updated':
        if (
          event.run_id === stream.activeRunId &&
          event.iter === stream.activeIter
        ) {
          // Fetch new events with offset
          fetchStream(event.run_id, event.iter, stream.offset)
            .then((newEvents) => {
              if (newEvents.length > 0) {
                appendStreamEvents(newEvents);
              }
            })
            .catch(console.error);
        }
        break;

      case 'iteration_completed':
        if (event.run_id && event.iter !== undefined) {
          // Fetch the completed iteration details
          fetchIteration(event.run_id, event.iter)
            .then((detail) => {
              updateTimelineEntry(
                event.run_id!,
                event.iter!,
                detail.meta.status,
                detail.meta.guard
              );
              // Clear timer
              timer.startTime = null;
            })
            .catch(console.error);
        }
        break;

      case 'config_changed':
        fetchConfig()
          .then((d) => (data.config = d))
          .catch(console.error);
        break;

      case 'assumptions_changed':
        fetchAssumptions()
          .then((d) => (data.assumptions = d))
          .catch(console.error);
        break;

      case 'questions_changed':
        fetchQuestions()
          .then((d) => (data.questions = d))
          .catch(console.error);
        break;
    }
  }

  let unsubscribe: (() => void) | null = null;

  onMount(() => {
    loadData();
    connect();
    unsubscribe = subscribe(handleChange);
  });

  onDestroy(() => {
    disconnect();
    if (unsubscribe) unsubscribe();
  });
</script>

<div class="app">
  <StatusBar />

  <main class="main">
    {#if connection.loading}
      <div class="loading-overlay">
        <span>Loading...</span>
      </div>
    {:else if connection.error}
      <div class="error-overlay">
        <p>Error: {connection.error}</p>
        <button onclick={loadData}>Retry</button>
      </div>
    {:else}
      <div class="left-panel">
        <LeftPanel tree={data.tree} />
      </div>
      <div class="right-panel">
        <RightPanel
          config={data.config}
          assumptions={data.assumptions}
          questions={data.questions}
        />
      </div>
    {/if}
  </main>
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background-color: #f8fafc;
  }

  .main {
    flex: 1;
    display: grid;
    grid-template-columns: 1fr 1fr;
    overflow: hidden;
    padding: 0.75rem;
    gap: 0.75rem;
  }

  .loading-overlay,
  .error-overlay {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 1rem;
    color: #64748b;
    grid-column: 1 / -1;
  }

  .error-overlay {
    color: #dc2626;
  }

  .error-overlay button {
    padding: 0.5rem 1rem;
    background-color: #3b82f6;
    color: white;
    border: none;
    border-radius: 0.375rem;
    cursor: pointer;
  }

  .error-overlay button:hover {
    background-color: #2563eb;
  }

  .left-panel {
    min-height: 0;
    overflow: hidden;
  }

  .right-panel {
    min-height: 0;
    overflow: hidden;
  }
</style>
