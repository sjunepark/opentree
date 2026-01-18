<script lang="ts">
  import type { Node } from './types';

  interface Props {
    node: Node;
  }

  let { node }: Props = $props();
</script>

<div class="node-detail">
  <div class="section">
    <h3 class="section-title">Title</h3>
    <p class="section-content">{node.title}</p>
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

  <div class="section">
    <h3 class="section-title">Status</h3>
    <div class="status-grid">
      <div class="status-item">
        <span class="status-label">Passes</span>
        <span class="status-value" class:pass={node.passes} class:fail={!node.passes}>
          {node.passes ? 'Yes' : 'No'}
        </span>
      </div>
      <div class="status-item">
        <span class="status-label">Attempts</span>
        <span class="status-value">{node.attempts} / {node.max_attempts}</span>
      </div>
      <div class="status-item">
        <span class="status-label">Order</span>
        <span class="status-value">{node.order}</span>
      </div>
      <div class="status-item">
        <span class="status-label">Children</span>
        <span class="status-value">{node.children.length}</span>
      </div>
    </div>
  </div>

  <div class="section">
    <h3 class="section-title">ID</h3>
    <code class="node-id">{node.id}</code>
  </div>
</div>

<style>
  .node-detail {
    display: flex;
    flex-direction: column;
    gap: 1rem;
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

  .status-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 0.75rem;
  }

  .status-item {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .status-label {
    font-size: 0.75rem;
    color: #64748b;
  }

  .status-value {
    font-weight: 500;
    color: #1e293b;
  }

  .status-value.pass {
    color: #166534;
  }

  .status-value.fail {
    color: #991b1b;
  }

  .node-id {
    font-family: ui-monospace, monospace;
    font-size: 0.75rem;
    background-color: #f1f5f9;
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    color: #475569;
  }
</style>
