<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import TreeView from './lib/TreeView.svelte';
  import NodeDetail from './lib/NodeDetail.svelte';
  import IterationList from './lib/IterationList.svelte';
  import IterationDetail from './lib/IterationDetail.svelte';
  import ConnectionStatus from './lib/ConnectionStatus.svelte';
  import ConfigPanel from './lib/ConfigPanel.svelte';
  import DocsPanel from './lib/DocsPanel.svelte';
  import { tree, runState, runs, selectedNode, selectedIteration, loading, error, config, assumptions, questions, activeTab, type ActiveTab } from './lib/stores';
  import { fetchTree, fetchRunState, fetchIterations, fetchConfig, fetchAssumptions, fetchQuestions } from './lib/api';
  import { connect, disconnect, subscribe } from './lib/sse';

  // Load initial data
  async function loadData() {
    loading.set(true);
    error.set(null);

    try {
      const [treeData, runStateData, iterationsData, configData, assumptionsData, questionsData] = await Promise.all([
        fetchTree().catch(() => null),
        fetchRunState().catch(() => null),
        fetchIterations().catch(() => ({ runs: [] })),
        fetchConfig().catch(() => null),
        fetchAssumptions().catch(() => ''),
        fetchQuestions().catch(() => ''),
      ]);

      if (treeData) tree.set(treeData);
      if (runStateData) runState.set(runStateData);
      runs.set(iterationsData.runs);
      if (configData) config.set(configData);
      assumptions.set(assumptionsData);
      questions.set(questionsData);
    } catch (e) {
      error.set(e instanceof Error ? e.message : 'Failed to load data');
    } finally {
      loading.set(false);
    }
  }

  // Handle SSE events
  function handleChange(event: { type: string; run_id?: string; iter?: number }) {
    switch (event.type) {
      case 'tree_changed':
        fetchTree()
          .then((data) => tree.set(data))
          .catch(console.error);
        break;
      case 'run_state_changed':
        fetchRunState()
          .then((data) => runState.set(data))
          .catch(console.error);
        break;
      case 'iteration_added':
        fetchIterations()
          .then((data) => runs.set(data.runs))
          .catch(console.error);
        break;
      case 'config_changed':
        fetchConfig()
          .then((data) => config.set(data))
          .catch(console.error);
        break;
      case 'assumptions_changed':
        fetchAssumptions()
          .then((data) => assumptions.set(data))
          .catch(console.error);
        break;
      case 'questions_changed':
        fetchQuestions()
          .then((data) => questions.set(data))
          .catch(console.error);
        break;
    }
  }

  let unsubscribe: (() => void) | null = null;

  function setTab(tab: ActiveTab) {
    activeTab.set(tab);
  }

  // When selecting a node or iteration, switch to appropriate tab
  $effect(() => {
    if ($selectedIteration) {
      activeTab.set('iteration');
    }
  });

  $effect(() => {
    if ($selectedNode && !$selectedIteration) {
      activeTab.set('node');
    }
  });

  onMount(() => {
    loadData();
    connect();
    unsubscribe = subscribe(handleChange);
  });

  onDestroy(() => {
    disconnect();
    if (unsubscribe) unsubscribe();
  });
</script>

