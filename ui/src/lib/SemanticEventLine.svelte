<script lang="ts">
  import type { UiStreamEvent } from './types';

  interface Props {
    event: UiStreamEvent;
    index: number;
  }

  let { event, index }: Props = $props();

  function extractFirstBold(text: string): string | null {
    const match = text.match(/\*\*(.+?)\*\*/);
    return match ? match[1].trim() : null;
  }

  function formatAgentMessage(text: string): string {
    return text.trim();
  }
</script>

<div class="event-line">
  <span class="index">{index + 1}</span>
  <div class="body">
    {#if event.kind === 'turn_started'}
      <div class="meta">
        <span class="badge">Turn</span>
        <span class="status">started</span>
      </div>
    {:else if event.kind === 'turn_completed'}
      <div class="meta">
        <span class="badge">Turn</span>
        <span class="status">completed</span>
        {#if event.usage}
          <span class="usage">
            in {event.usage.input_tokens ?? 0} | out {event.usage.output_tokens ?? 0}
          </span>
        {/if}
      </div>
    {:else if event.kind === 'command'}
      <div class="meta">
        <span class="badge">Command</span>
        <span class="status {event.status}">{event.status}</span>
        {#if event.exitCode !== null}
          <span class="exit">exit {event.exitCode}</span>
        {/if}
      </div>
      <pre class="command">{event.command}</pre>
      {#if event.output}
        <details class="output" open={event.status === 'running'}>
          <summary>Output</summary>
          <pre>{event.output}</pre>
        </details>
      {/if}
    {:else if event.kind === 'reasoning'}
      <details class="reasoning">
        <summary>{extractFirstBold(event.text) ?? 'Reasoning'}</summary>
        <pre>{event.text}</pre>
      </details>
    {:else if event.kind === 'agent_message'}
      <div class="meta">
        <span class="badge">Message</span>
      </div>
      <pre class="message">{formatAgentMessage(event.text)}</pre>
    {:else}
      <div class="meta">
        <span class="badge">Event</span>
        <span class="status">{event.kind}</span>
      </div>
      <pre class="message">{JSON.stringify(event.raw, null, 2)}</pre>
    {/if}
  </div>
</div>

<style>
  .event-line {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    font-family: ui-monospace, monospace;
    font-size: 0.75rem;
    line-height: 1.4;
  }

  .event-line:nth-child(odd) {
    background-color: rgba(255, 255, 255, 0.02);
  }

  .index {
    flex-shrink: 0;
    width: 2rem;
    text-align: right;
    color: #475569;
  }

  .body {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    color: #cbd5e1;
  }

  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    align-items: center;
  }

  .badge {
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-size: 0.6rem;
    color: #93c5fd;
  }

  .status {
    font-size: 0.65rem;
    color: #e2e8f0;
  }

  .status.running {
    color: #fbbf24;
  }

  .status.completed {
    color: #34d399;
  }

  .usage,
  .exit {
    font-size: 0.65rem;
    color: #94a3b8;
  }

  .command,
  .message {
    margin: 0;
    white-space: pre-wrap;
  }

  .output {
    background-color: rgba(15, 23, 42, 0.5);
    border: 1px solid rgba(148, 163, 184, 0.2);
    border-radius: 0.25rem;
    padding: 0.35rem 0.5rem;
  }

  .output summary {
    cursor: pointer;
    color: #93c5fd;
    margin-bottom: 0.25rem;
  }

  .output pre {
    margin: 0;
    white-space: pre-wrap;
  }

  .reasoning {
    background-color: rgba(30, 41, 59, 0.35);
    border: 1px solid rgba(148, 163, 184, 0.2);
    border-radius: 0.25rem;
    padding: 0.35rem 0.5rem;
  }

  .reasoning summary {
    cursor: pointer;
    color: #fbbf24;
    margin-bottom: 0.25rem;
  }

  .reasoning pre {
    margin: 0;
    white-space: pre-wrap;
  }
</style>
