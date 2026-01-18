<script lang="ts">
  import { SvelteSet } from 'svelte/reactivity';
  import type { RunEntry } from './stores';
  import { selectedIteration } from './stores';

  interface Props {
    runs: RunEntry[];
  }

  let { runs }: Props = $props();

  // Track which runs are expanded (SvelteSet for Svelte 5 reactivity)
  let expandedRuns = new SvelteSet<string>();

  function toggleRun(runId: string) {
    if (expandedRuns.has(runId)) {
      expandedRuns.delete(runId);
    } else {
      expandedRuns.add(runId);
    }
  }

  function selectIteration(runId: string, iter: number) {
    selectedIteration.set({ run_id: runId, iter });
  }

  // Auto-expand the first (most recent) run
  $effect(() => {
    if (runs.length > 0 && expandedRuns.size === 0) {
      expandedRuns.add(runs[runs.length - 1].run_id);
    }
  });
</script>

<div class="iteration-list">
  {#if runs.length === 0}
    <div class="empty-state">No iterations yet</div>
  {:else}
    {#each [...runs].reverse() as run (run.run_id)}
      <div class="run-group">
        <button class="run-header" onclick={() => toggleRun(run.run_id)}>
          <span class="toggle">{expandedRuns.has(run.run_id) ? '▼' : '▶'}</span>
          <span class="run-id">{run.run_id}</span>
          <span class="iter-count">{run.iterations.length} iteration{run.iterations.length !== 1 ? 's' : ''}</span>
        </button>

        {#if expandedRuns.has(run.run_id)}
          <div class="iterations">
            {#each [...run.iterations].reverse() as iter (iter)}
              {@const isSelected = $selectedIteration?.run_id === run.run_id && $selectedIteration?.iter === iter}
              <button
                class="iteration-row"
                class:selected={isSelected}
                onclick={() => selectIteration(run.run_id, iter)}
              >
                <span class="iter-number">#{iter}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  {/if}
</div>

<style>
  .iteration-list {
    font-size: 0.875rem;
    overflow: auto;
    height: 100%;
  }

  .empty-state {
    padding: 1rem;
    color: #64748b;
    text-align: center;
  }

  .run-group {
    border-bottom: 1px solid #e2e8f0;
  }

  .run-group:last-child {
    border-bottom: none;
  }

  .run-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.75rem;
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    transition: background-color 0.1s;
  }

  .run-header:hover {
    background-color: #f8fafc;
  }

  .toggle {
    font-size: 0.625rem;
    color: #64748b;
    width: 0.75rem;
  }

  .run-id {
    flex: 1;
    font-weight: 500;
    color: #1e293b;
    font-family: ui-monospace, monospace;
    font-size: 0.75rem;
  }

  .iter-count {
    font-size: 0.75rem;
    color: #64748b;
  }

  .iterations {
    padding-bottom: 0.5rem;
  }

  .iteration-row {
    display: block;
    width: 100%;
    padding: 0.375rem 1rem 0.375rem 2rem;
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    transition: background-color 0.1s;
  }

  .iteration-row:hover {
    background-color: #f1f5f9;
  }

  .iteration-row.selected {
    background-color: #e0f2fe;
  }

  .iter-number {
    color: #475569;
    font-family: ui-monospace, monospace;
    font-size: 0.75rem;
  }
</style>
