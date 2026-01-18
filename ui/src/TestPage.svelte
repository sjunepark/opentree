<script lang="ts">
  import AncestryTreeView from './lib/AncestryTreeView.svelte';
  import type { Node } from './lib/types';

  // Test fixtures matching the test file
  function createNode(
    id: string,
    title: string,
    passes: boolean,
    children: Node[] = [],
    order: number = 0
  ): Node {
    return {
      id,
      order,
      title,
      goal: id === 'root' ? 'Complete the authentication system' : '',
      acceptance: [],
      passes,
      attempts: passes ? 1 : 0,
      max_attempts: 3,
      children,
    };
  }

  // Simple tree with active path
  // Invariant: parent passes only if ALL children pass
  const simpleTree = createNode('root', 'Root', false, [
    createNode('a', 'Child A (passed)', true),
    createNode('b', 'Child B (active)', false),
  ]);

  // Tree with +N badges (collapsed siblings)
  const treeWithBadges = createNode('root', 'Project Root', false, [
    createNode('auth', 'Auth Module', true, [
      createNode('login', 'Login', true),
      createNode('logout', 'Logout', true),
      createNode('register', 'Register', true),
    ]),
    createNode('api', 'API Layer', false), // active leaf, blocks root from passing
  ]);

  // Deeply nested tree
  const deepTree = createNode('root', 'Deep Root', false, [
    createNode('l1', 'Level 1', false, [
      createNode('l2', 'Level 2', false, [
        createNode('l3', 'Level 3', false, [
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

  // Complicated tree with multiple nodes per level
  // Invariants:
  // 1. Parent passes only if ALL children pass (bottom-up)
  // 2. Siblings processed in (order, id) order - later siblings can't decompose
  //    until earlier siblings complete
  //
  // Flow: Auth(0) ✓ → Catalog(1) in progress → Cart(2) pending → Checkout(3) pending
  const complicatedTree = createNode('root', 'E-Commerce Platform', false, [
    // Auth - COMPLETE (will be folded)
    createNode('auth', 'Authentication', true, [
      createNode('register', 'Registration', true, [
        createNode('validation', 'Input Validation', true, [], 0),
        createNode('verify', 'Email Verification', true, [], 1),
      ], 0),
      createNode('login', 'Login Flow', true, [
        createNode('email', 'Email Login', true, [], 0),
        createNode('oauth', 'OAuth Providers', true, [
          createNode('google', 'Google SSO', true, [], 0),
          createNode('github', 'GitHub SSO', true, [], 1),
          createNode('apple', 'Apple SSO', true, [], 2),
        ], 1),
        createNode('mfa', 'MFA Setup', true, [], 2),
      ], 1),
      createNode('password', 'Password Reset', true, [], 2),
    ], 0),
    // Catalog - IN PROGRESS (expanded, being worked on)
    createNode('catalog', 'Product Catalog', false, [
      // Listing(0) done → Detail(1) in progress → Inventory(2) pending
      createNode('listing', 'Product Listing', true, [
        createNode('search', 'Search & Filter', true, [], 0),
        createNode('sort', 'Sorting', true, [], 1),
        createNode('pagination', 'Pagination', true, [], 2),
      ], 0),
      createNode('detail', 'Product Detail', false, [
        // Gallery(0) done → Reviews(1) in progress → Related(2) pending
        createNode('gallery', 'Image Gallery', true, [], 0),
        createNode('reviews', 'Reviews', false, [
          createNode('rating', 'Star Ratings', true, [], 0),
          createNode('comments', 'Comments', false, [], 1),  // ← active leaf
          createNode('helpful', 'Helpful Votes', false, [], 2),
        ], 1),
        createNode('related', 'Related Items', false, [], 2),
      ], 1),
      createNode('inventory', 'Inventory Mgmt', false, [], 2),
    ], 1),
    // Cart - pending (catalog incomplete)
    createNode('cart', 'Shopping Cart', false, [], 2),
    // Checkout - pending (catalog incomplete)
    createNode('checkout', 'Checkout', false, [], 3),
  ]);

  let selectedTree = $state<'simple' | 'badges' | 'deep' | 'allPassed' | 'complicated'>('simple');

  const trees = {
    simple: simpleTree,
    badges: treeWithBadges,
    deep: deepTree,
    allPassed: allPassedTree,
    complicated: complicatedTree,
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
    <label>
      <input type="radio" bind:group={selectedTree} value="complicated" />
      Complicated (multi-node)
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
