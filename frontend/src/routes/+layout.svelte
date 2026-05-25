<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import '../app.css';
	import { page } from '$app/stores';

	let { children } = $props();

	const links = [
		{ href: '/', label: 'Upload' },
		{ href: '/providers', label: 'Providers' }
	];

	let pathname = $derived($page.url.pathname);
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<nav class="border-b border-neutral-800 bg-neutral-950">
	<div class="mx-auto flex max-w-4xl items-center gap-8 px-4 py-3">
		<a href="/" class="text-sm font-semibold tracking-tight text-white"> Secureflow </a>
		<div class="flex gap-1">
			{#each links as link}
				<a
					href={link.href}
					class="rounded-lg px-3 py-1.5 text-sm transition
						{pathname === link.href ? 'bg-neutral-800 text-white' : 'text-neutral-400 hover:text-neutral-200'}"
				>
					{link.label}
				</a>
			{/each}
		</div>
	</div>
</nav>

<main>
	{@render children()}
</main>
