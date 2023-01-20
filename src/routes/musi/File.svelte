<script lang="ts">
	import { invoke, convertFileSrc } from '@tauri-apps/api/tauri'
	import { playing } from "./Player"

	export let path: string;

	$: type = path.slice(path.lastIndexOf('.') + 1);
    $: {
		console.log(path)
		console.log(convertFileSrc(path))
	}

	const allowed_types = ["mp3", "m4a"]

	$: show = allowed_types.includes(type)
	$: name = path.split("/").pop()

	const play = async () => {
		let p = "file://" + path
		console.log(p)
		playing.set(p)
	}
</script>

{#if show}
	<button on:click={play}>{name}</button>
{/if}
