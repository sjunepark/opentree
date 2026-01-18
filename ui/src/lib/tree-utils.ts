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
