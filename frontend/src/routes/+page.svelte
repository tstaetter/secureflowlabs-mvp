<script lang="ts">
	type UploadStatus = 'idle' | 'uploading' | 'success' | 'error';

	interface UploadResult {
		status: string;
		path: string;
		filename: string;
	}

	let file = $state<File | null>(null);
	let uploadStatus = $state<UploadStatus>('idle');
	let result = $state<UploadResult | null>(null);
	let errorMessage = $state<string | null>(null);
	let dragOver = $state(false);

	let canUpload = $derived(file !== null && uploadStatus !== 'uploading');
	let buttonLabel = $derived(uploadStatus === 'uploading' ? 'Uploading…' : 'Upload JSON File');

	function handleFileInput(e: Event) {
		const target = e.target as HTMLInputElement;
		const selected = target.files?.[0];
		if (selected) {
			file = selected;
			uploadStatus = 'idle';
			result = null;
			errorMessage = null;
		}
	}

	function handleDragOver(e: DragEvent) {
		e.preventDefault();
		dragOver = true;
	}

	function handleDragLeave() {
		dragOver = false;
	}

	function handleDrop(e: DragEvent) {
		e.preventDefault();
		dragOver = false;

		const dropped = e.dataTransfer?.files?.[0];
		if (dropped) {
			file = dropped;
			uploadStatus = 'idle';
			result = null;
			errorMessage = null;
		}
	}

	async function upload() {
		if (!file) return;

		uploadStatus = 'uploading';
		errorMessage = null;
		result = null;

		try {
			const formData = new FormData();
			formData.append('file', file);

			const res = await fetch('http://localhost:8000/upload', {
				method: 'POST',
				body: formData
			});

			if (!res.ok) {
				const text = await res.text();
				throw new Error(text || `Upload failed (${res.status})`);
			}

			result = await res.json();
			uploadStatus = 'success';
		} catch (err) {
			errorMessage = err instanceof Error ? err.message : 'Unknown error';
			uploadStatus = 'error';
		}
	}

	function reset() {
		file = null;
		uploadStatus = 'idle';
		result = null;
		errorMessage = null;
	}
</script>

<div class="flex min-h-screen items-center justify-center bg-neutral-950 px-4">
	<div class="w-full max-w-lg space-y-8">
		<!-- Header -->
		<header class="text-center">
			<h1 class="text-3xl font-semibold tracking-tight text-white">OpenAPI Spec Uploader</h1>
			<p class="mt-2 text-sm text-neutral-400">
				Upload a JSON OpenAPI specification to normalize and process it.
			</p>
		</header>

		<!-- Success state -->
		{#if uploadStatus === 'success' && result}
			<div class="space-y-4 rounded-xl border border-emerald-800/50 bg-emerald-950/30 p-6">
				<div class="flex items-center gap-2">
					<svg
						class="h-5 w-5 text-emerald-400"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						viewBox="0 0 24 24"
					>
						<path d="M5 13l4 4L19 7" />
					</svg>
					<span class="font-medium text-emerald-300">Upload complete</span>
				</div>

				<dl class="space-y-2 text-sm">
					<div class="flex justify-between">
						<dt class="text-neutral-400">Filename</dt>
						<dd class="font-mono text-neutral-200">{result.filename}</dd>
					</div>
					<div class="flex justify-between">
						<dt class="text-neutral-400">Saved to</dt>
						<dd class="font-mono text-neutral-200 break-all">{result.path}</dd>
					</div>
				</dl>

				<button
					onclick={reset}
					class="w-full rounded-lg bg-neutral-800 px-4 py-2 text-sm font-medium text-neutral-200 transition hover:bg-neutral-700 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-500"
				>
					Upload another file
				</button>
			</div>
		{:else}
			<!-- Drop zone -->
			<label
				class:dragover={dragOver}
				class="flex cursor-pointer flex-col items-center gap-4 rounded-xl border-2 border-dashed px-6 py-12 transition
				{dragOver
					? 'border-blue-400 bg-blue-950/20'
					: 'border-neutral-700 bg-neutral-900/50 hover:border-neutral-500'}
				{uploadStatus === 'error' ? 'border-red-700 bg-red-950/10' : ''}"
				ondragover={handleDragOver}
				ondragleave={handleDragLeave}
				ondrop={handleDrop}
			>
				<input
					type="file"
					accept=".json,application/json"
					onchange={handleFileInput}
					class="hidden"
				/>

				{#if file}
					<div class="flex items-center gap-2 rounded-lg bg-neutral-800 px-4 py-2">
						<svg
							class="h-5 w-5 text-neutral-400"
							fill="none"
							stroke="currentColor"
							stroke-width="1.5"
							viewBox="0 0 24 24"
						>
							<path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z" />
							<polyline points="14 2 14 8 20 8" />
						</svg>
						<span class="text-sm text-neutral-200">{file.name}</span>
						<button
							type="button"
							onclick={(e) => {
								e.stopPropagation();
								e.preventDefault();
								reset();
							}}
							aria-label="Remove file"
							class="ml-1 rounded p-0.5 text-neutral-500 transition hover:text-neutral-300 focus:outline-none"
						>
							<svg
								class="h-4 w-4"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
								viewBox="0 0 24 24"
							>
								<path d="M18 6L6 18M6 6l12 12" />
							</svg>
						</button>
					</div>

					<p class="text-xs text-neutral-500">{(file.size / 1024).toFixed(1)} kB</p>
				{:else}
					<svg
						class="h-10 w-10 text-neutral-500"
						fill="none"
						stroke="currentColor"
						stroke-width="1.5"
						viewBox="0 0 24 24"
					>
						<path d="M12 16V4m0 0L8 8m4-4l4 4M4 20h16" />
					</svg>
					<div class="text-center">
						<p class="text-sm font-medium text-neutral-300">Drop your JSON file here</p>
						<p class="mt-1 text-xs text-neutral-500">or click to browse — .json only</p>
					</div>
				{/if}
			</label>

			<!-- Error state -->
			{#if uploadStatus === 'error' && errorMessage}
				<div class="rounded-lg border border-red-800/50 bg-red-950/20 px-4 py-3">
					<p class="text-sm text-red-400">{errorMessage}</p>
				</div>
			{/if}

			<!-- Upload button -->
			<button
				onclick={upload}
				disabled={!canUpload}
				class="w-full rounded-lg px-4 py-3 text-sm font-semibold text-white transition focus:outline-none focus-visible:ring-2 focus-visible:ring-blue-500
				{canUpload
					? 'bg-blue-600 hover:bg-blue-500'
					: 'cursor-not-allowed bg-neutral-800 text-neutral-500'}"
			>
				{#if uploadStatus === 'uploading'}
					<span class="flex items-center justify-center gap-2">
						<svg class="h-4 w-4 animate-spin" fill="none" viewBox="0 0 24 24">
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
						Uploading…
					</span>
				{:else}
					{buttonLabel}
				{/if}
			</button>
		{/if}
	</div>
</div>
