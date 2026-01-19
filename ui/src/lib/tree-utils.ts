// Tree traversal utilities for ancestry tree view.
// Mirrors runner's selector.rs logic for consistency.

import type { Node } from './types';

/**
 * Find the first leaf with passes=false via depth-first traversal.
 * Returns null if all leaves pass (tree is complete).
 *
 * Mirrors: runner/src/core/selector.rs:8-20
 */
export function findLeftmostOpenLeaf(node: Node): Node | null {
  if (node.children.length === 0) {
    return node.passes ? null : node;
  }

  for (const child of node.children) {
    const found = findLeftmostOpenLeaf(child);
    if (found) return found;
  }

  return null;
}

/**
 * Find the path from root to a target node by ID.
 * Returns array of node IDs from root to target (inclusive).
 * Returns empty array if target not found.
 */
export function findPathToNode(root: Node, targetId: string): string[] {
  if (root.id === targetId) {
    return [root.id];
  }

  for (const child of root.children) {
    const childPath = findPathToNode(child, targetId);
    if (childPath.length > 0) {
      return [root.id, ...childPath];
    }
  }

  return [];
}

/**
 * Find a node by ID in the tree.
 * Returns null if not found.
 */
export function findNodeById(root: Node, targetId: string): Node | null {
  if (root.id === targetId) {
    return root;
  }

  for (const child of root.children) {
    const found = findNodeById(child, targetId);
    if (found) return found;
  }

  return null;
}

/**
 * Find the deepest passing leaf in the tree via depth-first traversal.
 * Returns the last (rightmost) passing leaf, or root if no leaves pass.
 */
export function findDeepestPassingLeaf(node: Node): Node {
  // If this node has passing children, recurse into the last one
  const passingChildren = node.children.filter((c) => c.passes);
  if (passingChildren.length > 0) {
    return findDeepestPassingLeaf(passingChildren[passingChildren.length - 1]);
  }

  // No passing children - return this node if it passes, or just this node as fallback
  return node;
}

/**
 * Find the best node to auto-select on load:
 * 1. Current working node (leftmost open leaf with attempts > 0)
 * 2. Leftmost open leaf (next to work on)
 * 3. Deepest passing leaf (if tree is complete)
 */
export function findAutoSelectNode(root: Node): Node {
  // First, check for a node currently being worked on (has attempts but not passing)
  const workingNode = findNodeInProgress(root);
  if (workingNode) return workingNode;

  // Next, try the leftmost open leaf (next node to work on)
  const openLeaf = findLeftmostOpenLeaf(root);
  if (openLeaf) return openLeaf;

  // Tree is complete - return the deepest passing leaf
  return findDeepestPassingLeaf(root);
}

/**
 * Find a node that's currently in progress (attempts > 0 but not passing).
 */
function findNodeInProgress(node: Node): Node | null {
  // Check if this node is in progress
  if (!node.passes && node.attempts > 0 && node.attempts < node.max_attempts) {
    // But prefer to find the deepest in-progress node
    for (const child of node.children) {
      const found = findNodeInProgress(child);
      if (found) return found;
    }
    return node;
  }

  // Recurse into children
  for (const child of node.children) {
    const found = findNodeInProgress(child);
    if (found) return found;
  }

  return null;
}
