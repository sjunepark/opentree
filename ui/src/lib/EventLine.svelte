<script lang="ts">
  import type { StreamEvent } from './types';

  interface Props {
    event: StreamEvent;
    index: number;
  }

  let { event, index }: Props = $props();
  let expanded = $state(true);
  let showRaw = $state(false);

  // Get event type with color coding
  function getEventTypeClass(type: string | undefined): string {
    switch (type) {
      case 'agent_start':
      case 'agent_end':
        return 'type-agent';
      case 'tool_use':
        return 'type-tool';
      case 'tool_result':
        return 'type-result';
      case 'thinking':
        return 'type-thinking';
      case 'text':
        return 'type-text';
      case 'error':
        return 'type-error';
      default:
        return 'type-default';
    }
  }

  // Format event content for display
  function formatContent(event: StreamEvent): string {
    const type = event.type || 'unknown';

    switch (type) {
      case 'agent_start':
        return `Starting work on ${event.node_id || 'task'}`;
      case 'agent_end':
        return `Finished: ${event.status || 'unknown'}`;
      case 'tool_use':
        return formatToolUse(event);
      case 'tool_result':
        return formatToolResult(event);
      case 'thinking':
        return String(event.content || '');
      case 'text':
        return String(event.content || '');
      case 'error':
        return String(event.message || event.error || 'Unknown error');
      default:
        // For unknown types, show a compact JSON representation
        const { type: _, ...rest } = event;
        return Object.keys(rest).length > 0
          ? JSON.stringify(rest)
          : '';
    }
  }

  function formatToolUse(event: StreamEvent): string {
    const tool = String(event.tool || event.name || 'unknown');
    const input = event.input || event.arguments;

    if (tool === 'Read' && input && typeof input === 'object') {
      return `Read ${(input as Record<string, unknown>).file_path || ''}`;
    }
    if (tool === 'Edit' && input && typeof input === 'object') {
      const inp = input as Record<string, unknown>;
      return `Edit ${inp.file_path || ''}`;
    }
    if (tool === 'Write' && input && typeof input === 'object') {
      return `Write ${(input as Record<string, unknown>).file_path || ''}`;
    }
    if (tool === 'Bash' && input && typeof input === 'object') {
      const cmd = String((input as Record<string, unknown>).command || '');
      return `Bash: ${cmd}`;
    }
    return tool;
  }

  function formatToolResult(event: StreamEvent): string {
    const result = event.result || event.output;
    if (typeof result === 'string') {
      return result;
    }
    if (result && typeof result === 'object') {
      return JSON.stringify(result);
    }
    return 'Done';
  }

  function buildPreview(text: string, maxChars: number, maxLines: number) {
    const lines = text.split('\n');
    let truncated = false;
    let previewLines = lines;

    if (lines.length > maxLines) {
      previewLines = lines.slice(0, maxLines);
      truncated = true;
    }

    let preview = previewLines.join('\n');
    if (preview.length > maxChars) {
      preview = preview.slice(0, maxChars);
      truncated = true;
    }

    if (truncated) {
      preview = preview.trimEnd() + '…';
    }

    return { preview, truncated };
  }

  async function copyText(value: string) {
    if (!value) return;
    try {
      await navigator.clipboard.writeText(value);
    } catch {
      // no-op: clipboard may be unavailable
    }
  }

  function downloadText(filename: string, value: string) {
    if (!value) return;
    const blob = new Blob([value], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    link.click();
    URL.revokeObjectURL(url);
  }

  const content = $derived(formatContent(event));
  const rawJson = $derived(JSON.stringify(event, null, 2));
  const previewState = $derived(buildPreview(content, 180, 2));
  const preview = $derived(previewState.preview);
  const isTruncated = $derived(previewState.truncated);
</script>

<div class="event-line">
  <span class="index">{index + 1}</span>
  <span class="type {getEventTypeClass(event.type)}">[{event.type || 'event'}]</span>
  <div class="content">
    <pre class="content-text">{expanded ? content : preview}</pre>
    {#if isTruncated}
      <button class="toggle" onclick={() => (expanded = !expanded)}>
        {expanded ? 'Show less' : 'Show more'}
      </button>
    {/if}
    <div class="actions">
      <button class="action" onclick={() => copyText(content)}>Copy text</button>
      <button class="action" onclick={() => copyText(rawJson)}>Copy JSON</button>
      <button
        class="action"
        onclick={() => downloadText(`event-${index + 1}.json`, rawJson)}
      >
        Download JSON
      </button>
      <button class="action" onclick={() => (showRaw = !showRaw)}>
        {showRaw ? 'Hide JSON' : 'Show JSON'}
      </button>
    </div>
    {#if showRaw}
      <pre class="raw-json">{rawJson}</pre>
    {/if}
  </div>
</div>

<style>
  .event-line {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    padding: 0.25rem 0.5rem;
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

  .type {
    flex-shrink: 0;
    font-weight: 500;
  }

  .type-agent {
    color: #a78bfa;
  }

  .type-tool {
    color: #60a5fa;
  }

  .type-result {
    color: #34d399;
  }

  .type-thinking {
    color: #fbbf24;
  }

  .type-text {
    color: #e2e8f0;
  }

  .type-error {
    color: #f87171;
  }

  .type-default {
    color: #94a3b8;
  }

  .content {
    flex: 1;
    color: #cbd5e1;
    word-break: break-word;
    white-space: pre-wrap;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .content-text {
    margin: 0;
    font-family: inherit;
    white-space: pre-wrap;
  }

  .toggle {
    align-self: flex-start;
    font-size: 0.625rem;
    color: #93c5fd;
    background: transparent;
    border: 1px solid transparent;
    padding: 0;
    cursor: pointer;
  }

  .toggle:hover {
    text-decoration: underline;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }

  .action {
    font-size: 0.625rem;
    color: #e2e8f0;
    background-color: rgba(148, 163, 184, 0.1);
    border: 1px solid rgba(148, 163, 184, 0.2);
    border-radius: 0.25rem;
    padding: 0.125rem 0.375rem;
    cursor: pointer;
  }

  .action:hover {
    background-color: rgba(148, 163, 184, 0.2);
  }

  .raw-json {
    margin: 0.25rem 0 0;
    padding: 0.5rem;
    background-color: rgba(15, 23, 42, 0.6);
    border: 1px solid rgba(148, 163, 184, 0.2);
    border-radius: 0.25rem;
    font-family: ui-monospace, monospace;
    font-size: 0.7rem;
    white-space: pre-wrap;
  }
</style>
