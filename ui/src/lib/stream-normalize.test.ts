import { describe, expect, it } from 'vitest';

import { normalizeStreamEvents } from './stream-normalize';
import type { StreamEvent } from './types';
import executorFixture from './fixtures/stream-sample-executor.jsonl?raw';

function parseFixture(text: string): StreamEvent[] {
  return text
    .trim()
    .split('\n')
    .filter((line: string) => line.length > 0)
    .map((line: string) => JSON.parse(line) as StreamEvent);
}

describe('normalizeStreamEvents', () => {
  it('groups command events and ignores thread.started in semantic mode', () => {
    const events = parseFixture(executorFixture);
    const normalized = normalizeStreamEvents(events);

    expect(normalized[0]?.kind).toBe('turn_started');

    const commands = normalized.filter((event) => event.kind === 'command');
    expect(commands).toHaveLength(3);

    const messages = normalized.filter((event) => event.kind === 'agent_message');
    expect(messages).toHaveLength(1);
  });

  it('returns raw passthrough events when mode=raw', () => {
    const events = parseFixture(executorFixture);
    const normalized = normalizeStreamEvents(events, { mode: 'raw' });

    expect(normalized).toHaveLength(events.length);
    expect(normalized[0]?.kind).toBe('raw');
  });

  it('handles completed command events without a prior start', () => {
    const events: StreamEvent[] = [
      {
        type: 'item.completed',
        item: {
          id: 'item_99',
          type: 'command_execution',
          command: 'echo hello',
          aggregated_output: 'hello\n',
          exit_code: 0,
          status: 'completed',
        },
      },
    ];

    const normalized = normalizeStreamEvents(events);
    expect(normalized).toHaveLength(1);

    const command = normalized[0];
    if (command?.kind !== 'command') {
      throw new Error('Expected command event');
    }
    expect(command.status).toBe('completed');
    expect(command.output).toContain('hello');
  });

  it('can include unknown events when requested', () => {
    const events: StreamEvent[] = [{ type: 'unknown.event' }];
    const normalized = normalizeStreamEvents(events, { includeUnknown: true });

    expect(normalized).toHaveLength(1);
    expect(normalized[0]?.kind).toBe('unknown');
  });
});
