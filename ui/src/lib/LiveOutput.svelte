<script lang="ts">
  import { stream } from './stores.svelte';
  import EventLine from './EventLine.svelte';

  let container: HTMLDivElement;
  let userScrolled = $state(false);

  // Auto-scroll when new events arrive (if user hasn't scrolled up)
  $effect(() => {
    const eventCount = stream.events.length;
    if (eventCount > 0 && stream.autoScroll && container) {
      // Use requestAnimationFrame to ensure DOM is updated
      requestAnimationFrame(() => {
        container.scrollTop = container.scrollHeight;
      });
    }
  });

  function handleScroll() {
    if (!container) return;

    // Check if user scrolled away from bottom
    const isAtBottom = container.scrollHeight - container.scrollTop - container.clientHeight < 50;

    if (isAtBottom) {
      stream.autoScroll = true;
      userScrolled = false;
    } else {
      stream.autoScroll = false;
      userScrolled = true;
    }
  }

  function scrollToBottom() {
    if (container) {
      container.scrollTop = container.scrollHeight;
      stream.autoScroll = true;
      userScrolled = false;
    }
  }
</script>

<div class="live-output">
  <div class="header">
    <span class="title">Live Output</span>
    {#if stream.activeRunId && stream.activeIter !== null}
      <span class="context">
        {stream.activeRunId} #{stream.activeIter}
      </span>
    {/if}
    <span class="event-count">{stream.events.length} events</span>
    {#if userScrolled}
      <button class="scroll-btn" onclick={scrollToBottom}>
        ↓ Scroll to bottom
      </button>
    {/if}
  </div>

  <div class="events" bind:this={container} onscroll={handleScroll}>
    {#if stream.events.length === 0}
      <div class="empty">
        {#if stream.activeIter !== null}
          Waiting for events...
        {:else}
          Select an iteration to view events
        {/if}
      </div>
    {:else}
      {#each stream.events as event, i (i)}
        <EventLine {event} index={i} />
      {/each}
      {#if stream.autoScroll}
        <div class="cursor">█</div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .live-output {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: #0f172a;
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 0.75rem;
    background-color: #1e293b;
    border-bottom: 1px solid #334155;
    flex-shrink: 0;
  }

  .title {
    font-size: 0.75rem;
    font-weight: 600;
    color: #94a3b8;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .context {
    font-family: ui-monospace, monospace;
    font-size: 0.75rem;
    color: #60a5fa;
  }

  .event-count {
    font-family: ui-monospace, monospace;
    font-size: 0.625rem;
    color: #64748b;
    margin-left: auto;
  }

  .scroll-btn {
    font-size: 0.625rem;
    padding: 0.25rem 0.5rem;
    background-color: #3b82f6;
    color: white;
    border: none;
    border-radius: 0.25rem;
    cursor: pointer;
  }

  .scroll-btn:hover {
    background-color: #2563eb;
  }

  .events {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 0.25rem 0;
  }

  .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #475569;
    font-size: 0.875rem;
  }

  .cursor {
    padding-left: 3.5rem;
    color: #4ade80;
    animation: blink 1s step-end infinite;
  }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0; }
  }
</style>
