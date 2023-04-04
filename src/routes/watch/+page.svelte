<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri';
    import type { ListResults, MultiSearchResult } from 'types';
    import { search_results, search_query, include_adult } from './media';
    import Observer from '$lib/Observer.svelte';
    import { tick } from 'svelte';
    import Card from './Card.svelte';

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
    let elements_per_row = 5;
    let search_input: any;
    const on_keydown = async (event: KeyboardEvent) => {
        if (document.activeElement?.tagName == 'INPUT') {
            if (event.key == 'Escape') {
                (document.activeElement as HTMLElement).blur();
            }
            return;
        }

        if (event.key == 'ArrowLeft') {
            if (selected - 1 >= 0) {
                selected -= 1;
            }
        } else if (event.key == 'ArrowRight') {
            if (selected + 1 < $search_results.results.length) {
                selected += 1;
            }
        } else if (event.key == 'ArrowUp') {
            if (selected - elements_per_row >= 0) {
                selected -= elements_per_row;
            }
        } else if (event.key == 'ArrowDown') {
            if (selected + 1 < $search_results.results.length) {
                selected += elements_per_row;
            }
        } else if (event.key == 'a') {
            // await add_tag_button();
            // event.preventDefault();
        } else if (event.key == '/') {
            selected = 0;
            $search_query = '';
            search_input.focus();
            event.preventDefault();
        } else if (false && event.key == 'Escape') {
            // event.preventDefault();
        }

        if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].indexOf(event.key) > -1) {
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
    {#each $search_results.results as media, i (media.id)}
        <Card
            width={window_width / 5}
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

    <!-- observer -->
    <Observer enter_screen={end_reached} bind:visible={end_is_visible} />
</cl>

<svelte:window
    bind:innerHeight={window_height}
    bind:innerWidth={window_width}
    on:keydown={on_keydown}
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
