<script lang="ts">
  import type { Node } from './types';
  import { findLeftmostOpenLeaf, findPathToNode, findNodeById } from './tree-utils';
  import { selection, selectNode } from './stores.svelte';
  import { createTreeRenderer } from './d3-tree-renderer';

  interface Props {
    tree: Node;
    activeNodeId?: string | null;
  }

  let { tree, activeNodeId = null }: Props = $props();

  // Container ref for auto-scroll
  let containerEl: HTMLDivElement;

  // Manually expanded node IDs (survives across renders, adds to active path)
  let manuallyExpanded = $state<Set<string>>(new Set());

  // Reset expansions when tree changes
  $effect(() => {
    const _ = tree.id;
    manuallyExpanded = new Set();
  });

  // Compute active node: use prop override or leftmost open leaf
  const activeNode = $derived.by(() => {
    if (activeNodeId) {
      return { id: activeNodeId };
    }
    // Default: leftmost open leaf (matches runner's selector)
    const leaf = findLeftmostOpenLeaf(tree);
    return leaf;
  });

  // Compute base path as Set for O(1) membership checks
  const activePath = $derived.by(() => {
    if (!activeNode) return new Set<string>();
    const pathArray = findPathToNode(tree, activeNode.id);
    return new Set(pathArray);
  });

  // Combined path: active path + manually expanded paths
  const visiblePath = $derived.by(() => {
    const combined = new Set(activePath);
    for (const nodeId of manuallyExpanded) {
      // Add path to each manually expanded node
      const pathArray = findPathToNode(tree, nodeId);
      for (const id of pathArray) {
        combined.add(id);
      }
    }
    return combined;
  });

  // Handle node click - expand collapsed subtrees or select node
  function handleNodeClick(node: Node) {
    // If clicking a node that's not visible, expand its path
    if (!visiblePath.has(node.id)) {
      // Find a leaf in this subtree to expand the full path
      const leaf = findLeftmostOpenLeaf(node);
      const targetId = leaf?.id ?? node.id;
      manuallyExpanded = new Set([...manuallyExpanded, targetId]);
    }
    // Always update selection for detail panel
    selectNode(node);
  }

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
        onNodeClick: handleNodeClick,
      });
      return () => renderer.destroy();
    };
  }

  // Auto-scroll to active node when path changes
  $effect(() => {
    // Re-run when visiblePath changes
    const _ = visiblePath.size;

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
    {@attach createTreeAttachment(tree, visiblePath, selection.nodeId)}
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
