import { describe, it, expect } from 'vitest';
import { findLeftmostOpenLeaf, findPathToNode, findNodeById } from './tree-utils';
import type { Node } from './types';

// Helper to create a minimal Node for testing
function createNode(
  id: string,
  passes: boolean,
  children: Node[] = []
): Node {
  return {
    id,
    order: 0,
    title: id,
    goal: '',
    acceptance: [],
    next: 'execute',
    passes,
    attempts: 0,
    max_attempts: 3,
    children,
  };
}

describe('findLeftmostOpenLeaf', () => {
  it('returns single unpassed node', () => {
    const node = createNode('root', false);
    expect(findLeftmostOpenLeaf(node)).toBe(node);
  });

  it('returns null for single passed node', () => {
    const node = createNode('root', true);
    expect(findLeftmostOpenLeaf(node)).toBeNull();
  });

  it('returns deepest leftmost unpassed leaf', () => {
    // Tree structure:
    //       root (pass)
    //      /    \
    //     a      b
    //    / \      \
    //   c   d      e
    // Expected: c (leftmost leaf, assuming unpassed)
    const c = createNode('c', false);
    const d = createNode('d', false);
    const e = createNode('e', false);
    const a = createNode('a', true, [c, d]);
    const b = createNode('b', true, [e]);
    const root = createNode('root', true, [a, b]);

    expect(findLeftmostOpenLeaf(root)?.id).toBe('c');
  });

  it('skips passed leaves to find first unpassed', () => {
    // Tree structure:
    //       root
    //      /    \
    //     a      b
    //    / \      \
    //   c   d      e
    // c passes, d does not
    const c = createNode('c', true);
    const d = createNode('d', false);
    const e = createNode('e', false);
    const a = createNode('a', true, [c, d]);
    const b = createNode('b', true, [e]);
    const root = createNode('root', true, [a, b]);

    expect(findLeftmostOpenLeaf(root)?.id).toBe('d');
  });

  it('returns null when all leaves pass', () => {
    const c = createNode('c', true);
    const d = createNode('d', true);
    const a = createNode('a', true, [c, d]);
    const root = createNode('root', true, [a]);

    expect(findLeftmostOpenLeaf(root)).toBeNull();
  });

  it('respects order in wide tree', () => {
    // Wide tree: root -> [a, b, c, d]
    // All unpassed, should return 'a' (first/leftmost)
    const a = createNode('a', false);
    const b = createNode('b', false);
    const c = createNode('c', false);
    const d = createNode('d', false);
    const root = createNode('root', true, [a, b, c, d]);

    expect(findLeftmostOpenLeaf(root)?.id).toBe('a');
  });

  it('traverses depth-first, not breadth-first', () => {
    // Tree:
    //       root
    //      /    \
    //     a      d (unpassed leaf)
    //    / \
    //   b   c (both passed)
    // Depth-first would visit a->b->c before d
    // Since b and c pass, should return d
    const b = createNode('b', true);
    const c = createNode('c', true);
    const a = createNode('a', true, [b, c]);
    const d = createNode('d', false);
    const root = createNode('root', true, [a, d]);

    expect(findLeftmostOpenLeaf(root)?.id).toBe('d');
  });
});

describe('findPathToNode', () => {
  it('returns single-element path for root match', () => {
    const root = createNode('root', false);
    expect(findPathToNode(root, 'root')).toEqual(['root']);
  });

  it('returns full path to deeply nested node', () => {
    const d = createNode('d', false);
    const c = createNode('c', true, [d]);
    const b = createNode('b', true, [c]);
    const a = createNode('a', true, [b]);

    expect(findPathToNode(a, 'd')).toEqual(['a', 'b', 'c', 'd']);
  });

  it('returns empty array for non-existent node', () => {
    const b = createNode('b', false);
    const a = createNode('a', true, [b]);

    expect(findPathToNode(a, 'nonexistent')).toEqual([]);
  });

  it('finds correct path in branching tree', () => {
    //       root
    //      /    \
    //     a      b
    //    / \      \
    //   c   d      e
    const c = createNode('c', false);
    const d = createNode('d', false);
    const e = createNode('e', false);
    const a = createNode('a', true, [c, d]);
    const b = createNode('b', true, [e]);
    const root = createNode('root', true, [a, b]);

    expect(findPathToNode(root, 'e')).toEqual(['root', 'b', 'e']);
    expect(findPathToNode(root, 'd')).toEqual(['root', 'a', 'd']);
  });
});

describe('findNodeById', () => {
  it('returns root if id matches', () => {
    const root = createNode('root', false);
    expect(findNodeById(root, 'root')).toBe(root);
  });

  it('returns null for non-existent id', () => {
    const root = createNode('root', false);
    expect(findNodeById(root, 'nonexistent')).toBeNull();
  });

  it('finds deeply nested node', () => {
    const d = createNode('d', false);
    const c = createNode('c', true, [d]);
    const b = createNode('b', true, [c]);
    const a = createNode('a', true, [b]);

    const found = findNodeById(a, 'd');
    expect(found).toBe(d);
    expect(found?.id).toBe('d');
  });

  it('finds node in branching tree', () => {
    const c = createNode('c', false);
    const d = createNode('d', false);
    const e = createNode('e', false);
    const a = createNode('a', true, [c, d]);
    const b = createNode('b', true, [e]);
    const root = createNode('root', true, [a, b]);

    expect(findNodeById(root, 'e')).toBe(e);
    expect(findNodeById(root, 'a')).toBe(a);
    expect(findNodeById(root, 'c')).toBe(c);
  });

  it('returns actual node reference, not copy', () => {
    const child = createNode('child', false);
    const root = createNode('root', true, [child]);

    const found = findNodeById(root, 'child');
    expect(found).toBe(child); // Same reference
  });
});
