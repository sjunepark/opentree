<script lang="ts">
  import type { RunnerConfig } from './types';

  interface Props {
    config: RunnerConfig;
  }

  let { config }: Props = $props();

  // Format value for display
  function formatValue(value: unknown): string {
    if (value === null || value === undefined) {
      return 'Not set';
    }
    if (typeof value === 'boolean') {
      return value ? 'Yes' : 'No';
    }
    if (typeof value === 'number') {
      return value.toLocaleString();
    }
    return String(value);
  }

  // Known config keys with labels
  const knownKeys: Array<{ key: keyof RunnerConfig; label: string }> = [
    { key: 'max_iterations', label: 'Max Iterations' },
    { key: 'max_attempts_default', label: 'Max Attempts (default)' },
    { key: 'iteration_timeout_secs', label: 'Iteration Timeout (secs)' },
    { key: 'iteration_output_limit', label: 'Output Limit (bytes)' },
    { key: 'guard_command', label: 'Guard Command' },
  ];

  // Get other keys not in knownKeys
  $effect(() => {
    // Reactivity placeholder
  });

  function getOtherKeys(cfg: RunnerConfig): string[] {
    const known = new Set(knownKeys.map((k) => k.key));
    return Object.keys(cfg).filter((k) => !known.has(k as keyof RunnerConfig));
  }
</script>

<div class="config-panel">
  <div class="section">
    <h3 class="section-title">Runner Configuration</h3>
    <div class="config-grid">
      {#each knownKeys as { key, label }}
        {#if config[key] !== undefined}
          <div class="config-item">
            <span class="config-label">{label}</span>
            <span class="config-value" class:mono={key === 'guard_command'}>
              {formatValue(config[key])}
            </span>
          </div>
        {/if}
      {/each}
    </div>
  </div>

  {#if getOtherKeys(config).length > 0}
    <div class="section">
      <h3 class="section-title">Other Settings</h3>
      <div class="config-grid">
        {#each getOtherKeys(config) as key}
          <div class="config-item">
            <span class="config-label">{key}</span>
            <span class="config-value mono">
              {formatValue(config[key])}
            </span>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .config-panel {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .section-title {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #64748b;
    margin-bottom: 0.75rem;
  }

  .config-grid {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .config-item {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    padding: 0.5rem 0;
    border-bottom: 1px solid #f1f5f9;
  }

  .config-item:last-child {
    border-bottom: none;
  }

  .config-label {
    font-size: 0.875rem;
    color: #64748b;
  }

  .config-value {
    font-weight: 500;
    color: #1e293b;
  }

  .config-value.mono {
    font-family: ui-monospace, monospace;
    font-size: 0.75rem;
    background-color: #f1f5f9;
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    color: #475569;
  }
</style>
