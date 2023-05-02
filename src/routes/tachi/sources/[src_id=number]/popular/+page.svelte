<script lang="ts" context="module">
    import { writable } from 'svelte/store';

    let selected = writable(0);
    let search_results = writable({ mangaList: new Array(), hasNextPage: false });
    let search_query = writable('');
    let page_num_fetched = writable(1);
</script>

<script lang="ts">
    import { tick } from 'svelte';
    import Card from '$lib/Card.svelte';
    import { invoke } from '@tauri-apps/api/tauri';
    import { page } from '$app/stores';
    import type { MangaListPage } from 'types';
    import VirtualScrollable from '$lib/VirtualScrollable.svelte';

    let include_adult = false;
    const search = async () => {
        if ($search_query == '') {
            $page_num_fetched = 1;
            $search_results = await invoke('tachidesk_get_popular_manga_list', {
                sourceId: $page.params.src_id,
                page: 1
            });
            console.log($search_results);
        } else {
            $page_num_fetched = 1;
            $search_results = await invoke('tachidesk_search_manga_in', {
                sourceId: $page.params.src_id,
                query: $search_query,
                page: 1
            });
            console.log($search_results);
        }

        id_set = new Set();
        collisions = new Array();
        $search_results.mangaList = $search_results.mangaList.filter((item) => {
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

        if ($search_results.hasNextPage) {
            let new_res: MangaListPage;
            $page_num_fetched += 1;
            if ($search_query == '') {
                new_res = await invoke('tachidesk_get_popular_manga_list', {
                    sourceId: $page.params.src_id,
                    page: $page_num_fetched
                });
            } else {
                new_res = await invoke('tachidesk_search_manga_in', {
                    sourceId: $page.params.src_id,
                    query: $search_query,
                    page: $page_num_fetched
                });
            }
            let hasNextPage = new_res.mangaList.length > 0;
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
                $page_num_fetched,
                new_res.mangaList.map((e) => e.id)
            );
            $search_results.hasNextPage = new_res.hasNextPage && hasNextPage;
            $search_results.mangaList.push(...new_res.mangaList);
            $search_results = $search_results;

            setTimeout(end_reached, 500);
        }
    };

    let end_is_visible = true;
    let window_width = 100;
    let window_height = 100;
    let search_input: any;
    const on_keydown = async (event: KeyboardEvent, scroll_selected_into_view: any) => {
        if (event.key == 'a') {
            // await add_tag_button();
            // event.preventDefault();
        } else if (event.key == '/') {
            $selected = 0;
            await tick();
            await scroll_selected_into_view();
            $search_query = '';
            search_input.focus();
            event.preventDefault();
        }
    };

    $: items = $search_results.mangaList.map((e) => {
        return { id: e.id, data: e };
    });

    if ($search_results.mangaList.length == 0) {
        search();
    }
    invoke('tachidesk_get_source_filters', { sourceId: $page.params.src_id }).then(async (e) => {
        console.log(e);
        // https://github.com/Suwayomi/Tachidesk-Server/blob/cde5dc5bfa4ce6cce6d565b41589672a754460c0/server/src/main/kotlin/suwayomi/tachidesk/manga/impl/Search.kt#L137
        // let r = await fetch(`http://localhost:4567/api/v1/source/${$page.params.src_id}/filters`, {
        //     method: 'POST',
        //     body: JSON.stringify({
        //         position: 2,
        //         state: JSON.stringify({
        //             position: 3,
        //             state: true
        //         })
        //     }),
        //     headers: {
        //         'Content-type': 'application/json'
        //     }
        // });
        // console.log(r);
    });
</script>

<cl class={'inputs'}>
    <input bind:value={$search_query} on:input={search} bind:this={search_input} />
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
            $search_results.mangaList.forEach((m) => {
                if (id_set.has(m.id)) {
                    collisions.push(m.id);
                } else {
                    id_set.add(m.id);
                }
            });
            console.log(collisions);
        }}
    >
        {$search_results.mangaList.length} | end visible: {end_is_visible}
    </button>
</cl>

<cl class="scrollable">
    <VirtualScrollable
        bind:items
        item_width={166}
        item_height={243}
        gap={15}
        {end_reached}
        bind:selected={$selected}
        {on_keydown}
        bind:end_is_visible
        let:item_width={width}
        let:item_height
        let:root
        let:item={manga}
        let:selected={s}
    >
        <Card
            {width}
            aspect_ratio={width / item_height}
            selected={s}
            item={manga}
            {root}
            title={manga.title}
            get_img_source={async () => {
                return 'http://0.0.0.0:4567' + manga.thumbnailUrl;
            }}
        >
            <a href="/tachi/manga/{manga.id}">
                read
            </a>
        </Card>
    </VirtualScrollable>
</cl>

<svelte:window bind:innerHeight={window_height} bind:innerWidth={window_width} />

<style>
    a {
        --width: 40px;
        --height: 20px;
        position: absolute;
        z-index: 2;
        float: left;
        height: var(--height);
        width: var(--width);
        top: calc(var(--height) / 2);
        left: calc(var(--height) / 2);
        border: 2px solid;
        border-radius: 8px;
        border-color: var(--color);
        padding: 0px;
        margin: 0px;
        transition: width 0.2s ease;
        text-align: center;
        line-height: calc(var(--height) / 2);
        font-size: 1.57ch;
        font-weight: 700;
        color: #282828;
        background-color: var(--color);
        line-height: 2.5ch;
        text-decoration: none;
    }

    a:hover {
        color: #d8d8d8;
        background-color: #558855af;
    }

    * {
        --input-height: 33px;
    }

    .inputs {
        height: var(--input-height);
    }

    .scrollable {
        --margin: 15px;
    }

    cl {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        overflow: hidden;
        width: calc(100% - var(--margin) * 2);
        height: calc(100% - var(--input-height) - var(--margin));

        margin-left: var(--margin);
        margin-right: var(--margin);
        margin-top: var(--margin);
    }
</style>
