<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri';
    import type { ListResults, MultiSearchResult } from 'types';
    import { search_results, search_query, include_adult } from './media';
    import { tick } from 'svelte';
    import Card from './Card.svelte';
    import Scrollable from '$lib/Scrollable.svelte';

    const search_tmdb_multi = async (
        query: string,
        page: number,
        include_adult: Boolean
    ): Promise<ListResults<MultiSearchResult>> => {
        console.log('searched for page', page, 'with query', query);
        return await invoke('search_tmdb_multi', {
            query: query,
            page: page,
            includeAdult: include_adult
        });
    };

    const search = async () => {
        if ($search_query == '') {
            $search_results.results.length = 0;
            $search_results.page = null;
        } else {
            $search_results = await search_tmdb_multi($search_query, 1, $include_adult);
            id_set = new Set();
            collisions = new Array();
            $search_results.results = $search_results.results.filter((item) => {
                if (id_set.has(item.id)) {
                    collisions.push(item);
                    return false;
                } else {
                    id_set.add(item.id);
                    return true;
                }
            });

            setTimeout(end_reached, 500);
        }
    };

    let id_set = new Set();
    let collisions = new Array();
    const end_reached = async () => {
        await tick();

        if (!end_is_visible) {
            return;
        }

        if ($search_results.page! < $search_results.total_pages!) {
            let new_res = await search_tmdb_multi(
                $search_query,
                $search_results.page! + 1,
                $include_adult
            );

            // tmdb returns duplicates for some reason :(
            new_res.results = new_res.results.filter((item) => {
                if (id_set.has(item.id)) {
                    collisions.push(item);
                    return false;
                } else {
                    id_set.add(item.id);
                    return true;
                }
            });

            $search_results.results.push(...new_res.results);
            new_res.results = $search_results.results;
            $search_results = new_res;

            setTimeout(end_reached, 500);
        }
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
</script>

<cl class={'inputs'}>
    <input bind:value={$search_query} on:input={search} bind:this={search_input} />
    <button on:click={search}>Search</button>
    <button
        on:click={() => {
            $include_adult = !$include_adult;
            search();
        }}
    >
        include mature: {$include_adult}
    </button>
    <button
        on:click={() => {
            console.log($search_results.results, collisions, id_set);
            let ids = $search_results.results.map((e) => e.id);
            console.log(collisions.filter((e) => !ids.includes(e.id)));
        }}
    >
        {$search_results.results.length} | end visible: {end_is_visible}
    </button>
</cl>

<cl>
    <Scrollable
        columns={5}
        num_items={$search_results.results.length}
        bind:selected={selected}
        width={window_width}
        end_reached={end_reached}
        on_keydown={on_keydown}
        bind:end_is_visible={end_is_visible}
        keyboard_control={true}

        let:item_width={width}
    >
        {#each $search_results.results as media, i (media.id)}
            <Card
                width={width}
                aspect_ratio={2 / 3}
                selected={selected == i ||
                    (i == $search_results.results.length - 1 &&
                        selected >= $search_results.results.length)}
                {media}
                on_click={() => {
                    selected = i;
                }}
            />
        {/each}
    </Scrollable>
</cl>

<svelte:window
    bind:innerHeight={window_height}
    bind:innerWidth={window_width}
/>

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
