<script lang="ts">
  import HistoryTimeline from './HistoryTimeline.svelte';
  import ContextPanel from './ContextPanel.svelte';
</script>

<div class="bottom-section">
  <div class="timeline-pane">
    <HistoryTimeline />
  </div>
  <div class="context-pane">
    <ContextPanel />
  </div>
</div>

<style>
  .bottom-section {
    display: grid;
    grid-template-columns: minmax(200px, 1fr) 2fr;
    gap: 0.75rem;
    height: 100%;
  }

  .timeline-pane {
    min-width: 0;
  }

  .context-pane {
    min-width: 0;
  }
</style>
