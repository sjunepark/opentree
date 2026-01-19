<script lang="ts">
  import type { Node } from './types';
  import { findLeftmostOpenLeaf, findPathToNode } from './tree-utils';
  import { selection, selectNode } from './stores.svelte';
  import { createTreeRenderer } from './d3-tree-renderer';

  interface Props {
    /** The root node of the tree to render */
    tree: Node;
    /** Optional node ID to focus on instead of the default leftmost open leaf */
    activeNodeId?: string | null;
  }

  let { tree, activeNodeId = null }: Props = $props();

  let containerEl: HTMLDivElement;

  /**
   * User-expanded node IDs that persist across clicks.
   * Updated by `handleNodeClick` when toggling expand/collapse.
   * Reset by tree change effect.
   */
  let manuallyExpanded = $state<Set<string>>(new Set());

  /**
   * Resets expansion state when tree identity changes.
   * Clears both manual expansions and selection.
   *
   * Triggers: `tree.id` change
   * Mutates: `manuallyExpanded`, `selection` (via selectNode)
   */
  $effect(() => {
    const _ = tree.id;
    manuallyExpanded = new Set();
    selectNode(null);
  });

  /**
   * Base path from root to the "focus" node.
   * Uses `activeNodeId` prop if provided, otherwise finds the leftmost open leaf.
   *
   * Depends on: `tree`, `activeNodeId`
   */
  const defaultPath = $derived.by(() => {
    if (activeNodeId) {
      return new Set(findPathToNode(tree, activeNodeId));
    }
    const leaf = findLeftmostOpenLeaf(tree);
    if (!leaf) return new Set<string>();
    return new Set(findPathToNode(tree, leaf.id));
  });

  /**
   * All visible nodes in the tree (controls which nodes are rendered).
   * Combines the default path with all manually expanded branches.
   *
   * Depends on: `defaultPath`, `manuallyExpanded`, `tree`
   */
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

  /**
   * Nodes to highlight with blue styling (selected node's ancestry).
   * Falls back to `defaultPath` when nothing is selected.
   *
   * Depends on: `selection.nodeId`, `tree`, `defaultPath`
   */
  const highlightedPath = $derived.by(() => {
    if (selection.nodeId) {
      return new Set(findPathToNode(tree, selection.nodeId));
    }
    return defaultPath;
  });

  /**
   * Handles node click: toggles expand/collapse and updates selection.
   * - Expanded nodes (children visible) → collapse by removing descendants from manuallyExpanded
   * - Collapsed nodes → expand by adding leftmost open leaf path to manuallyExpanded
   */
  function handleNodeClick(node: Node) {
    if (node.children.length > 0) {
      const hasExpandedChildren = node.children.some((c) => expandedPath.has(c.id));

      if (hasExpandedChildren) {
        const newExpanded = new Set(manuallyExpanded);
        for (const child of node.children) {
          removeSubtree(child, newExpanded);
        }
        manuallyExpanded = newExpanded;
      } else {
        const leaf = findLeftmostOpenLeaf(node);
        const targetId = leaf?.id ?? node.id;
        manuallyExpanded = new Set([...manuallyExpanded, targetId]);
      }
    }
    selectNode(node);
  }

  /** Recursively removes a node and all its descendants from a set. */
  function removeSubtree(node: Node, set: Set<string>) {
    set.delete(node.id);
    for (const child of node.children) {
      removeSubtree(child, set);
    }
  }

  /** Creates D3 tree renderer attachment for the SVG element. */
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

  /**
   * Auto-scrolls to the selected node when selection changes.
   *
   * Triggers: `selection.nodeId` change
   */
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

<!--
@component
Interactive D3-powered tree visualization for navigating node ancestry.

## Reactive Data Flow

```
Props: tree, activeNodeId
         │
         ▼
┌─────────────────────┐
│  $state            │
│  manuallyExpanded   │◄──── handleNodeClick (toggle)
└─────────────────────┘
         │
         ▼
┌─────────────────────┐     ┌─────────────────────┐
│  $derived           │     │  $derived           │
│  defaultPath        │────►│  expandedPath       │──► D3 renderer (visibility)
│  (focus node path)  │     │  (all visible)      │
└─────────────────────┘     └─────────────────────┘
         │                           │
         ▼                           ▼
┌─────────────────────┐     ┌─────────────────────┐
│  $derived           │     │  $effect            │
│  highlightedPath    │──► D3 renderer (styling) │  auto-scroll on select
│  (selected ancestry)│     └─────────────────────┘
└─────────────────────┘

$effect (tree.id change) ──► resets manuallyExpanded + selection
```

## Usage
```svelte
<AncestryTreeView {tree} />
<AncestryTreeView {tree} activeNodeId="node-123" />
```
-->
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

  /* Styles for D3-rendered SVG elements (parent scoped, children global) */
  .ancestry-tree-view :global(.link) {
    transition: stroke 0.15s ease;
  }

  .ancestry-tree-view :global(.node) {
    transition: opacity 0.15s ease;
  }

  .ancestry-tree-view :global(.node:hover) {
    opacity: 1 !important;
  }

  .ancestry-tree-view :global(.node:hover .node-rect) {
    fill: #f1f5f9;
  }

  .ancestry-tree-view :global(.node-title),
  .ancestry-tree-view :global(.node-goal),
  .ancestry-tree-view :global(.status-badge-text),
  .ancestry-tree-view :global(.collapse-badge-text) {
    user-select: none;
    pointer-events: none;
  }
</style>
