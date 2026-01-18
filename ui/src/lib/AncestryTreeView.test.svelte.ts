import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import AncestryTreeView from './AncestryTreeView.svelte';
import { selection, clearSelection } from './stores.svelte';
import type { Node } from './types';

// Helper to create a minimal Node for testing
function createNode(
  id: string,
  title: string,
  passes: boolean,
  children: Node[] = []
): Node {
  return {
    id,
    order: 0,
    title,
    goal: id === 'root' ? 'Test goal' : '',
    acceptance: [],
    passes,
    attempts: 0,
    max_attempts: 3,
    children,
  };
}

// Reset stores between tests
beforeEach(() => {
  clearSelection();
});

describe('AncestryTreeView', () => {
  it('renders root node with goal', () => {
    const tree = createNode('root', 'Root Node', false);

    render(AncestryTreeView, { props: { tree } });

    expect(screen.getByText('Root Node')).toBeInTheDocument();
    expect(screen.getByText('Test goal')).toBeInTheDocument();
  });

  it('auto-expands active path to leftmost open leaf', () => {
    // Tree: root -> [a (pass), b (fail)]
    // b should be the active leaf since a passes
    const a = createNode('a', 'Child A', true);
    const b = createNode('b', 'Child B', false);
    const root = createNode('root', 'Root', true, [a, b]);

    render(AncestryTreeView, { props: { tree: root } });

    // Root should be visible
    expect(screen.getByText('Root')).toBeInTheDocument();

    // Both children visible (b is on path, a is collapsed sibling)
    expect(screen.getByText('Child A')).toBeInTheDocument();
    expect(screen.getByText('Child B')).toBeInTheDocument();
  });

  it('shows +N badge for collapsed siblings with children', () => {
    // Tree:
    //     root
    //    /    \
    //   a      b (active path)
    //  / \
    // c   d
    const c = createNode('c', 'Grandchild C', true);
    const d = createNode('d', 'Grandchild D', true);
    const a = createNode('a', 'Child A', true, [c, d]);
    const b = createNode('b', 'Child B', false); // Active leaf
    const root = createNode('root', 'Root', true, [a, b]);

    render(AncestryTreeView, { props: { tree: root } });

    // Child A has 2 children, should show +2 badge
    expect(screen.getByText('+2')).toBeInTheDocument();
  });

  it('clicking sibling updates selection', async () => {
    const user = userEvent.setup();

    const a = createNode('a', 'Child A', true);
    const b = createNode('b', 'Child B', false);
    const root = createNode('root', 'Root', true, [a, b]);

    render(AncestryTreeView, { props: { tree: root } });

    // Initially no selection
    expect(selection.nodeId).toBeNull();

    // Click on sibling (Child A)
    await user.click(screen.getByText('Child A'));

    // Selection should update to Child A
    expect(selection.nodeId).toBe('a');
    expect(selection.type).toBe('node');
  });

  it('respects activeNodeId override', () => {
    // Tree: root -> a -> b
    // Without override, b would be active (leftmost open leaf)
    // With override to 'a', a should be active
    const b = createNode('b', 'Deepest', false);
    const a = createNode('a', 'Middle', true, [b]);
    const root = createNode('root', 'Root', true, [a]);

    render(AncestryTreeView, { props: { tree: root, activeNodeId: 'a' } });

    // All nodes should be visible since 'a' is on the path
    expect(screen.getByText('Root')).toBeInTheDocument();
    expect(screen.getByText('Middle')).toBeInTheDocument();
  });

  it('handles all-passed tree gracefully', () => {
    const b = createNode('b', 'Child B', true);
    const a = createNode('a', 'Child A', true, [b]);
    const root = createNode('root', 'Root', true, [a]);

    // When all pass, no active leaf, but tree should still render
    render(AncestryTreeView, { props: { tree: root } });

    expect(screen.getByText('Root')).toBeInTheDocument();
  });

  it('renders deeply nested active path', () => {
    // Tree: root -> a -> b -> c (active leaf)
    const c = createNode('c', 'Level 3', false);
    const b = createNode('b', 'Level 2', true, [c]);
    const a = createNode('a', 'Level 1', true, [b]);
    const root = createNode('root', 'Root', true, [a]);

    render(AncestryTreeView, { props: { tree: root } });

    // All levels should be expanded and visible
    expect(screen.getByText('Root')).toBeInTheDocument();
    expect(screen.getByText('Level 1')).toBeInTheDocument();
    expect(screen.getByText('Level 2')).toBeInTheDocument();
    expect(screen.getByText('Level 3')).toBeInTheDocument();
  });
});
