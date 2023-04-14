<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri';
    import { onMount } from 'svelte';
    import { extensions } from './state';
    import Card from './Card.svelte';
    import VirtualScrollable from '$lib/VirtualScrollable.svelte';
    import type { Extension } from 'types';

    // TODO: make extensions searchable
    const get_all_extensions = async () => {
        let exts: Extension[] = await invoke('tachidesk_get_all_extensions');
        $extensions = exts.map((e) => {return {data: e, id: e.pkgName}});
        console.log($extensions);
    };

    onMount(async () => {
        await invoke('init_tachidesk_client');
        if ($extensions.length == 0) {
            await get_all_extensions();
        }
    });

    const on_keydown = async (_: KeyboardEvent, _scroll_selected_into_view: any) => {};

    let window_width = 100;
    let selected = 0;

    let item_aspect_ratio = 2 / 3;
</script>

<svelte:window bind:innerWidth={window_width} />
<cl class={'inputs'}>
    <button on:click={get_all_extensions}>get all extensions</button>
</cl>

<cl>
    <VirtualScrollable
        columns={5}
        bind:items={$extensions}
        width={window_width}
        item_height={window_width / 5 / item_aspect_ratio}
        bind:selected
        {on_keydown}
        let:item_width={width}
        let:root
        let:item={ext}
        let:index={i}
        let:selected={s}
    >
        <Card
            {width}
            aspect_ratio={item_aspect_ratio}
            selected={s}
            {ext}
            {root}
        />
    </VirtualScrollable>
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
