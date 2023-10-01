<script lang="ts">
	import Fractal from '$lib/Fractal.svelte';
	import '../app.css';

	import { ResponseType, fetch } from '@tauri-apps/api/http';
	import { invoke } from '@tauri-apps/api/tauri';
	import { onMount } from 'svelte';

	let stuff: any = null;

	let noise: number[] = [];
	async function py(code: string) {
		noise = (await invoke('invoke_py', { code })) as number[];
		console.log(noise);
	}

	let creator: string | null = null;

	let frame: HTMLIFrameElement;
	let expression: string =
		'cx_div(vec2(1.0, 0.0), cpow(vec2(i, 0.0), vec2(uv.x * 6.0 + 2.0, uv.y * 6.0)))';
</script>

<svelte:window on:message={(evt) => py(evt.data)} />

<input bind:value={expression} />
<Fractal {expression} {noise} />
<button on:click={() => frame.contentWindow?.postMessage('click', '*')}>Generate</button>
<div class="grow">
	<iframe bind:this={frame} src="http://localhost:6800/composer/files/new" class="w-full h-full" />
</div>
