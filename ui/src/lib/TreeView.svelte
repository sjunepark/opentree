<script lang="ts">
  import type { Node } from './types';
  import TreeNode from './TreeNode.svelte';

  interface Props {
    tree: Node;
  }

  let { tree }: Props = $props();
</script>

<div class="tree-view">
  <TreeNode node={tree} depth={0} />
</div>

<style>
  .tree-view {
    font-size: 0.875rem;
    overflow: auto;
    height: 100%;
  }
</style>
