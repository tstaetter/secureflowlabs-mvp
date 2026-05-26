<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import '../app.css';
	import { page } from '$app/stores';

	let { children } = $props();

	const dropdownLinks = [
		{ href: '/providers', label: 'Providers' },
		{ href: '/endpoints', label: 'Endpoints' },
		{ href: '/capabilities', label: 'Capabilities' },
		{ href: '/execution-plans', label: 'Execution Plans' }
	];

	let pathname = $derived($page.url.pathname);
	let dropdownOpen = $state(false);

	function toggle() {
		dropdownOpen = !dropdownOpen;
	}

	function close() {
		dropdownOpen = false;
	}

	function handleClickOutside(e: MouseEvent) {
		const target = e.target as HTMLElement;
		if (!target.closest('#nav-dropdown')) {
			close();
		}
	}

	$effect(() => {
		document.addEventListener('click', handleClickOutside);
		return () => document.removeEventListener('click', handleClickOutside);
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<nav class="border-b border-neutral-800 bg-neutral-950">
	<div class="mx-auto flex max-w-4xl items-center gap-8 px-4 py-3">
		<a href="/" class="text-sm font-semibold tracking-tight text-white">Secureflow</a>

		<div class="flex items-center gap-1">
			<!-- Upload -->
			<a
				href="/"
				class="rounded-lg px-3 py-1.5 text-sm transition
					{pathname === '/' ? 'bg-neutral-800 text-white' : 'text-neutral-400 hover:text-neutral-200'}"
			>
				Upload
			</a>

			<!-- Dropdown -->
			<div id="nav-dropdown" class="relative">
				<button
					onclick={toggle}
					class="flex items-center gap-1 rounded-lg px-3 py-1.5 text-sm transition
						{pathname !== '/' ? 'bg-neutral-800 text-white' : 'text-neutral-400 hover:text-neutral-200'}"
				>
					Data
					<svg
						class="h-3 w-3 transition {dropdownOpen ? 'rotate-180' : ''}"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						viewBox="0 0 24 24"
					>
						<path d="M6 9l6 6 6-6" />
					</svg>
				</button>

				{#if dropdownOpen}
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div
						class="absolute left-0 top-full z-50 mt-1 w-40 rounded-lg border border-neutral-700 bg-neutral-900 py-1 shadow-lg"
						onclick={close}
						onkeydown={(e) => e.key === 'Escape' && close()}
					>
						{#each dropdownLinks as link}
							<a
								href={link.href}
								class="block px-4 py-2 text-sm transition
									{pathname === link.href
									? 'bg-neutral-800 text-white'
									: 'text-neutral-400 hover:bg-neutral-800 hover:text-neutral-200'}"
							>
								{link.label}
							</a>
						{/each}
					</div>
				{/if}
			</div>
		</div>
	</div>
</nav>

<main>
	{@render children()}
</main>
