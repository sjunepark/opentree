<script lang="ts">
  import type { Node } from './types';
  import { selection, selectNode } from './stores.svelte';
  import TreeNode from './TreeNode.svelte';

  interface Props {
    node: Node;
    depth: number;
  }

  let { node, depth }: Props = $props();

  // Initial expansion state based on depth (depth is constant per component instance)
  let expanded = $state(false);
  $effect(() => {
    expanded = depth < 2;
  });

  function toggle() {
    if (node.children.length > 0) {
      expanded = !expanded;
    }
  }

  function select(event: MouseEvent) {
    event.stopPropagation();
    selectNode(node);
  }

  function getStatusClass(node: Node): string {
    if (node.passes) return 'status-pass';
    if (node.attempts >= node.max_attempts) return 'status-fail';
    if (node.attempts > 0) return 'status-running';
    return 'status-pending';
  }

  function getStatusLabel(node: Node): string {
    if (node.passes) return 'pass';
    if (node.attempts >= node.max_attempts) return 'fail';
    if (node.attempts > 0) return `${node.attempts}/${node.max_attempts}`;
    return 'pending';
  }

  const isSelected = $derived(selection.nodeId === node.id);
</script>

<div class="tree-node" style="--depth: {depth}">
  <div
    class="node-row"
    class:selected={isSelected}
    class:has-children={node.children.length > 0}
    onclick={select}
    onkeydown={(e) => e.key === 'Enter' && select(e as unknown as MouseEvent)}
    role="button"
    tabindex="0"
  >
    {#if node.children.length > 0}
      <button class="toggle" onclick={toggle} aria-label={expanded ? 'Collapse' : 'Expand'}>
        {expanded ? '▼' : '▶'}
      </button>
    {:else}
      <span class="toggle-placeholder"></span>
    {/if}

    <span class="node-title">{node.title}</span>
    <span class="node-status {getStatusClass(node)}">{getStatusLabel(node)}</span>
  </div>

  {#if expanded && node.children.length > 0}
    <div class="children">
      {#each node.children as child (child.id)}
        <TreeNode node={child} depth={depth + 1} />
      {/each}
    </div>
  {/if}
</div>

<style>
  .tree-node {
    --indent: calc(var(--depth) * 1rem);
  }

  .node-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.375rem 0.5rem;
    padding-left: calc(0.5rem + var(--indent));
    cursor: pointer;
    border-radius: 0.25rem;
    transition: background-color 0.1s;
  }

  .node-row:hover {
    background-color: #f1f5f9;
  }

  .node-row.selected {
    background-color: #e0f2fe;
  }

  .toggle {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    width: 1rem;
    font-size: 0.625rem;
    color: #64748b;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .toggle-placeholder {
    width: 1rem;
  }

  .node-title {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .node-status {
    font-size: 0.75rem;
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
    font-weight: 500;
  }

  .status-pass {
    background-color: #dcfce7;
    color: #166534;
  }

  .status-fail {
    background-color: #fee2e2;
    color: #991b1b;
  }

  .status-running {
    background-color: #dbeafe;
    color: #1e40af;
  }

  .status-pending {
    background-color: #f1f5f9;
    color: #64748b;
  }
</style>
