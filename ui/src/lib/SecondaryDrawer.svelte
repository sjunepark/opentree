<script lang="ts">
  import { drawer, data, setDrawerTab, toggleDrawer } from './stores.svelte';
  import AncestryTreeView from './AncestryTreeView.svelte';
  import ConfigPanel from './ConfigPanel.svelte';
  import DocsPanel from './DocsPanel.svelte';
</script>

<div class="secondary-drawer" class:open={drawer.open}>
  <div class="tab-bar">
    <button
      class="tab"
      class:active={drawer.activeTab === 'tree'}
      onclick={() => setDrawerTab('tree')}
    >
      Tree
    </button>
    <button
      class="tab"
      class:active={drawer.activeTab === 'config'}
      onclick={() => setDrawerTab('config')}
    >
      Config
    </button>
    <button
      class="tab"
      class:active={drawer.activeTab === 'docs'}
      onclick={() => setDrawerTab('docs')}
    >
      Docs
    </button>
    <button class="toggle" onclick={toggleDrawer}>
      {drawer.open ? '▼' : '▲'}
    </button>
  </div>

  {#if drawer.open}
    <div class="content">
      {#if drawer.activeTab === 'tree'}
        {#if data.tree}
          <AncestryTreeView tree={data.tree} />
        {:else}
          <div class="empty">No tree loaded</div>
        {/if}
      {:else if drawer.activeTab === 'config'}
        {#if data.config}
          <ConfigPanel config={data.config} />
        {:else}
          <div class="empty">No config loaded</div>
        {/if}
      {:else if drawer.activeTab === 'docs'}
        <DocsPanel assumptions={data.assumptions} questions={data.questions} />
      {/if}
    </div>
  {/if}
</div>

<style>
  .secondary-drawer {
    background: white;
    border-top: 1px solid #e2e8f0;
    transition: max-height 0.2s ease;
    max-height: 2.5rem;
    overflow: hidden;
  }

  .secondary-drawer.open {
    max-height: 20rem;
  }

  .tab-bar {
    display: flex;
    align-items: center;
    padding: 0 0.5rem;
    height: 2.5rem;
    border-bottom: 1px solid #e2e8f0;
  }

  .tab {
    padding: 0.375rem 0.75rem;
    font-size: 0.75rem;
    font-weight: 500;
    color: #64748b;
    background: none;
    border: none;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .tab:hover {
    color: #1e293b;
  }

  .tab.active {
    color: #3b82f6;
  }

  .toggle {
    margin-left: auto;
    padding: 0.25rem 0.5rem;
    font-size: 0.625rem;
    color: #64748b;
    background: none;
    border: none;
    cursor: pointer;
  }

  .toggle:hover {
    color: #1e293b;
  }

  .content {
    height: calc(20rem - 2.5rem);
    overflow: auto;
    padding: 0.75rem;
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
