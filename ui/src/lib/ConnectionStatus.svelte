<script lang="ts">
  import { sseConnected } from './stores';
</script>

<div class="connection-status" class:connected={$sseConnected}>
  <span class="indicator"></span>
  <span class="label">{$sseConnected ? 'Live' : 'Disconnected'}</span>
</div>

<style>
  .connection-status {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.75rem;
    color: #64748b;
    padding: 0.25rem 0.5rem;
    border-radius: 9999px;
    background-color: #f1f5f9;
  }

  .connection-status.connected {
    background-color: #dcfce7;
    color: #166534;
  }

  .indicator {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
    background-color: #94a3b8;
  }

  .connected .indicator {
    background-color: #22c55e;
    animation: pulse 2s infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }

  .label {
    font-weight: 500;
  }
</style>
