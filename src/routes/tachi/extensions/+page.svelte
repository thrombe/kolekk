<script lang="ts" context="module">
    import { writable, type Writable } from 'svelte/store';
    import type { Extension, JsonObject } from 'types';

    interface Item<E, K> {
      data: E,
      id: K,
    }
    export let extensions: Writable<Item<JsonObject<Extension>, any>[]> = writable(new Array());
    export let search_query = writable('');
</script>


<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri';
    import { onMount } from 'svelte';
    import Card from './Card.svelte';
    import VirtualScrollable from '$lib/VirtualScrollable.svelte';

    const facet = '/temp/tachi/extension';

    const get_all_extensions = async () => {
        let exts: Extension[] = await invoke('tachidesk_get_all_extensions');
        await invoke('delete_facet_objects', { facet });
        await invoke('enter_searchable', {
            facet,
            data: exts.map((e) => {
                return { obj: e, search_context: [e.name] };
            })
        });
        search();
    };
    const search = async () => {
        let exts: JsonObject<Extension>[] = await invoke('search_jsml', {
            query: $search_query,
            facet,
            limit: 50,
            offset: 0
        });
        $extensions = exts.map((e) => {
            return { data: e, id: e.id };
        });
        // console.log($extensions.map(e => e.data.obj.name));
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
    <input bind:value={$search_query} on:input={search} />
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
        <Card {width} aspect_ratio={item_aspect_ratio} selected={s} ext={ext.obj} {root} />
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
