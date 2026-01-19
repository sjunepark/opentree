import { connection } from './stores.svelte';

type ChangeHandler = (event: ChangeEvent) => void;

interface ChangeEvent {
  type:
    | 'tree_changed'
    | 'run_state_changed'
    | 'iteration_added'
    | 'iteration_completed'
    | 'stream_updated'
    | 'config_changed'
    | 'assumptions_changed'
    | 'questions_changed';
  run_id?: string;
  iter?: number;
}

let eventSource: EventSource | null = null;
let reconnectTimeout: number | null = null;
let staticModeInterval: number | null = null;
const handlers: Set<ChangeHandler> = new Set();

const STATIC_MODE_THRESHOLD_MS = 10000;

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
    connection.sseConnected = true;
    connection.lastEventTime = Date.now();
    if (reconnectTimeout) {
      clearTimeout(reconnectTimeout);
      reconnectTimeout = null;
    }
    // Start static mode detection
    if (!staticModeInterval) {
      staticModeInterval = window.setInterval(() => {
        if (connection.sseConnected && connection.lastEventTime !== null) {
          if (Date.now() - connection.lastEventTime > STATIC_MODE_THRESHOLD_MS) {
            connection.staticMode = true;
          }
        }
      }, 5000);
    }
  });

  eventSource.addEventListener('change', (event) => {
    connection.lastEventTime = Date.now();
    connection.staticMode = false;
    try {
      const data: ChangeEvent = JSON.parse(event.data);
      handlers.forEach((handler) => handler(data));
    } catch (e) {
      console.error('Failed to parse SSE event:', e);
    }
  });

  eventSource.addEventListener('error', () => {
    connection.sseConnected = false;
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
  if (staticModeInterval) {
    clearInterval(staticModeInterval);
    staticModeInterval = null;
  }
  if (eventSource) {
    eventSource.close();
    eventSource = null;
  }
  connection.sseConnected = false;
  connection.staticMode = false;
  connection.lastEventTime = null;
}
