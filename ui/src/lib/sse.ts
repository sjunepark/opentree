import { sseConnected } from './stores';

type ChangeHandler = (event: ChangeEvent) => void;

interface ChangeEvent {
  type: 'tree_changed' | 'run_state_changed' | 'iteration_added' | 'config_changed' | 'assumptions_changed' | 'questions_changed';
  run_id?: string;
  iter?: number;
}

let eventSource: EventSource | null = null;
let reconnectTimeout: number | null = null;
const handlers: Set<ChangeHandler> = new Set();

export function subscribe(handler: ChangeHandler): () => void {
  handlers.add(handler);
  return () => handlers.delete(handler);
}

export function connect(): void {
  if (eventSource) {
    return;
  }

  eventSource = new EventSource('/events');

  eventSource.addEventListener('connected', () => {
    sseConnected.set(true);
    if (reconnectTimeout) {
      clearTimeout(reconnectTimeout);
      reconnectTimeout = null;
    }
  });

  eventSource.addEventListener('change', (event) => {
    try {
      const data: ChangeEvent = JSON.parse(event.data);
      handlers.forEach((handler) => handler(data));
    } catch (e) {
      console.error('Failed to parse SSE event:', e);
    }
  });

  eventSource.addEventListener('error', () => {
    sseConnected.set(false);
    eventSource?.close();
    eventSource = null;

    // Reconnect after delay
    if (!reconnectTimeout) {
      reconnectTimeout = window.setTimeout(() => {
        reconnectTimeout = null;
        connect();
      }, 3000);
    }
  });
}

export function disconnect(): void {
  if (reconnectTimeout) {
    clearTimeout(reconnectTimeout);
    reconnectTimeout = null;
  }
  if (eventSource) {
    eventSource.close();
    eventSource = null;
  }
  sseConnected.set(false);
}
