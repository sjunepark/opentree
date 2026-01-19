<script lang="ts">
  import type { RunnerConfig } from './types';
  import { rightPanel, setRightPanelTab } from './stores.svelte';
  import ContextPanel from './ContextPanel.svelte';
  import ConfigPanel from './ConfigPanel.svelte';
  import DocsPanel from './DocsPanel.svelte';
  import LiveOutput from './LiveOutput.svelte';

  interface Props {
    config: RunnerConfig | null;
    assumptions: string;
    questions: string;
  }

  let { config, assumptions, questions }: Props = $props();
</script>

<div class="right-panel">
  <div class="tabs-section">
    <div class="tab-bar">
      <button
        class="tab"
        class:active={rightPanel.activeTab === 'details'}
        onclick={() => setRightPanelTab('details')}
      >
        Details
      </button>
      <button
        class="tab"
        class:active={rightPanel.activeTab === 'config'}
        onclick={() => setRightPanelTab('config')}
      >
        Config
      </button>
      <button
        class="tab"
        class:active={rightPanel.activeTab === 'docs'}
        onclick={() => setRightPanelTab('docs')}
      >
        Docs
      </button>
    </div>
    <div class="tab-content">
      {#if rightPanel.activeTab === 'details'}
        <ContextPanel />
      {:else if rightPanel.activeTab === 'config'}
        {#if config}
          <ConfigPanel {config} />
        {:else}
          <div class="empty">No config loaded</div>
        {/if}
      {:else if rightPanel.activeTab === 'docs'}
        <DocsPanel {assumptions} {questions} />
      {/if}
    </div>
  </div>
  <div class="logs-section">
    <LiveOutput />
  </div>
</div>

<style>
  .right-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    gap: 0.75rem;
  }

  .tabs-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: white;
    border: 1px solid #e2e8f0;
    border-radius: 0.5rem;
    overflow: hidden;
    min-height: 0;
  }

  .tab-bar {
    display: flex;
    gap: 0.25rem;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid #e2e8f0;
    flex-shrink: 0;
  }

  .tab {
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

  .tab:hover {
    background-color: #f1f5f9;
    color: #1e293b;
  }

  .tab.active {
    background-color: #3b82f6;
    color: white;
  }

  .tab-content {
    flex: 1;
    overflow: auto;
    padding: 0.75rem;
    min-height: 0;
  }

  .logs-section {
    flex: 1;
    min-height: 0;
  }

  .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #94a3b8;
    font-size: 0.875rem;
  }
</style>
