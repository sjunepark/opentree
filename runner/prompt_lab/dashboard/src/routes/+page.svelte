<script lang="ts">
	import type { CombinationResult, ResultGrid } from '$lib/results';
	import { loadResults, getResult } from '$lib/results';

	let grid: ResultGrid = $state({ prompts: [], inputs: [], results: new Map() });
	let selectedResult: CombinationResult | null = $state(null);
	let loading = $state(true);
	let agent = 'tree_agent';

	// Load data on mount
	$effect(() => {
		loadResults(agent).then((result) => {
			grid = result;
			loading = false;
		});
	});

	function selectResult(promptName: string, inputId: string) {
		selectedResult = getResult(grid, promptName, inputId) ?? null;
	}

	function closeModal() {
		selectedResult = null;
	}

	function getStatusClass(result: CombinationResult | undefined): string {
		if (!result) return 'empty';
		if (result.error) return 'error';
		if (result.matches_expected === true) return 'pass';
		if (result.matches_expected === false) return 'fail';
		return 'neutral';
	}

	function getStatusIcon(result: CombinationResult | undefined): string {
		if (!result) return '-';
		if (result.error) return '!';
		if (result.matches_expected === true) return '✓';
		if (result.matches_expected === false) return '✗';
		return '○';
	}
</script>

<svelte:head>
	<title>Prompt Laboratory - {agent}</title>
</svelte:head>

