<script lang="ts">
	import { PUBLIC_API_PREFIX } from '$env/static/public';

	interface EndpointSummary {
		id: string;
		internal_id: string;
		method: string;
		path: string;
		summary: string | null;
		inputs: number;
		outputs: number;
	}

	let endpoints = $state<EndpointSummary[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	async function load() {
		loading = true;
		error = null;
		try {
			const res = await fetch(`${PUBLIC_API_PREFIX}/endpoints`);
			if (!res.ok) throw new Error(await res.text());
			endpoints = await res.json();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load';
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		load();
	});

	const methodColour = (m: string) => {
		switch (m) {
			case 'Get':
				return 'text-emerald-400';
			case 'Post':
				return 'text-blue-400';
			case 'Put':
			case 'Patch':
				return 'text-amber-400';
			case 'Delete':
				return 'text-red-400';
			default:
				return 'text-neutral-400';
		}
	};
</script>

<div class="min-h-screen bg-neutral-950 px-4 py-12">
	<div class="mx-auto max-w-5xl space-y-8">
		<header class="flex items-center justify-between">
			<div>
				<h1 class="text-2xl font-semibold tracking-tight text-white">Endpoints</h1>
				<p class="mt-1 text-sm text-neutral-400">Normalized API endpoints across all providers</p>
			</div>
			<button
				onclick={load}
				class="rounded-lg bg-neutral-800 px-4 py-2 text-sm font-medium text-neutral-200 transition hover:bg-neutral-700"
			>
				Refresh
			</button>
		</header>

		{#if loading}
			<div class="flex items-center justify-center py-20">
				<svg class="h-6 w-6 animate-spin text-neutral-500" fill="none" viewBox="0 0 24 24">
					<circle
						class="opacity-25"
						cx="12"
						cy="12"
						r="10"
						stroke="currentColor"
						stroke-width="4"
					/>
					<path
						class="opacity-75"
						fill="currentColor"
						d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
					/>
				</svg>
			</div>
		{:else if error}
			<div class="rounded-lg border border-red-800/50 bg-red-950/20 px-4 py-3">
				<p class="text-sm text-red-400">{error}</p>
			</div>
		{:else if endpoints.length === 0}
			<div
				class="rounded-xl border border-dashed border-neutral-700 bg-neutral-900/50 px-6 py-16 text-center"
			>
				<p class="text-sm text-neutral-400">No endpoints yet.</p>
			</div>
		{:else}
			<div class="overflow-hidden rounded-xl border border-neutral-800">
				<table class="w-full text-left text-sm">
					<thead>
						<tr class="border-b border-neutral-800 bg-neutral-900/50">
							<th class="px-6 py-3 font-medium text-neutral-400">Method</th>
							<th class="px-6 py-3 font-medium text-neutral-400">Path</th>
							<th class="px-6 py-3 font-medium text-neutral-400">Summary</th>
							<th class="px-6 py-3 font-medium text-neutral-400 text-right">Inputs</th>
							<th class="px-6 py-3 font-medium text-neutral-400 text-right">Outputs</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-neutral-800">
						{#each endpoints as ep}
							<tr class="transition hover:bg-neutral-900/30">
								<td class="px-6 py-3">
									<span class="font-mono text-xs font-medium {methodColour(ep.method)}">
										{ep.method.toUpperCase()}
									</span>
								</td>
								<td class="px-6 py-3 font-mono text-xs text-neutral-200">{ep.path}</td>
								<td class="px-6 py-3 text-neutral-400">{ep.summary ?? '—'}</td>
								<td class="px-6 py-3 text-right tabular-nums text-neutral-400">{ep.inputs}</td>
								<td class="px-6 py-3 text-right tabular-nums text-neutral-400">{ep.outputs}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
			<p class="text-xs text-neutral-600">
				{endpoints.length} endpoint{endpoints.length !== 1 ? 's' : ''}
			</p>
		{/if}
	</div>
</div>
