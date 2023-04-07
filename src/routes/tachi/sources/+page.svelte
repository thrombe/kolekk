<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri';
    import { onMount } from 'svelte';
    import { sources } from './state';
    import Scrollable from '$lib/Scrollable.svelte';
    import Card from './Card.svelte';

    // TODO: make sources searchable
    const get_sources = async () => {
        $sources = await invoke('tachidesk_get_source_list');
        console.log($sources);
        $sources = $sources.filter((s) => s.lang == 'en');
    };

    onMount(async () => {
        await invoke('init_tachidesk_client');
        if ($sources.length == 0) {
            await get_sources();
        }
    });

    const on_keydown = async (e: KeyboardEvent) => {};

    let window_width = 100;
    let selected = 0;
</script>

<svelte:window bind:innerWidth={window_width} />
<cl class={'inputs'}>
    <button on:click={get_sources}>get sources</button>
</cl>

<cl>
    <Scrollable
        columns={5}
        num_items={$sources.length}
        bind:selected
        width={window_width}
        {on_keydown}
        keyboard_control={true}
        let:item_width={width}
        let:root
    >
        {#each $sources as source, i (source.id)}
            <Card
                {width}
                aspect_ratio={2 / 3}
                selected={selected == i ||
                    (i == $sources.length - 1 && selected >= $sources.length)}
                {source}
                on_click={() => {
                    selected = i;
                }}
                {root}
            />
        {/each}
    </Scrollable>
</cl>

<style>
    * {
        --input-height: 33px;
    }

    .inputs {
        height: var(--input-height);
    }

    cl {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        overflow: auto;
        width: 100%;
        height: calc(100% - var(--input-height));
    }
</style>
