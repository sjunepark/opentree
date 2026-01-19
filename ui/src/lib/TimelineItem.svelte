<script lang="ts">
  import type { TimelineEntry } from './types';
  import { selection, selectIteration, makeIterKey, data } from './stores.svelte';
  import { findNodeById } from './tree-utils';

  interface Props {
    entry: TimelineEntry;
  }

  let { entry }: Props = $props();

  const isSelected = $derived(selection.iterKey === makeIterKey(entry.run_id, entry.iter));
  const nodeTitle = $derived(() => {
    if (!data.tree) return entry.node_id;
    const node = findNodeById(data.tree, entry.node_id);
    return node?.title ?? entry.node_id;
  });

  function getStatusIcon(status: TimelineEntry['status'], guard: TimelineEntry['guard']): string {
    switch (status) {
      case 'running':
        return '●';
      case 'done':
        return guard === 'pass' ? '✓' : guard === 'fail' ? '✗' : '◇';
      case 'retry':
        return '↻';
      case 'decomposed':
        return '◇';
      default:
        return '○';
    }
  }

  function getStatusClass(status: TimelineEntry['status'], guard: TimelineEntry['guard']): string {
    if (status === 'running') return 'running';
    if (status === 'done' && guard === 'pass') return 'pass';
    if (status === 'done' && guard === 'fail') return 'fail';
    if (status === 'retry') return 'retry';
    if (status === 'decomposed') return 'decomposed';
    return '';
  }

  function handleClick() {
    selectIteration(entry.run_id, entry.iter);
  }
</script>

<button
  class="timeline-item"
  class:selected={isSelected}
  onclick={handleClick}
>
  <span class="iter">#{entry.iter}</span>
  <span class="node-id">{nodeTitle()}</span>
  <span class="status {getStatusClass(entry.status, entry.guard)}">
    {getStatusIcon(entry.status, entry.guard)}
  </span>
</button>

<style>
  .timeline-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.5rem 0.75rem;
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    transition: background-color 0.1s;
    font-size: 0.75rem;
  }

  .timeline-item:hover {
    background-color: #f1f5f9;
  }

  .timeline-item.selected {
    background-color: #e0f2fe;
  }

  .iter {
    flex-shrink: 0;
    width: 2rem;
    font-family: ui-monospace, monospace;
    font-weight: 600;
    color: #1e293b;
  }

  .node-id {
    flex: 1;
    font-family: ui-monospace, monospace;
    color: #475569;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .status {
    flex-shrink: 0;
    width: 1rem;
    text-align: center;
    font-weight: 600;
  }

  .status.running {
    color: #3b82f6;
    animation: pulse 1.5s ease-in-out infinite;
  }

  .status.pass {
    color: #16a34a;
  }

  .status.fail {
    color: #dc2626;
  }

  .status.retry {
    color: #f59e0b;
  }

  .status.decomposed {
    color: #8b5cf6;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
</style>
