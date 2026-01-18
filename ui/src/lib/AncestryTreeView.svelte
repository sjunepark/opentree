<script lang="ts">
  import type { Node } from './types';
  import { findLeftmostOpenLeaf, findPathToNode } from './tree-utils';
  import { selection, selectNode } from './stores.svelte';
  import { createTreeRenderer } from './d3-tree-renderer';

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

  // Create attachment function for D3 tree rendering
  function createTreeAttachment(
    treeData: Node,
    path: Set<string>,
    selectedId: string | null
  ) {
    return (svg: SVGSVGElement) => {
      const renderer = createTreeRenderer(svg, {
        tree: treeData,
        activePath: path,
        selectedNodeId: selectedId,
        onNodeClick: (node) => selectNode(node),
      });
      return () => renderer.destroy();
    };
  }

  // Auto-scroll to active node when path changes
  $effect(() => {
    // Re-run when activePath changes
    const _ = activePath.size;

    // Use setTimeout to ensure DOM is updated
    setTimeout(() => {
      if (!containerEl) return;
      const activeLeaf = containerEl.querySelector('.node[data-node-id] rect.node-glow');
      if (activeLeaf) {
        const nodeGroup = activeLeaf.closest('.node');
        if (nodeGroup) {
          nodeGroup.scrollIntoView({ behavior: 'smooth', block: 'center' });
        }
      }
    }, 50);
  });
</script>

<div class="ancestry-tree-view" bind:this={containerEl}>
  <svg
    class="tree-svg"
    {@attach createTreeAttachment(tree, activePath, selection.nodeId)}
  ></svg>
</div>

<style>
  .ancestry-tree-view {
    font-family: system-ui, -apple-system, sans-serif;
    overflow: auto;
    height: 100%;
    padding: 0.5rem;
  }

  .tree-svg {
    display: block;
    min-width: 100%;
    min-height: 100%;
    cursor: grab;
  }

  .tree-svg:active {
    cursor: grabbing;
  }

  /* Global styles for SVG elements rendered by D3 */
  :global(.ancestry-tree-view .link) {
    transition: stroke 0.15s ease;
  }

  :global(.ancestry-tree-view .node) {
    transition: opacity 0.15s ease;
  }

  :global(.ancestry-tree-view .node:hover) {
    opacity: 1 !important;
  }

  :global(.ancestry-tree-view .node:hover .node-rect) {
    fill: #f1f5f9;
  }

  :global(.ancestry-tree-view .node-title) {
    user-select: none;
    pointer-events: none;
  }

  :global(.ancestry-tree-view .node-goal) {
    user-select: none;
    pointer-events: none;
  }

  :global(.ancestry-tree-view .status-badge-text) {
    user-select: none;
    pointer-events: none;
  }

  :global(.ancestry-tree-view .collapse-badge-text) {
    user-select: none;
    pointer-events: none;
  }
</style>
