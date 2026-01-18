<script lang="ts">
  import type { StreamEvent } from './types';

  interface Props {
    event: StreamEvent;
    index: number;
  }

  let { event, index }: Props = $props();

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
        return truncate(String(event.content || ''), 100);
      case 'text':
        return truncate(String(event.content || ''), 100);
      case 'error':
        return String(event.message || event.error || 'Unknown error');
      default:
        // For unknown types, show a compact JSON representation
        const { type: _, ...rest } = event;
        return Object.keys(rest).length > 0
          ? truncate(JSON.stringify(rest), 80)
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
      return `Bash: ${truncate(cmd, 60)}`;
    }
    return tool;
  }

  function formatToolResult(event: StreamEvent): string {
    const result = event.result || event.output;
    if (typeof result === 'string') {
      return truncate(result, 80);
    }
    if (result && typeof result === 'object') {
      return truncate(JSON.stringify(result), 80);
    }
    return 'Done';
  }

  function truncate(text: string, maxLen: number): string {
    if (text.length <= maxLen) return text;
    return text.slice(0, maxLen) + '…';
  }
</script>

<div class="event-line">
  <span class="index">{index + 1}</span>
  <span class="type {getEventTypeClass(event.type)}">[{event.type || 'event'}]</span>
  <span class="content">{formatContent(event)}</span>
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
  }
</style>
