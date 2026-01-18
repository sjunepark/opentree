<script lang="ts">
  import type { Node } from './types';
  import { selection, selectNode } from './stores.svelte';
  import AncestryNode from './AncestryNode.svelte';

  interface Props {
    node: Node;
    activePath: Set<string>;
    isRoot?: boolean;
  }

  let { node, activePath, isRoot = false }: Props = $props();

  const isOnPath = $derived(activePath.has(node.id));
  const isSelected = $derived(selection.nodeId === node.id);
  const hasChildren = $derived(node.children.length > 0);
  const isActiveLeaf = $derived(isOnPath && !hasChildren);

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
</script>

<div class="ancestry-node" class:on-path={isOnPath} class:is-root={isRoot}>
  <!-- Connector line from parent (not for root) -->
  {#if !isRoot}
    <div class="connector-vertical" class:active={isOnPath}></div>
  {/if}

  <!-- Node card -->
  <div
    class="node-card"
    class:selected={isSelected}
    class:active-leaf={isActiveLeaf}
    class:on-path={isOnPath}
    onclick={select}
    onkeydown={(e) => e.key === 'Enter' && select(e as unknown as MouseEvent)}
    role="button"
    tabindex="0"
  >
    <div class="node-header">
      <span class="node-title">{node.title}</span>
      <span class="node-status {getStatusClass(node)}">{getStatusLabel(node)}</span>
    </div>
    {#if isRoot && node.goal}
      <div class="node-goal">{node.goal}</div>
    {/if}
  </div>

  <!-- Children section -->
  {#if hasChildren}
    <div class="children-container">
      {#each node.children as child (child.id)}
        {@const childOnPath = activePath.has(child.id)}
        {#if childOnPath}
          <!-- Expanded child on active path -->
          <AncestryNode node={child} {activePath} />
        {:else}
          <!-- Collapsed sibling -->
          <div class="collapsed-sibling">
            <div class="connector-horizontal"></div>
            <div
              class="sibling-card"
              class:selected={selection.nodeId === child.id}
              onclick={(e) => { e.stopPropagation(); selectNode(child); }}
              onkeydown={(e) => e.key === 'Enter' && selectNode(child)}
              role="button"
              tabindex="0"
            >
              <span class="sibling-title">{child.title}</span>
              <span class="node-status {getStatusClass(child)}">{getStatusLabel(child)}</span>
              {#if child.children.length > 0}
                <span class="child-count">+{child.children.length}</span>
              {/if}
            </div>
          </div>
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .ancestry-node {
    position: relative;
    display: flex;
    flex-direction: column;
  }

  .connector-vertical {
    position: absolute;
    left: 1rem;
    top: -0.75rem;
    width: 2px;
    height: 0.75rem;
    background-color: #e2e8f0;
  }

  .connector-vertical.active {
    background-color: #3b82f6;
  }

  .node-card {
    padding: 0.5rem 0.75rem;
    background: #f8fafc;
    border: 1px solid #e2e8f0;
    border-radius: 0.375rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .node-card:hover {
    background: #f1f5f9;
  }

  .node-card.selected {
    background: #e0f2fe;
    border-color: #7dd3fc;
  }

  .node-card.on-path {
    border-color: #93c5fd;
  }

  .node-card.active-leaf {
    background: #dbeafe;
    border-color: #3b82f6;
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
  }

  .node-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .node-title {
    flex: 1;
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .node-goal {
    margin-top: 0.25rem;
    font-size: 0.75rem;
    color: #64748b;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .node-status {
    font-size: 0.625rem;
    padding: 0.125rem 0.375rem;
    border-radius: 9999px;
    font-weight: 500;
    flex-shrink: 0;
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

  .children-container {
    margin-left: 1rem;
    padding-left: 1rem;
    margin-top: 0.5rem;
    border-left: 2px solid #e2e8f0;
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .children-container:has(.ancestry-node.on-path) {
    border-left-color: #3b82f6;
  }

  .collapsed-sibling {
    display: flex;
    align-items: center;
    position: relative;
  }

  .connector-horizontal {
    position: absolute;
    left: -1rem;
    width: 1rem;
    height: 2px;
    background-color: #e2e8f0;
  }

  .sibling-card {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.375rem 0.625rem;
    background: #fafafa;
    border: 1px solid #e2e8f0;
    border-radius: 0.25rem;
    cursor: pointer;
    opacity: 0.7;
    transition: all 0.15s ease;
    flex: 1;
  }

  .sibling-card:hover {
    opacity: 1;
    background: #f1f5f9;
  }

  .sibling-card.selected {
    opacity: 1;
    background: #e0f2fe;
    border-color: #7dd3fc;
  }

  .sibling-title {
    flex: 1;
    font-size: 0.8125rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .child-count {
    font-size: 0.625rem;
    padding: 0.0625rem 0.25rem;
    background: #e2e8f0;
    color: #64748b;
    border-radius: 0.25rem;
    flex-shrink: 0;
  }
</style>
