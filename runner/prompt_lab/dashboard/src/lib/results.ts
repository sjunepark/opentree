/**
 * Types and utilities for loading prompt lab results.
 */

export interface TreeDecision {
	decision: 'execute' | 'decompose';
	summary: string;
	children?: Array<{
		title: string;
		goal: string;
		acceptance?: string[];
	}>;
}

export interface CombinationResult {
	prompt_name: string;
	prompt_hash: string;
	input_id: string;
	input_name: string;
	expected_decision: string | null;
	actual_decision: string | null;
	matches_expected: boolean | null;
	output: TreeDecision | null;
	error: string | null;
	duration_ms: number;
	timestamp: string;
}

export interface ResultGrid {
	prompts: string[];
	inputs: string[];
	results: Map<string, CombinationResult>;
}

/**
 * Load results from the results directory.
 * In development, fetch from the API endpoint.
 * In production, load from static JSON.
 */
export async function loadResults(agent: string): Promise<ResultGrid> {
	const indexUrl = `/results/${agent}/index.json`;

	try {
		const response = await fetch(indexUrl);
		if (!response.ok) {
			console.warn(`No results index found at ${indexUrl}`);
			return { prompts: [], inputs: [], results: new Map() };
		}

		const data = await response.json();
		const results = new Map<string, CombinationResult>();

		for (const result of data.results) {
			const key = `${result.prompt_name}:${result.input_id}`;
			results.set(key, result);
		}

		return {
			prompts: data.prompts,
			inputs: data.inputs,
			results
		};
	} catch (e) {
		console.error('Failed to load results:', e);
		return { prompts: [], inputs: [], results: new Map() };
	}
}

/**
 * Get a result for a specific prompt × input combination.
 */
export function getResult(
	grid: ResultGrid,
	promptName: string,
	inputId: string
): CombinationResult | undefined {
	return grid.results.get(`${promptName}:${inputId}`);
}

/**
 * Generate a result index from individual result files.
 * Used by the CLI to create the index.json.
 */
export function generateIndex(results: CombinationResult[]): {
	prompts: string[];
	inputs: string[];
	results: CombinationResult[];
} {
	const prompts = [...new Set(results.map(r => r.prompt_name))].sort();
	const inputs = [...new Set(results.map(r => r.input_id))].sort();

	return { prompts, inputs, results };
}
