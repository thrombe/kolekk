<script lang="ts" context="module">
    import { writable } from 'svelte/store';

    let searcher_factory = writable(TachiMangaSearch.factory(""));
    let searcher = writable<RSearcher<Manga>>(TachiMangaSearch.new("", ""));
    let search_query = writable('');
    let selected = writable(0);
</script>

<script lang="ts">
    import { tick } from 'svelte';
    import Card from '$lib/Card.svelte';
    import { invoke } from '@tauri-apps/api/tauri';
    import { page } from '$app/stores';
    import VirtualScrollable from '$lib/VirtualScrollable.svelte';
    import { TachiMangaSearch } from '$lib/searcher/tachi.ts';
    import type { Unique } from '$lib/virtual.ts';
    import type { RObject, RSearcher } from '$lib/searcher/searcher.ts';
    import type { Manga } from 'types';

    $searcher_factory.source = $page.params.src_id;

    let items = new Array<Unique<RObject<Manga>, number>>();

    let include_adult = false;
    const search = async () => {
        let s = await $searcher_factory.with_query($search_query);
        if (!s) {
            return;
        }
        $searcher = s;
        items = []
        await tick();
        setTimeout(end_reached, 500);
    };
    search();

    const end_reached = async () => {
        if (!end_is_visible || !$searcher.has_next_page) {
            return;
        }
        let r = await $searcher.next_page();
        items = r.map(e => {
            return {
                id: Number(e.id),
                data: e,
            };
        })
        await tick();
        setTimeout(end_reached, 500);
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
            console.log($searcher, end_is_visible);
        }}
    >
        {$searcher.search_results.length} | end visible: {end_is_visible}
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
            height={item_height}
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
