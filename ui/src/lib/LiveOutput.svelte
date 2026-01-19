<script lang="ts">
  import { SvelteSet } from 'svelte/reactivity';
  import { stream } from './stores.svelte';
  import EventLine from './EventLine.svelte';
  import SemanticEventLine from './SemanticEventLine.svelte';
  import { normalizeStreamEvents } from './stream-normalize';
  import type { UiStreamEvent } from './types';
  import type { StreamEvent } from './types';

  let container: HTMLDivElement;
  let userScrolled = $state(false);
  let viewMode = $state<'semantic' | 'raw'>('semantic');
  let searchQuery = $state('');
  let errorsOnly = $state(false);
  const typeFilter = new SvelteSet<string>();
  let errorCursor = $state(-1);
  let viewportHeight = $state(0);
  let scrollTop = $state(0);

  type DisplayEntry =
    | { key: string; kind: 'semantic'; event: UiStreamEvent }
    | { key: string; kind: 'raw'; event: StreamEvent; rawIndex: number };

  function extractFirstBold(text: string): string | null {
    const match = text.match(/\*\*(.+?)\*\*/);
    return match ? match[1].trim() : null;
  }

  function latestReasoningHeader(events: UiStreamEvent[]): string | null {
    for (let i = events.length - 1; i >= 0; i -= 1) {
      const event = events[i];
      if (event.kind === 'reasoning') {
        return extractFirstBold(event.text) ?? 'Reasoning';
      }
    }
    return null;
  }

  const semanticEvents = $derived(normalizeStreamEvents(stream.events));
  const statusHeader = $derived(latestReasoningHeader(semanticEvents));
  const query = $derived(searchQuery.trim().toLowerCase());

  function toggleType(type: string) {
    if (typeFilter.has(type)) {
      typeFilter.delete(type);
    } else {
      typeFilter.add(type);
    }
  }

  function clearFilters() {
    searchQuery = '';
    errorsOnly = false;
    typeFilter.clear();
  }

  function eventTextFromSemantic(event: UiStreamEvent): string {
    switch (event.kind) {
      case 'command':
        return `${event.command}\n${event.output}`;
      case 'reasoning':
        return event.text;
      case 'agent_message':
        return event.text;
      default:
        return '';
    }
  }

  function eventTextFromRaw(event: StreamEvent): string {
    try {
      return JSON.stringify(event);
    } catch {
      return '';
    }
  }

  function isSemanticError(event: UiStreamEvent): boolean {
    if (event.kind === 'command') {
      return event.exitCode !== null && event.exitCode !== 0;
    }
    if (event.kind === 'agent_message') {
      return event.text.toLowerCase().includes('error');
    }
    return false;
  }

  function isRawError(event: StreamEvent): boolean {
    if (event.type === 'error') return true;
    const item = (event as Record<string, unknown>).item as
      | { exit_code?: unknown; type?: unknown }
      | undefined;
    if (item && item.type === 'command_execution') {
      return typeof item.exit_code === 'number' && item.exit_code !== 0;
    }
    return false;
  }

  const availableTypes = $derived.by(() => {
    const types = new SvelteSet<string>();
    if (viewMode === 'semantic') {
      semanticEvents.forEach((event) => types.add(event.kind));
    } else {
      stream.events.forEach((event) => {
        const rawType = typeof event.type === 'string' ? event.type : 'event';
        types.add(rawType);
      });
    }
    return Array.from(types).sort();
  });

  const filteredSemanticEvents = $derived(
    semanticEvents.filter((event) => {
      const type = event.kind;
      if (typeFilter.size > 0 && !typeFilter.has(type)) return false;
      if (errorsOnly && !isSemanticError(event)) return false;
      if (query && !eventTextFromSemantic(event).toLowerCase().includes(query)) return false;
      return true;
    })
  );

  const filteredRawEvents = $derived(
    stream.events
      .map((event, index) => ({ event, index }))
      .filter(({ event }) => {
        const type = typeof event.type === 'string' ? event.type : 'event';
        if (typeFilter.size > 0 && !typeFilter.has(type)) return false;
        if (errorsOnly && !isRawError(event)) return false;
        if (query && !eventTextFromRaw(event).toLowerCase().includes(query)) return false;
        return true;
      })
  );

  const errorIds = $derived.by(() => {
    if (viewMode === 'semantic') {
      return filteredSemanticEvents
        .filter((event) => isSemanticError(event))
        .map((event) => event.id);
    }
    return filteredRawEvents
      .filter(({ event }) => isRawError(event))
      .map(({ index }) => `raw:${index}`);
  });

  $effect(() => {
    errorIds;
    errorCursor = -1;
  });

  $effect(() => {
    if (container) {
      viewportHeight = container.clientHeight;
    }
  });

  const displayEvents = $derived.by(() => {
    if (viewMode === 'semantic') {
      return filteredSemanticEvents.map((event) => ({
        key: event.id,
        kind: 'semantic' as const,
        event,
      }));
    }
    return filteredRawEvents.map(({ event, index }) => ({
      key: `raw:${index}`,
      kind: 'raw' as const,
      event,
      rawIndex: index,
    }));
  });

  const totalEvents = $derived(displayEvents.length);
  const useWindowing = $derived(totalEvents > 400);
  const estimatedRowHeight = 28;
  const overscan = 10;
  const visibleCount = $derived(
    viewportHeight > 0 ? Math.ceil(viewportHeight / estimatedRowHeight) : 20
  );
  const windowStart = $derived(
    useWindowing
      ? Math.max(0, Math.floor(scrollTop / estimatedRowHeight) - overscan)
      : 0
  );
  const windowEnd = $derived(
    useWindowing
      ? Math.min(totalEvents, windowStart + visibleCount + overscan * 2)
      : totalEvents
  );
  const windowedEvents = $derived(displayEvents.slice(windowStart, windowEnd));
  const topSpacer = $derived(useWindowing ? windowStart * estimatedRowHeight : 0);
  const bottomSpacer = $derived(
    useWindowing ? Math.max(0, (totalEvents - windowEnd) * estimatedRowHeight) : 0
  );

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
    scrollTop = container.scrollTop;
    viewportHeight = container.clientHeight;

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
      scrollTop = container.scrollTop;
      viewportHeight = container.clientHeight;
      stream.autoScroll = true;
      userScrolled = false;
    }
  }

  function toggleAutoScroll() {
    if (stream.autoScroll) {
      stream.autoScroll = false;
      userScrolled = true;
    } else {
      stream.autoScroll = true;
      userScrolled = false;
      requestAnimationFrame(() => scrollToBottom());
    }
  }

  function jumpError(direction: 1 | -1) {
    if (!container || errorIds.length === 0) return;
    let nextIndex = errorCursor + direction;
    if (nextIndex < 0) nextIndex = errorIds.length - 1;
    if (nextIndex >= errorIds.length) nextIndex = 0;
    errorCursor = nextIndex;
    const target = container.querySelector<HTMLElement>(
      `[data-event-id=\"${errorIds[nextIndex]}\"]`
    );
    if (target) {
      target.scrollIntoView({ block: 'center' });
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
    {#if statusHeader && viewMode === 'semantic'}
      <span class="status-header">{statusHeader}</span>
    {/if}
    <span class="event-count">
      {viewMode === 'semantic' ? filteredSemanticEvents.length : filteredRawEvents.length}
      /{viewMode === 'semantic' ? semanticEvents.length : stream.events.length}
    </span>
    <div class="view-toggle">
      <button
        class="toggle-btn {viewMode === 'semantic' ? 'active' : ''}"
        onclick={() => (viewMode = 'semantic')}
      >
        Semantic
      </button>
      <button
        class="toggle-btn {viewMode === 'raw' ? 'active' : ''}"
        onclick={() => (viewMode = 'raw')}
      >
        Raw
      </button>
    </div>
    {#if userScrolled}
      <button class="scroll-btn" onclick={scrollToBottom}>
        ↓ Scroll to bottom
      </button>
    {/if}
  </div>

  <div class="toolbar">
    <div class="toolbar-section">
      <input
        class="search"
        type="search"
        placeholder="Search logs"
        bind:value={searchQuery}
      />
      <button class="tool-btn" onclick={() => (errorsOnly = !errorsOnly)}>
        {errorsOnly ? 'Errors only' : 'All events'}
      </button>
      <button class="tool-btn" onclick={toggleAutoScroll}>
        {stream.autoScroll ? 'Auto-scroll on' : 'Auto-scroll off'}
      </button>
      <button class="tool-btn" onclick={() => jumpError(-1)}>Prev error</button>
      <button class="tool-btn" onclick={() => jumpError(1)}>Next error</button>
      <button class="tool-btn" onclick={clearFilters}>Clear</button>
    </div>
    <div class="toolbar-section types">
      {#each availableTypes as type (type)}
        <button
          class="type-chip {typeFilter.has(type) ? 'active' : ''}"
          onclick={() => toggleType(type)}
        >
          {type}
        </button>
      {/each}
    </div>
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
      {#if useWindowing}
        <div class="spacer" style={`height: ${topSpacer}px`}></div>
      {/if}
      {#each windowedEvents as entry, i (entry.key)}
        {#if entry.kind === 'semantic'}
          <div class="event-row" data-event-id={entry.key}>
            <SemanticEventLine event={entry.event} index={windowStart + i} />
          </div>
        {:else}
          <div class="event-row" data-event-id={entry.key}>
            <EventLine event={entry.event} index={windowStart + i} />
          </div>
        {/if}
      {/each}
      {#if useWindowing}
        <div class="spacer" style={`height: ${bottomSpacer}px`}></div>
      {/if}
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

  .status-header {
    font-size: 0.7rem;
    color: #fbbf24;
    margin-left: 0.25rem;
  }

  .view-toggle {
    display: inline-flex;
    border: 1px solid #334155;
    border-radius: 0.375rem;
    overflow: hidden;
  }

  .toggle-btn {
    font-size: 0.625rem;
    padding: 0.25rem 0.5rem;
    background-color: transparent;
    color: #94a3b8;
    border: none;
    cursor: pointer;
  }

  .toggle-btn.active {
    background-color: #1d4ed8;
    color: #f8fafc;
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

  .toolbar {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid #334155;
    background-color: #0b1220;
  }

  .toolbar-section {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: center;
  }

  .search {
    flex: 1 1 12rem;
    min-width: 10rem;
    padding: 0.35rem 0.5rem;
    border-radius: 0.375rem;
    border: 1px solid #334155;
    background-color: #0f172a;
    color: #e2e8f0;
    font-size: 0.75rem;
  }

  .tool-btn {
    font-size: 0.625rem;
    padding: 0.25rem 0.5rem;
    border-radius: 0.375rem;
    border: 1px solid #334155;
    background-color: #111827;
    color: #cbd5f5;
    cursor: pointer;
  }

  .tool-btn:hover {
    background-color: #1f2937;
  }

  .types {
    gap: 0.35rem;
  }

  .type-chip {
    font-size: 0.6rem;
    padding: 0.2rem 0.45rem;
    border-radius: 999px;
    border: 1px solid #334155;
    background-color: transparent;
    color: #94a3b8;
    cursor: pointer;
  }

  .type-chip.active {
    background-color: #1d4ed8;
    border-color: #1d4ed8;
    color: #f8fafc;
  }

  .events {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 0.25rem 0;
  }

  .spacer {
    width: 100%;
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
