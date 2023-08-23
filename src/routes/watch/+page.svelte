<script lang="ts" context="module">
    import { writable } from 'svelte/store';
    import type { MultiSearchResult } from 'types';
    import { Tmdb } from '$lib/searcher/tmdb.ts';

    let searcher_factory = writable(Tmdb.factory());
    let searcher = writable(Tmdb.new(""));
    let search_query = writable('');
</script>


<script lang="ts">
    import { tick } from 'svelte';
    import Card from './Card.svelte';
    import Scrollable from '$lib/Scrollable.svelte';
    import Virtual from '$lib/Virtual.svelte';
    import Selectable from '$lib/Selectable.svelte';
    import type { RObject } from '$lib/searcher/searcher.ts';

    let items = new Array<RObject<MultiSearchResult>>();

    const search = async () => {
        let s = await $searcher_factory.with_query($search_query);
        if (!s) {
            return;
        }
        $searcher = s;
        items = [];
        await tick();
        setTimeout(end_reached, 500);
    };

    let id_set = new Set();
    let collisions = new Array();
    const end_reached = async () => {
        if (!end_is_visible || !$searcher.has_next_page) {
            return;
        }
        items = await $searcher.next_page();
        console.log($searcher);
        await tick();
        setTimeout(end_reached, 500);
    };

    let end_is_visible = true;
    let window_width = 100;
    let window_height = 100;
    let selected = 0;
    let search_input: any;
    const on_keydown = async (event: KeyboardEvent) => {
        if (event.key == 'a') {
            // await add_tag_button();
            // event.preventDefault();
        } else if (event.key == '/') {
            selected = 0;
            $search_query = '';
            search_input.focus();
            event.preventDefault();
        }
    };

    let item_aspect_ratio = 2 / 3;
</script>

<cl class={'inputs'}>
    <input bind:value={$search_query} on:input={search} bind:this={search_input} />
    <button on:click={search}>Search</button>
    <button
        on:click={() => {
            $searcher_factory.include_adult = !$searcher_factory.include_adult;
            search();
        }}
    >
        include mature: {$searcher.include_adult}
    </button>
    <button
        on:click={() => {
            console.log($searcher.search_results, collisions, id_set);
            let ids = $searcher.search_results.map((e) => e.id);
            console.log(collisions.filter((e) => !ids.includes(e.id)));
        }}
    >
        {$searcher.search_results.length} | end visible: {end_is_visible}
    </button>
</cl>

<cl>
    <Scrollable
        columns={5}
        num_items={$searcher.search_results.length}
        bind:selected
        width={window_width}
        {end_reached}
        {on_keydown}
        bind:end_is_visible
        keyboard_control={true}
        {item_aspect_ratio}
        let:item_width={width}
        let:root
    >
        {#each items as media, i (media.id)}
            <Selectable
                {width}
                {item_aspect_ratio}
                selected={selected == i ||
                    (i == $searcher.search_results.length - 1 &&
                        selected >= $searcher.search_results.length)}
                let:selected={s}
            >
                <Virtual {width} aspect_ratio={item_aspect_ratio} {root}>
                    <Card
                        {width}
                        aspect_ratio={item_aspect_ratio}
                        selected={s}
                        {media}
                        on_click={() => {
                            selected = i;
                        }}
                        {root}
                    />
                </Virtual>
            </Selectable>
        {/each}
    </Scrollable>
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
        overflow: auto;
        width: 100%;
        height: calc(100% - var(--input-height));
    }
</style>
