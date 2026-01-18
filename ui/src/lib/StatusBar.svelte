<script lang="ts">
  import { connection, data, stream, timer } from './stores.svelte';

  // Update elapsed time every second when there's an active iteration
  $effect(() => {
    if (stream.activeIter !== null && timer.startTime !== null) {
      const interval = setInterval(() => {
        timer.elapsed = Date.now() - timer.startTime!;
      }, 1000);
      return () => clearInterval(interval);
    }
  });

  function formatElapsed(ms: number): string {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const s = seconds % 60;
    const m = minutes % 60;
    if (hours > 0) {
      return `${hours}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
    }
    return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  }

  // Find current node from the active iteration
  function getCurrentNodeId(): string | null {
    if (!stream.activeRunId || stream.activeIter === null) return null;
    // Look up from run state if available
    return data.runState?.last_summary ? null : null;
  }
</script>

<header class="status-bar">
  <div class="left">
    <span class="connection" class:connected={connection.sseConnected}>
      {connection.sseConnected ? '● Connected' : '○ Disconnected'}
    </span>
  </div>

  <div class="center">
    {#if data.runState?.run_id}
      <span class="run-id">Run: {data.runState.run_id}</span>
      {#if stream.activeIter !== null}
        <span class="separator">│</span>
        <span class="iter-num">#{stream.activeIter}</span>
      {/if}
      {#if timer.startTime !== null}
        <span class="separator">│</span>
        <span class="timer">⏱ {formatElapsed(timer.elapsed)}</span>
      {/if}
    {:else}
      <span class="no-run">No active run</span>
    {/if}
  </div>

  <div class="right">
    {#if connection.loading}
      <span class="loading">Loading...</span>
    {:else if connection.error}
      <span class="error" title={connection.error}>Error</span>
    {/if}
  </div>
</header>

<style>
  .status-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 1rem;
    background-color: #1e293b;
    color: #e2e8f0;
    font-size: 0.75rem;
    font-family: ui-monospace, monospace;
    flex-shrink: 0;
  }

  .left, .center, .right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .left {
    flex: 0 0 auto;
  }

  .center {
    flex: 1;
    justify-content: center;
  }

  .right {
    flex: 0 0 auto;
  }

  .connection {
    color: #f87171;
  }

  .connection.connected {
    color: #4ade80;
  }

  .run-id {
    color: #94a3b8;
  }

  .separator {
    color: #475569;
  }

  .iter-num {
    color: #60a5fa;
    font-weight: 600;
  }

  .timer {
    color: #fbbf24;
  }

  .no-run {
    color: #64748b;
  }

  .loading {
    color: #fbbf24;
  }

  .error {
    color: #f87171;
    cursor: help;
  }
</style>
