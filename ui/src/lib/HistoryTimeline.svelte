<script lang="ts">
  import { iterations } from './stores.svelte';
  import TimelineItem from './TimelineItem.svelte';

  // Show entries in reverse order (newest first)
  const reversedEntries = $derived([...iterations.entries].reverse());
</script>

<div class="history-timeline">
  <div class="header">
    <span class="title">History</span>
    <span class="count">{iterations.entries.length}</span>
  </div>

  <div class="list">
    {#if reversedEntries.length === 0}
      <div class="empty">No iterations yet</div>
    {:else}
      {#each reversedEntries as entry (entry.run_id + '/' + entry.iter)}
        <TimelineItem {entry} />
      {/each}
    {/if}
  </div>
</div>

<style>
  .history-timeline {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: white;
    border: 1px solid #e2e8f0;
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
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

  .count {
    font-family: ui-monospace, monospace;
    font-size: 0.625rem;
    color: #94a3b8;
    background-color: #f1f5f9;
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
  }

  .list {
    flex: 1;
    overflow-y: auto;
  }

  .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #94a3b8;
    font-size: 0.875rem;
  }
</style>
