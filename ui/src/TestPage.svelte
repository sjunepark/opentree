<script lang="ts">
  import AncestryTreeView from './lib/AncestryTreeView.svelte';
  import type { Node } from './lib/types';

  // Test fixtures matching the test file
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
      goal: id === 'root' ? 'Complete the authentication system' : '',
      acceptance: [],
      passes,
      attempts: passes ? 1 : 0,
      max_attempts: 3,
      children,
    };
  }

  // Deep nested tree with active path
  const simpleTree = createNode('root', 'Root', true, [
    createNode('a', 'Child A (passed)', true),
    createNode('b', 'Child B (active)', false),
  ]);

  // Tree with +N badges
  const treeWithBadges = createNode('root', 'Project Root', true, [
    createNode('auth', 'Auth Module', true, [
      createNode('login', 'Login', true),
      createNode('logout', 'Logout', true),
      createNode('register', 'Register', true),
    ]),
    createNode('api', 'API Layer', false),
  ]);

  // Deeply nested tree
  const deepTree = createNode('root', 'Deep Root', true, [
    createNode('l1', 'Level 1', true, [
      createNode('l2', 'Level 2', true, [
        createNode('l3', 'Level 3', true, [
          createNode('l4', 'Level 4 (active leaf)', false),
        ]),
      ]),
    ]),
  ]);

  // All passed tree
  const allPassedTree = createNode('root', 'All Passed', true, [
    createNode('a', 'Task A', true),
    createNode('b', 'Task B', true),
  ]);

  let selectedTree = $state<'simple' | 'badges' | 'deep' | 'allPassed'>('simple');

  const trees = {
    simple: simpleTree,
    badges: treeWithBadges,
    deep: deepTree,
    allPassed: allPassedTree,
  };
</script>

<div class="test-page">
  <h1>AncestryTreeView Test Page</h1>

  <div class="controls">
    <label>
      <input type="radio" bind:group={selectedTree} value="simple" />
      Simple (2 children)
    </label>
    <label>
      <input type="radio" bind:group={selectedTree} value="badges" />
      With +N badges
    </label>
    <label>
      <input type="radio" bind:group={selectedTree} value="deep" />
      Deep nesting (4 levels)
    </label>
    <label>
      <input type="radio" bind:group={selectedTree} value="allPassed" />
      All passed
    </label>
  </div>

  <div class="tree-container">
    <AncestryTreeView tree={trees[selectedTree]} />
  </div>
</div>

<style>
  .test-page {
    padding: 2rem;
    font-family: system-ui, sans-serif;
  }

  h1 {
    margin-bottom: 1.5rem;
    font-size: 1.5rem;
  }

  .controls {
    display: flex;
    gap: 1.5rem;
    margin-bottom: 1.5rem;
    padding: 1rem;
    background: #f5f5f5;
    border-radius: 0.5rem;
  }

  label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
  }

  .tree-container {
    border: 1px solid #e2e8f0;
    border-radius: 0.5rem;
    height: 400px;
    overflow: auto;
  }
</style>
