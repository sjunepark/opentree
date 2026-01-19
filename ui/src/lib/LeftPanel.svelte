<script lang="ts">
  import type { Node } from './types';
  import AncestryTreeView from './AncestryTreeView.svelte';

  interface Props {
    tree: Node | null;
  }

  let { tree }: Props = $props();
</script>

<div class="left-panel">
  <div class="header">
    <span class="title">Tree</span>
  </div>
  <div class="content">
    {#if tree}
      <AncestryTreeView {tree} />
    {:else}
      <div class="empty">No tree data</div>
    {/if}
  </div>
</div>

<style>
  .left-panel {
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
    overflow: hidden;
    min-height: 0;
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
