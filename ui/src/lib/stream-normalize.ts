import type { StreamEvent, UiStreamEvent, NormalizeMode, StreamUsage } from './types';

type NormalizeOptions = {
  mode?: NormalizeMode;
  includeUnknown?: boolean;
};

type RawItem = {
  id?: unknown;
  type?: unknown;
  text?: unknown;
  command?: unknown;
  aggregated_output?: unknown;
  exit_code?: unknown;
  status?: unknown;
};

function asString(value: unknown): string {
  return typeof value === 'string' ? value : '';
}

function asNumberOrNull(value: unknown): number | null {
  return typeof value === 'number' ? value : null;
}

function asUsage(value: unknown): StreamUsage | undefined {
  if (!value || typeof value !== 'object') return undefined;
  return value as StreamUsage;
}

export function normalizeStreamEvents(
  rawEvents: StreamEvent[],
  options: NormalizeOptions = {}
): UiStreamEvent[] {
  const mode = options.mode ?? 'semantic';

  if (mode === 'raw') {
    return rawEvents.map((event, index) => ({
      kind: 'raw',
      id: `raw:${index}`,
      raw: event,
    }));
  }

  const out: UiStreamEvent[] = [];
  const commandIndexByItemId = new Map<string, number>();

  rawEvents.forEach((event, index) => {
    const type = typeof event.type === 'string' ? event.type : '';

    if (type === 'thread.started') {
      return;
    }

    if (type === 'turn.started') {
      out.push({
        kind: 'turn_started',
        id: `turn:${index}`,
        raw: event,
      });
      return;
    }

    if (type === 'turn.completed') {
      out.push({
        kind: 'turn_completed',
        id: `turn:${index}`,
        usage: asUsage((event as Record<string, unknown>).usage),
        raw: event,
      });
      return;
    }

    if (type === 'item.started' || type === 'item.completed') {
      const item = (event as Record<string, unknown>).item as RawItem | undefined;
      if (!item || typeof item !== 'object') {
        if (options.includeUnknown) {
          out.push({
            kind: 'unknown',
            id: `unknown:${index}`,
            raw: event,
          });
        }
        return;
      }

      const itemType = typeof item.type === 'string' ? item.type : '';
      const itemId = typeof item.id === 'string' ? item.id : '';

      if (itemType === 'command_execution') {
        const command = asString(item.command);
        const output = asString(item.aggregated_output);
        const exitCode = asNumberOrNull(item.exit_code);
        const status = type === 'item.started' ? 'running' : 'completed';

        if (itemId && commandIndexByItemId.has(itemId)) {
          const existingIndex = commandIndexByItemId.get(itemId);
          if (existingIndex !== undefined) {
            const existing = out[existingIndex];
            if (existing.kind === 'command') {
              if (command) {
                existing.command = command;
              }
              if (type === 'item.completed') {
                existing.status = 'completed';
                existing.output = output;
                existing.exitCode = exitCode;
                existing.rawEnd = event;
              } else if (!existing.rawStart) {
                existing.rawStart = event;
              }
            }
          }
          return;
        }

        const id = itemId ? `command:${itemId}` : `command:${index}`;
        const commandEvent: UiStreamEvent = {
          kind: 'command',
          id,
          status,
          command,
          output,
          exitCode,
          rawStart: type === 'item.started' ? event : undefined,
          rawEnd: type === 'item.completed' ? event : undefined,
        };
        out.push(commandEvent);
        if (itemId) {
          commandIndexByItemId.set(itemId, out.length - 1);
        }
        return;
      }

      if (itemType === 'reasoning') {
        const text = asString(item.text);
        if (text) {
          out.push({
            kind: 'reasoning',
            id: itemId ? `reasoning:${itemId}` : `reasoning:${index}`,
            text,
            raw: event,
          });
        }
        return;
      }

      if (itemType === 'agent_message') {
        const text = asString(item.text);
        if (text) {
          out.push({
            kind: 'agent_message',
            id: itemId ? `message:${itemId}` : `message:${index}`,
            text,
            raw: event,
          });
        }
        return;
      }

      if (options.includeUnknown) {
        out.push({
          kind: 'unknown',
          id: itemId ? `unknown:${itemId}` : `unknown:${index}`,
          raw: event,
        });
      }
      return;
    }

    if (options.includeUnknown) {
      out.push({
        kind: 'unknown',
        id: `unknown:${index}`,
        raw: event,
      });
    }
  });

  return out;
}
