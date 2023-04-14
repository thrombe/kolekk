<script lang="ts">
    import { tick } from 'svelte';
    import Card from './Card.svelte';
    import { invoke } from '@tauri-apps/api/tauri';
    import { page } from '$app/stores';
    import type { MangaListPage } from 'types';
    import VirtualScrollable from '$lib/VirtualScrollable.svelte';

    let search_query = '';
    let search_results = { mangaList: new Array(), hasNextPage: false };
    let page_num_fetched = 1;
    let include_adult = false;
    const search = async () => {
        if (search_query == '') {
            page_num_fetched = 1;
            search_results = await invoke('tachidesk_get_popular_manga_list', {
                sourceId: $page.params.src_id,
                page: 1
            });
            console.log(search_results);
        } else {
            page_num_fetched = 1;
            search_results = await invoke('tachidesk_search_manga_in', {
                sourceId: $page.params.src_id,
                query: search_query,
                page: 1
            });
            console.log(search_results);
        }

        id_set = new Set();
        collisions = new Array();
        search_results.mangaList = search_results.mangaList.filter((item) => {
            if (id_set.has(item.id)) {
                collisions.push(item);
                return false;
            } else {
                id_set.add(item.id);
                return true;
            }
        });

        setTimeout(end_reached, 500);
    };

    let id_set = new Set();
    let collisions = new Array();
    const end_reached = async () => {
        // return;
        await tick();

        if (!end_is_visible) {
            return;
        }

        if (search_results.hasNextPage) {
            let new_res: MangaListPage;
            page_num_fetched += 1;
            if (search_query == '') {
                new_res = await invoke('tachidesk_get_popular_manga_list', {
                    sourceId: $page.params.src_id,
                    page: page_num_fetched
                });
            } else {
                new_res = await invoke('tachidesk_search_manga_in', {
                    sourceId: $page.params.src_id,
                    query: search_query,
                    page: page_num_fetched
                });
            }
            new_res.mangaList = new_res.mangaList.filter((item) => {
                if (id_set.has(item.id)) {
                    collisions.push(item);
                    return false;
                } else {
                    id_set.add(item.id);
                    return true;
                }
            });
            console.log(
                page_num_fetched,
                new_res.mangaList.map((e) => e.id)
            );
            search_results.hasNextPage = new_res.hasNextPage;
            search_results.mangaList.push(...new_res.mangaList);
            search_results = search_results;

            setTimeout(end_reached, 500);
        }
    };

    let end_is_visible = true;
    let window_width = 100;
    let window_height = 100;
    let selected = 0;
    let search_input: any;
    const on_keydown = async (event: KeyboardEvent, scroll_selected_into_view: any) => {
        if (event.key == 'a') {
            // await add_tag_button();
            // event.preventDefault();
        } else if (event.key == '/') {
            selected = 0;
            await tick();
            await scroll_selected_into_view();
            search_query = '';
            search_input.focus();
            event.preventDefault();
        }
    };

    let item_aspect_ratio = 2 / 3;
    $: items = search_results.mangaList.map(e => {
        return {id: e.id, data: e};
    })
    search();
</script>

<cl class={'inputs'}>
    <input bind:value={search_query} on:input={search} bind:this={search_input} />
    <button on:click={search}>Search</button>
    <button
        on:click={() => {
            include_adult = !include_adult;
            search();
        }}
    >
        include mature: {include_adult}
    </button>
    <button
        on:click={() => {
            let id_set = new Set();
            let collisions = new Array();
            search_results.mangaList.forEach((m) => {
                if (id_set.has(m.id)) {
                    collisions.push(m.id);
                } else {
                    id_set.add(m.id);
                }
            });
            console.log(collisions);
        }}
    >
        {search_results.mangaList.length} | end visible: {end_is_visible}
    </button>
</cl>

<cl>
    <VirtualScrollable
        bind:items={items}
        columns={5}
        width={window_width}
        item_height={window_width / 5 / item_aspect_ratio}
        {end_reached}
        bind:selected
        {on_keydown}
        bind:end_is_visible
        let:item_width={width}
        let:root
        let:item={manga}
        let:index={i}
        let:selected={s}
    >
        <Card
            {width}
            aspect_ratio={item_aspect_ratio}
            selected={s}
            {manga}
            {root}
        />
    </VirtualScrollable>
</cl>

<svelte:window bind:innerHeight={window_height} bind:innerWidth={window_width} />

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
        overflow: hidden;
        width: 100%;
        height: calc(100% - var(--input-height));
    }
</style>
