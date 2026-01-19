<script lang="ts">
  import type { Node } from './types';

  interface Props {
    node: Node;
  }

  let { node }: Props = $props();
</script>

<div class="node-detail">
  <div class="title-row">
    <h2 class="node-title">{node.title}</h2>
    <code class="node-id">{node.id}</code>
  </div>

  <div class="section">
    <h3 class="section-title">Goal</h3>
    <p class="section-content goal">{node.goal}</p>
  </div>

  {#if node.acceptance.length > 0}
    <div class="section">
      <h3 class="section-title">Acceptance Criteria</h3>
      <ul class="acceptance-list">
        {#each node.acceptance as criterion}
          <li>{criterion}</li>
        {/each}
      </ul>
    </div>
  {/if}
</div>

<style>
  .node-detail {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .node-title {
    font-size: 1rem;
    font-weight: 600;
    color: #1e293b;
    margin: 0;
  }

  .node-id {
    font-family: ui-monospace, monospace;
    font-size: 0.625rem;
    background-color: #f1f5f9;
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    color: #64748b;
    flex-shrink: 0;
  }

  .section-title {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #64748b;
    margin-bottom: 0.375rem;
  }

  .section-content {
    color: #1e293b;
    line-height: 1.5;
  }

  .goal {
    white-space: pre-wrap;
  }

  .acceptance-list {
    list-style: disc;
    margin-left: 1.25rem;
    color: #1e293b;
  }

  .acceptance-list li {
    margin-bottom: 0.25rem;
  }
</style>
