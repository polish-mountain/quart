<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';

	let stuff: any = null;

	async function greet() {
		stuff = await invoke('invoke_py', {});
	}
</script>

<h1>Welcome to SvelteKit</h1>
<p>Visit <a href="https://kit.svelte.dev">kit.svelte.dev</a> to read the documentation</p>
DUUPA
<button on:click={greet}>Do it</button>
{stuff}
