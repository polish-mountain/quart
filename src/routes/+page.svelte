<script lang="ts">
	import Fractal from '$lib/Fractal.svelte';
	import '../app.css';

	import { ResponseType, fetch } from '@tauri-apps/api/http';
	import { invoke } from '@tauri-apps/api/tauri';
	import { onMount } from 'svelte';

	let stuff: any = null;

	let noise: number[] = [];
	let noise2: number[] = [];
	let noise3: number[] = [];
	async function py(code: string) {
		noise = (await invoke('invoke_py', { code })) as number[];
		noise2 = (await invoke('invoke_py', { code })) as number[];
		noise3 = (await invoke('invoke_py', { code })) as number[];
		console.log(noise);
		console.log(noise2);
		console.log(noise3);
	}

	let creator: string | null = null;

	let frame: HTMLIFrameElement;
	let expression: string =
		'cx_div(vec2(1.0, 0.0), cpow(vec2(i, 0.0), vec2(uv.x * 6.0 + 2.0, uv.y * 6.0)))';
</script>

<svelte:window on:message={(evt) => py(evt.data)} />

<input
	class="bg-neutral-800 border border-neutral-500 m-2 p-2 text-neutral-100"
	bind:value={expression}
/>
<div
	class="grid justify-center items-center gap-4 p-2 px-4 w-full"
	style="grid-template-columns: 1fr 1fr 1fr;"
>
	<Fractal {expression} {noise} />
	<Fractal {expression} noise={noise2} />
	<Fractal {expression} noise={noise3} />
</div>
<div class="p-4 flex justify-center items-center">
	<button
		class="bg-neutral-700 p-3 text-neutral-100"
		on:click={() => frame.contentWindow?.postMessage('click', '*')}>Generate</button
	>
</div>
<div class="grow border-t-2 border-neutral-700">
	<iframe bind:this={frame} src="http://localhost:6800/composer/files/new" class="w-full h-full" />
</div>
