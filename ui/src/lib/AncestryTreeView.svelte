<script lang="ts">
  import type { Node } from './types';
  import { findLeftmostOpenLeaf, findPathToNode } from './tree-utils';
  import AncestryNode from './AncestryNode.svelte';

  interface Props {
    tree: Node;
    activeNodeId?: string | null;
  }

  let { tree, activeNodeId = null }: Props = $props();

  // Container ref for auto-scroll
  let containerEl: HTMLDivElement;

  // Compute active node: use override or find leftmost open leaf
  const activeNode = $derived.by(() => {
    if (activeNodeId) {
      // Override provided - find that node
      return { id: activeNodeId };
    }
    // Default: leftmost open leaf (matches runner's selector)
    const leaf = findLeftmostOpenLeaf(tree);
    return leaf;
  });

  // Compute path as Set for O(1) membership checks
  const activePath = $derived.by(() => {
    if (!activeNode) return new Set<string>();
    const pathArray = findPathToNode(tree, activeNode.id);
    return new Set(pathArray);
  });

  // Auto-scroll to active node when path changes
  $effect(() => {
    // Re-run when activePath changes
    const _ = activePath.size;

    // Use setTimeout to ensure DOM is updated
    setTimeout(() => {
      if (!containerEl) return;
      const activeLeaf = containerEl.querySelector('.node-card.active-leaf');
      if (activeLeaf) {
        activeLeaf.scrollIntoView({ behavior: 'smooth', block: 'center' });
      }
    }, 50);
  });
</script>

<div class="ancestry-tree-view" bind:this={containerEl}>
  <AncestryNode node={tree} {activePath} isRoot={true} />
</div>

<style>
  .ancestry-tree-view {
    font-size: 0.875rem;
    overflow: auto;
    height: 100%;
    padding: 0.5rem;
  }
</style>
