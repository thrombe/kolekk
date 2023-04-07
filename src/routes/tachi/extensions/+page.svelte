<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri';
    import { onMount } from 'svelte';
    import { extensions } from './state';
    import Scrollable from '$lib/Scrollable.svelte';
    import Card from './Card.svelte';
    import Virtual from '$lib/Virtual.svelte';

    // TODO: make extensions searchable
    const get_all_extensions = async () => {
        $extensions = await invoke('tachidesk_get_all_extensions');
        console.log($extensions);
    };

    onMount(async () => {
        await invoke('init_tachidesk_client');
        if ($extensions.length == 0) {
            await get_all_extensions();
        }
    });

    const on_keydown = async (_: KeyboardEvent) => {};

    let window_width = 100;
    let selected = 0;

    let item_aspect_ratio = 2 / 3;
</script>

<svelte:window bind:innerWidth={window_width} />
<cl class={'inputs'}>
    <button on:click={get_all_extensions}>get all extensions</button>
</cl>

<cl>
    <Scrollable
        columns={5}
        num_items={$extensions.length}
        bind:selected
        width={window_width}
        item_aspect_ratio={item_aspect_ratio}
        {on_keydown}
        keyboard_control={true}

        let:item_width={width}
        let:root
    >
        {#each $extensions as ext, i (ext.pkgName)}
            <Virtual
                {width}
                aspect_ratio={item_aspect_ratio}
                {root}
            >
                <Card
                    {width}
                    aspect_ratio={item_aspect_ratio}
                    selected={selected == i ||
                        (i == $extensions.length - 1 && selected >= $extensions.length)}
                    {ext}
                    on_click={() => {
                        selected = i;
                    }}
                    {root}
                />
            </Virtual>
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