<div class="app">
  <header class="header">
    <h1 class="title">Runner UI</h1>
    <div class="header-right">
      {#if $runState?.run_id}
        <span class="run-info">Run: {$runState.run_id}</span>
      {/if}
      <ConnectionStatus />
    </div>
  </header>

  <main class="main">
    {#if $loading}
      <div class="loading-overlay">
        <span>Loading...</span>
      </div>
    {:else if $error}
      <div class="error-overlay">
        <p>Error: {$error}</p>
        <button onclick={loadData}>Retry</button>
      </div>
    {:else}
      <div class="panels">
        <!-- Left panel: Tree view -->
        <section class="panel tree-panel">
          <div class="panel-header">Task Tree</div>
          <div class="panel-body">
            {#if $tree}
              <TreeView tree={$tree} />
            {:else}
              <div class="empty-state">No tree loaded</div>
            {/if}
          </div>
        </section>

        <!-- Center panel: Tabbed detail view -->
        <section class="panel detail-panel">
          <div class="panel-header with-tabs">
            <div class="tabs">
              <button
                class="tab"
                class:active={$activeTab === 'node'}
                onclick={() => setTab('node')}
              >
                Node
              </button>
              <button
                class="tab"
                class:active={$activeTab === 'iteration'}
                onclick={() => setTab('iteration')}
              >
                Iteration
              </button>
              <button
                class="tab"
                class:active={$activeTab === 'config'}
                onclick={() => setTab('config')}
              >
                Config
              </button>
              <button
                class="tab"
                class:active={$activeTab === 'docs'}
                onclick={() => setTab('docs')}
              >
                Docs
              </button>
            </div>
          </div>
          <div class="panel-body">
            {#if $activeTab === 'node'}
              {#if $selectedNode}
                <NodeDetail node={$selectedNode} />
              {:else}
                <div class="empty-state">Select a node from the tree</div>
              {/if}
            {:else if $activeTab === 'iteration'}
              {#if $selectedIteration}
                <IterationDetail runId={$selectedIteration.run_id} iter={$selectedIteration.iter} />
              {:else}
                <div class="empty-state">Select an iteration from the list</div>
              {/if}
            {:else if $activeTab === 'config'}
              {#if $config}
                <ConfigPanel config={$config} />
              {:else}
                <div class="empty-state">No config loaded</div>
              {/if}
            {:else if $activeTab === 'docs'}
              <DocsPanel assumptions={$assumptions} questions={$questions} />
            {/if}
          </div>
        </section>

        <!-- Right panel: Iterations -->
        <section class="panel iterations-panel">
          <div class="panel-header">Iterations</div>
          <div class="panel-body">
            <IterationList runs={$runs} />
          </div>
        </section>
      </div>
    {/if}
  </main>
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background-color: #f8fafc;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background-color: #ffffff;
    border-bottom: 1px solid #e2e8f0;
    flex-shrink: 0;
  }

  .title {
    font-size: 1.125rem;
    font-weight: 600;
    color: #1e293b;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .run-info {
    font-family: ui-monospace, monospace;
    font-size: 0.75rem;
    color: #64748b;
  }

  .main {
    flex: 1;
    overflow: hidden;
    padding: 1rem;
  }

  .loading-overlay,
  .error-overlay {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 1rem;
    color: #64748b;
  }

  .error-overlay {
    color: #dc2626;
  }

  .error-overlay button {
    padding: 0.5rem 1rem;
    background-color: #3b82f6;
    color: white;
    border: none;
    border-radius: 0.375rem;
    cursor: pointer;
  }

  .error-overlay button:hover {
    background-color: #2563eb;
  }

  .panels {
    display: grid;
    grid-template-columns: 1fr 1.5fr 1fr;
    gap: 1rem;
    height: 100%;
  }

  .panel {
    display: flex;
    flex-direction: column;
    background: #ffffff;
    border: 1px solid #e2e8f0;
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .panel-header {
    padding: 0.75rem 1rem;
    border-bottom: 1px solid #e2e8f0;
    font-weight: 600;
    font-size: 0.875rem;
    color: #64748b;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    flex-shrink: 0;
  }

  .panel-body {
    flex: 1;
    overflow: auto;
    padding: 0.5rem;
  }

  .detail-panel .panel-body {
    padding: 1rem;
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #94a3b8;
    font-size: 0.875rem;
  }

  .panel-header.with-tabs {
    padding: 0;
  }

  .tabs {
    display: flex;
    gap: 0;
  }

  .tab {
    padding: 0.75rem 1rem;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #64748b;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .tab:hover {
    color: #1e293b;
    background-color: #f8fafc;
  }

  .tab.active {
    color: #3b82f6;
    border-bottom-color: #3b82f6;
  }
</style>
