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

  // Manually expanded node IDs (persists across clicks)
  let manuallyExpanded = $state<Set<string>>(new Set());

  // Reset expansions when tree changes
  $effect(() => {
    const _ = tree.id;
    manuallyExpanded = new Set();
    selectNode(null);
  });

  // Default path: leftmost open leaf or activeNodeId prop
  const defaultPath = $derived.by(() => {
    if (activeNodeId) {
      return new Set(findPathToNode(tree, activeNodeId));
    }
    const leaf = findLeftmostOpenLeaf(tree);
    if (!leaf) return new Set<string>();
    return new Set(findPathToNode(tree, leaf.id));
  });

  // Expanded path: default + all manually expanded paths (controls visibility)
  const expandedPath = $derived.by(() => {
    const combined = new Set(defaultPath);
    for (const nodeId of manuallyExpanded) {
      const pathArray = findPathToNode(tree, nodeId);
      for (const id of pathArray) {
        combined.add(id);
      }
    }
    return combined;
  });

  // Highlighted path: only the selected node's ancestors (controls blue styling)
  const highlightedPath = $derived.by(() => {
    if (selection.nodeId) {
      return new Set(findPathToNode(tree, selection.nodeId));
    }
    // When no selection, highlight the default path
    return defaultPath;
  });

  // Handle node click - expand if collapsed, always select
  function handleNodeClick(node: Node) {
    // If clicking a collapsed node (not in expanded path), expand it
    if (!expandedPath.has(node.id)) {
      const leaf = findLeftmostOpenLeaf(node);
      const targetId = leaf?.id ?? node.id;
      manuallyExpanded = new Set([...manuallyExpanded, targetId]);
    }
    selectNode(node);
  }

  // Create attachment function for D3 tree rendering
  function createTreeAttachment(
    treeData: Node,
    expanded: Set<string>,
    highlighted: Set<string>,
    selectedId: string | null
  ) {
    return (svg: SVGSVGElement) => {
      const renderer = createTreeRenderer(svg, {
        tree: treeData,
        expandedPath: expanded,
        highlightedPath: highlighted,
        selectedNodeId: selectedId,
        onNodeClick: handleNodeClick,
      });
      return () => renderer.destroy();
    };
  }

  // Auto-scroll to selected node when selection changes
  $effect(() => {
    const _ = selection.nodeId;
    setTimeout(() => {
      if (!containerEl || !selection.nodeId) return;
      const selectedNode = containerEl.querySelector(`.node[data-node-id="${selection.nodeId}"]`);
      if (selectedNode) {
        selectedNode.scrollIntoView({ behavior: 'smooth', block: 'center' });
      }
    }, 50);
  });
</script>

<div class="ancestry-tree-view" bind:this={containerEl}>
  <svg
    class="tree-svg"
    {@attach createTreeAttachment(tree, expandedPath, highlightedPath, selection.nodeId)}
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
