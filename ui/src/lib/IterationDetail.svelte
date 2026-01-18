<script lang="ts">
  import type { IterationMeta, AgentOutput } from './stores';
  import { fetchIteration, fetchGuardLog } from './api';

  interface Props {
    runId: string;
    iter: number;
  }

  let { runId, iter }: Props = $props();

  let meta: IterationMeta | null = $state(null);
  let output: AgentOutput | null = $state(null);
  let guardLog: string = $state('');
  let loading = $state(true);
  let error: string | null = $state(null);
  let showGuardLog = $state(false);

  // Load data when props change
  $effect(() => {
    loadData(runId, iter);
  });

  async function loadData(runId: string, iter: number) {
    loading = true;
    error = null;
    showGuardLog = false;

    try {
      const detail = await fetchIteration(runId, iter);
      meta = detail.meta;
      output = detail.output;

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

  function getStatusColor(status: string): string {
    switch (status) {
      case 'done':
        return 'status-done';
      case 'retry':
        return 'status-retry';
      case 'decomposed':
        return 'status-decomposed';
      default:
        return '';
    }
  }

  function getGuardColor(guard: string): string {
    switch (guard) {
      case 'pass':
        return 'guard-pass';
      case 'fail':
        return 'guard-fail';
      case 'skipped':
        return 'guard-skipped';
      default:
        return '';
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
</script>

<div class="iteration-detail">
  {#if loading}
    <div class="loading">Loading...</div>
  {:else if error}
    <div class="error">{error}</div>
  {:else if meta && output}
    <div class="header">
      <span class="iter-label">Iteration #{iter}</span>
      <span class="run-label">{runId}</span>
    </div>

    <div class="section">
      <h3 class="section-title">Node</h3>
      <code class="node-id">{meta.node_id}</code>
    </div>

    <div class="section">
      <h3 class="section-title">Result</h3>
      <div class="result-grid">
        <div class="result-item">
          <span class="result-label">Status</span>
          <span class="result-value {getStatusColor(meta.status)}">{meta.status}</span>
        </div>
        <div class="result-item">
          <span class="result-label">Guard</span>
          <span class="result-value {getGuardColor(meta.guard)}">{meta.guard}</span>
        </div>
        <div class="result-item">
          <span class="result-label">Duration</span>
          <span class="result-value">{formatDuration(meta.duration_ms)}</span>
        </div>
      </div>
    </div>

    <div class="section">
      <h3 class="section-title">Summary</h3>
      <p class="summary">{output.summary}</p>
    </div>

    {#if guardLog}
      <div class="section">
        <button class="guard-log-toggle" onclick={() => (showGuardLog = !showGuardLog)}>
          <span class="toggle">{showGuardLog ? '▼' : '▶'}</span>
          Guard Log
        </button>
        {#if showGuardLog}
          <pre class="guard-log">{guardLog}</pre>
        {/if}
      </div>
    {/if}

    {#if meta.started_at || meta.ended_at}
      <div class="section timestamps">
        {#if meta.started_at}
          <div class="timestamp">
            <span class="timestamp-label">Started</span>
            <span class="timestamp-value">{meta.started_at}</span>
          </div>
        {/if}
        {#if meta.ended_at}
          <div class="timestamp">
            <span class="timestamp-label">Ended</span>
            <span class="timestamp-value">{meta.ended_at}</span>
          </div>
        {/if}
      </div>
    {/if}
  {/if}
</div>

<style>
  .iteration-detail {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .loading,
  .error {
    padding: 1rem;
    text-align: center;
    color: #64748b;
  }

  .error {
    color: #dc2626;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .iter-label {
    font-weight: 600;
    font-size: 1rem;
    color: #1e293b;
  }

  .run-label {
    font-family: ui-monospace, monospace;
    font-size: 0.75rem;
    color: #64748b;
  }

  .section {
    /* Section styling */
  }

  .section-title {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #64748b;
    margin-bottom: 0.375rem;
  }

  .node-id {
    font-family: ui-monospace, monospace;
    font-size: 0.75rem;
    background-color: #f1f5f9;
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    color: #475569;
  }

  .result-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.75rem;
  }

  .result-item {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .result-label {
    font-size: 0.75rem;
    color: #64748b;
  }

  .result-value {
    font-weight: 500;
    text-transform: capitalize;
  }

  .status-done {
    color: #166534;
  }

  .status-retry {
    color: #b45309;
  }

  .status-decomposed {
    color: #1e40af;
  }

  .guard-pass {
    color: #166534;
  }

  .guard-fail {
    color: #dc2626;
  }

  .guard-skipped {
    color: #64748b;
  }

  .summary {
    line-height: 1.5;
    color: #1e293b;
    white-space: pre-wrap;
  }

  .guard-log-toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: none;
    border: none;
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
    color: #475569;
    padding: 0;
  }

  .guard-log-toggle:hover {
    color: #1e293b;
  }

  .toggle {
    font-size: 0.625rem;
  }

  .guard-log {
    margin-top: 0.5rem;
    padding: 0.75rem;
    background-color: #1e293b;
    color: #e2e8f0;
    border-radius: 0.375rem;
    font-family: ui-monospace, monospace;
    font-size: 0.75rem;
    overflow: auto;
    max-height: 20rem;
    white-space: pre-wrap;
  }

  .timestamps {
    display: flex;
    gap: 1.5rem;
    font-size: 0.75rem;
  }

  .timestamp {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .timestamp-label {
    color: #64748b;
  }

  .timestamp-value {
    font-family: ui-monospace, monospace;
    color: #475569;
  }
</style>
