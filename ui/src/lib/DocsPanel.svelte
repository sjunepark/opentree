<script lang="ts">
  type DocTab = 'assumptions' | 'questions';

  interface Props {
    assumptions: string;
    questions: string;
  }

  let { assumptions, questions }: Props = $props();
  let activeDocTab: DocTab = $state('assumptions');
</script>

<div class="docs-panel">
  <div class="doc-tabs">
    <button
      class="doc-tab"
      class:active={activeDocTab === 'assumptions'}
      onclick={() => (activeDocTab = 'assumptions')}
    >
      Assumptions
    </button>
    <button
      class="doc-tab"
      class:active={activeDocTab === 'questions'}
      onclick={() => (activeDocTab = 'questions')}
    >
      Questions
    </button>
  </div>

  <div class="doc-content">
    {#if activeDocTab === 'assumptions'}
      {#if assumptions}
        <pre class="markdown-content">{assumptions}</pre>
      {:else}
        <div class="empty-state">No assumptions defined</div>
      {/if}
    {:else}
      {#if questions}
        <pre class="markdown-content">{questions}</pre>
      {:else}
        <div class="empty-state">No questions defined</div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .docs-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .doc-tabs {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
    border-bottom: 1px solid #e2e8f0;
    padding-bottom: 0.5rem;
  }

  .doc-tab {
    padding: 0.375rem 0.75rem;
    font-size: 0.75rem;
    font-weight: 500;
    color: #64748b;
    background: none;
    border: none;
    border-radius: 0.25rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .doc-tab:hover {
    background-color: #f1f5f9;
    color: #1e293b;
  }

  .doc-tab.active {
    background-color: #3b82f6;
    color: white;
  }

  .doc-content {
    flex: 1;
    overflow: auto;
  }

  .markdown-content {
    font-family: ui-monospace, monospace;
    font-size: 0.8125rem;
    line-height: 1.6;
    color: #1e293b;
    white-space: pre-wrap;
    word-wrap: break-word;
    margin: 0;
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #94a3b8;
    font-size: 0.875rem;
  }
</style>