<main>
	<header>
		<h1>Prompt Laboratory</h1>
		<p class="subtitle">Agent: <strong>{agent}</strong></p>
	</header>

	{#if loading}
		<div class="loading">Loading results...</div>
	{:else if grid.prompts.length === 0}
		<div class="empty-state">
			<h2>No Results Found</h2>
			<p>Run <code>prompt-lab run {agent}</code> to generate results.</p>
		</div>
	{:else}
		<div class="grid-container">
			<table class="results-grid">
				<thead>
					<tr>
						<th class="corner">Input / Prompt</th>
						{#each grid.prompts as prompt}
							<th class="prompt-header">{prompt}</th>
						{/each}
					</tr>
				</thead>
				<tbody>
					{#each grid.inputs as inputId}
						<tr>
							<td class="input-label">{inputId}</td>
							{#each grid.prompts as prompt}
								{@const result = getResult(grid, prompt, inputId)}
								<td
									class="cell {getStatusClass(result)}"
									onclick={() => selectResult(prompt, inputId)}
									onkeydown={(e) => e.key === 'Enter' && selectResult(prompt, inputId)}
									role="button"
									tabindex="0"
								>
									<span class="icon">{getStatusIcon(result)}</span>
									{#if result?.actual_decision}
										<span class="decision">{result.actual_decision}</span>
									{/if}
								</td>
							{/each}
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		<div class="legend">
			<span class="legend-item"><span class="icon pass">✓</span> Pass</span>
			<span class="legend-item"><span class="icon fail">✗</span> Fail</span>
			<span class="legend-item"><span class="icon error">!</span> Error</span>
			<span class="legend-item"><span class="icon neutral">○</span> No expectation</span>
		</div>
	{/if}

	{#if selectedResult}
		<div class="modal-backdrop" onclick={closeModal} onkeydown={(e) => e.key === 'Escape' && closeModal()} role="button" tabindex="0">
			<div class="modal" onclick={(e) => e.stopPropagation()} onkeydown={() => {}} role="dialog" tabindex="-1">
				<header>
					<h2>{selectedResult.prompt_name} × {selectedResult.input_id}</h2>
					<button class="close-btn" onclick={closeModal}>×</button>
				</header>

				<div class="modal-content">
					<section>
						<h3>Result</h3>
						<dl>
							<dt>Expected</dt>
							<dd>{selectedResult.expected_decision ?? 'N/A'}</dd>
							<dt>Actual</dt>
							<dd class={getStatusClass(selectedResult)}>{selectedResult.actual_decision ?? 'N/A'}</dd>
							<dt>Duration</dt>
							<dd>{selectedResult.duration_ms}ms</dd>
						</dl>
					</section>

					{#if selectedResult.error}
						<section class="error-section">
							<h3>Error</h3>
							<pre>{selectedResult.error}</pre>
						</section>
					{/if}

					{#if selectedResult.output}
						<section>
							<h3>Output</h3>
							<pre>{JSON.stringify(selectedResult.output, null, 2)}</pre>
						</section>
					{/if}
				</div>
			</div>
		</div>
	{/if}
</main>

<style>
	:global(body) {
		margin: 0;
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
		background: #1a1a2e;
		color: #eee;
	}

	main {
		max-width: 1200px;
		margin: 0 auto;
		padding: 2rem;
	}

	header {
		margin-bottom: 2rem;
	}

	h1 {
		margin: 0;
		font-size: 2rem;
	}

	.subtitle {
		margin: 0.5rem 0 0;
		color: #888;
	}

	.loading, .empty-state {
		text-align: center;
		padding: 4rem;
		color: #888;
	}

	.empty-state code {
		background: #2a2a4e;
		padding: 0.25rem 0.5rem;
		border-radius: 4px;
	}

	.grid-container {
		overflow-x: auto;
	}

	.results-grid {
		border-collapse: collapse;
		width: 100%;
	}

	.results-grid th,
	.results-grid td {
		border: 1px solid #333;
		padding: 0.75rem;
		text-align: center;
	}

	.corner {
		background: #2a2a4e;
		text-align: left !important;
	}

	.prompt-header {
		background: #2a2a4e;
		font-weight: 600;
		min-width: 120px;
	}

	.input-label {
		background: #2a2a4e;
		text-align: left !important;
		font-weight: 500;
	}

	.cell {
		cursor: pointer;
		transition: background 0.2s;
	}

	.cell:hover {
		filter: brightness(1.2);
	}

	.cell.pass { background: #1e4620; }
	.cell.fail { background: #4a1515; }
	.cell.error { background: #4a3515; }
	.cell.neutral { background: #2a2a4e; }
	.cell.empty { background: #1a1a2e; }

	.icon {
		font-size: 1.2rem;
		display: block;
	}

	.decision {
		font-size: 0.75rem;
		color: #aaa;
		display: block;
		margin-top: 0.25rem;
	}

	.legend {
		margin-top: 1rem;
		display: flex;
		gap: 1.5rem;
		justify-content: center;
		color: #888;
	}

	.legend-item {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.legend .icon {
		font-size: 1rem;
		width: 1.5rem;
		height: 1.5rem;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 4px;
	}

	.legend .icon.pass { background: #1e4620; }
	.legend .icon.fail { background: #4a1515; }
	.legend .icon.error { background: #4a3515; }
	.legend .icon.neutral { background: #2a2a4e; }

	.modal-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.8);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 100;
	}

	.modal {
		background: #2a2a4e;
		border-radius: 8px;
		max-width: 600px;
		width: 90%;
		max-height: 80vh;
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}

	.modal header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 1rem 1.5rem;
		border-bottom: 1px solid #444;
		margin: 0;
	}

	.modal h2 {
		margin: 0;
		font-size: 1.25rem;
	}

	.close-btn {
		background: none;
		border: none;
		color: #888;
		font-size: 1.5rem;
		cursor: pointer;
		padding: 0;
		line-height: 1;
	}

	.close-btn:hover {
		color: #fff;
	}

	.modal-content {
		padding: 1.5rem;
		overflow-y: auto;
	}

	.modal section {
		margin-bottom: 1.5rem;
	}

	.modal h3 {
		margin: 0 0 0.75rem;
		font-size: 1rem;
		color: #888;
	}

	.modal dl {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 0.5rem 1rem;
		margin: 0;
	}

	.modal dt {
		color: #888;
	}

	.modal dd {
		margin: 0;
	}

	.modal dd.pass { color: #4caf50; }
	.modal dd.fail { color: #f44336; }
	.modal dd.error { color: #ff9800; }

	.modal pre {
		background: #1a1a2e;
		padding: 1rem;
		border-radius: 4px;
		overflow-x: auto;
		margin: 0;
		font-size: 0.875rem;
	}

	.error-section pre {
		color: #ff9800;
	}
</style>
