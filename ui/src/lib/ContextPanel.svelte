<script lang="ts">
  import type { IterationMeta, AgentOutput } from './types';
  import { selection, parseIterKey, cacheIterationMeta, stream } from './stores.svelte';
  import { fetchIteration, fetchGuardLog, fetchStream } from './api';
  import NodeDetail from './NodeDetail.svelte';

  // Iteration detail state
  let meta: IterationMeta | null = $state(null);
  let output: AgentOutput | null = $state(null);
  let guardLog: string = $state('');
  let loading = $state(false);
  let error: string | null = $state(null);
  let showGuardLog = $state(false);

  // Load iteration data when selection changes
  $effect(() => {
    if (selection.type === 'iteration' && selection.iterKey) {
      const parsed = parseIterKey(selection.iterKey);
      if (parsed) {
        loadIterationData(parsed.runId, parsed.iter);
      }
    } else {
      meta = null;
      output = null;
      guardLog = '';
      showGuardLog = false;
    }
  });

  async function loadIterationData(runId: string, iter: number) {
    loading = true;
    error = null;
    showGuardLog = false;

    // Load stream events independently (don't block on iteration detail)
    loadStreamEvents(runId, iter);

    try {
      const detail = await fetchIteration(runId, iter);
      meta = detail.meta;
      output = detail.output;
      cacheIterationMeta(detail.meta);

      // Load guard log if guard failed
      if (meta.guard === 'fail') {
        guardLog = await fetchGuardLog(runId, iter);
      } else {
        guardLog = '';
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load iteration';
    } finally {
      loading = false;
    }
  }

  async function loadStreamEvents(runId: string, iter: number) {
    try {
      const events = await fetchStream(runId, iter);
      stream.events = events;
      stream.offset = events.length;
    } catch {
      // Stream may not exist, that's ok
      stream.events = [];
      stream.offset = 0;
    }
  }

  function formatDuration(ms: number | null): string {
    if (ms === null) return '-';
    if (ms < 1000) return `${ms}ms`;
    const seconds = Math.floor(ms / 1000);
    if (seconds < 60) return `${seconds}s`;
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes}m ${remainingSeconds}s`;
  }

  function getStatusClass(status: string): string {
    switch (status) {
      case 'done': return 'status-done';
      case 'retry': return 'status-retry';
      case 'decomposed': return 'status-decomposed';
      default: return '';
    }
  }

  function getGuardClass(guard: string): string {
    switch (guard) {
      case 'pass': return 'guard-pass';
      case 'fail': return 'guard-fail';
      case 'skipped': return 'guard-skipped';
      default: return '';
    }
  }
</script>

<div class="context-panel">
  <div class="header">
    <span class="title">
      {#if selection.type === 'node'}
        Node Detail
      {:else if selection.type === 'iteration'}
        Iteration Detail
      {:else}
        Context
      {/if}
    </span>
  </div>

  <div class="content">
    {#if selection.type === 'node' && selection.node}
      <NodeDetail node={selection.node} />
    {:else if selection.type === 'iteration'}
      {#if loading}
        <div class="center-message">Loading...</div>
      {:else if error}
        <div class="center-message error">{error}</div>
      {:else if meta && output}
        <div class="iteration-info">
          <div class="meta-header">
            <span class="iter-label">#{meta.iter}</span>
            <span class="node-id">{meta.node_id}</span>
          </div>

          <div class="result-grid">
            <div class="result-item">
              <span class="label">Status</span>
              <span class="value {getStatusClass(meta.status)}">{meta.status}</span>
            </div>
            <div class="result-item">
              <span class="label">Guard</span>
              <span class="value {getGuardClass(meta.guard)}">{meta.guard}</span>
            </div>
            <div class="result-item">
              <span class="label">Duration</span>
              <span class="value">{formatDuration(meta.duration_ms)}</span>
            </div>
          </div>

          <div class="section">
            <h4 class="section-title">Summary</h4>
            <p class="summary">{output.summary}</p>
          </div>

          {#if guardLog}
            <div class="section">
              <button class="toggle-btn" onclick={() => (showGuardLog = !showGuardLog)}>
                <span class="toggle-icon">{showGuardLog ? '▼' : '▶'}</span>
                Guard Log
              </button>
              {#if showGuardLog}
                <pre class="guard-log">{guardLog}</pre>
              {/if}
            </div>
          {/if}
        </div>
      {:else}
        <div class="center-message">No data</div>
      {/if}
    {:else}
      <div class="center-message">Select a node or iteration</div>
    {/if}
  </div>
</div>

<style>
  .context-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: white;
    border: 1px solid #e2e8f0;
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .header {
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid #e2e8f0;
    flex-shrink: 0;
  }

  .title {
    font-size: 0.75rem;
    font-weight: 600;
    color: #64748b;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 0.75rem;
  }

  .center-message {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #94a3b8;
    font-size: 0.875rem;
  }

  .center-message.error {
    color: #dc2626;
  }

  .iteration-info {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .meta-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .iter-label {
    font-weight: 600;
    font-size: 1rem;
    color: #1e293b;
  }

  .node-id {
    font-family: ui-monospace, monospace;
    font-size: 0.75rem;
    color: #64748b;
    background-color: #f1f5f9;
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
  }

  .result-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.5rem;
  }

  .result-item {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .label {
    font-size: 0.625rem;
    color: #64748b;
    text-transform: uppercase;
  }

  .value {
    font-weight: 500;
    font-size: 0.875rem;
    text-transform: capitalize;
  }

  .status-done { color: #166534; }
  .status-retry { color: #b45309; }
  .status-decomposed { color: #1e40af; }

  .guard-pass { color: #166534; }
  .guard-fail { color: #dc2626; }
  .guard-skipped { color: #64748b; }

  .section-title {
    font-size: 0.625rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #64748b;
    margin-bottom: 0.25rem;
  }

  .summary {
    font-size: 0.875rem;
    line-height: 1.5;
    color: #1e293b;
    white-space: pre-wrap;
  }

  .toggle-btn {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    background: none;
    border: none;
    cursor: pointer;
    font-size: 0.75rem;
    font-weight: 500;
    color: #475569;
    padding: 0;
  }

  .toggle-btn:hover {
    color: #1e293b;
  }

  .toggle-icon {
    font-size: 0.5rem;
  }

  .guard-log {
    margin-top: 0.375rem;
    padding: 0.5rem;
    background-color: #1e293b;
    color: #e2e8f0;
    border-radius: 0.25rem;
    font-family: ui-monospace, monospace;
    font-size: 0.625rem;
    overflow: auto;
    max-height: 10rem;
    white-space: pre-wrap;
  }
</style>
