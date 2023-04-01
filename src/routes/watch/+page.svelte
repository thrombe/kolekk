<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri';
    import type { ExternalIDs, ListResults, MultiSearchResult } from 'types';
    import { search_results, search_query, include_adult } from './media';
    const hasAPI = 'IntersectionObserver' in window;
    import { open } from '@tauri-apps/api/shell';
    import Observer from '$lib/Observer.svelte';
    import ImageCard from '$lib/ImageCard.svelte';
    import { tick } from 'svelte';
    import { fastScroll } from '$lib/fast_scroll';

    const open_in_stremio = async (id: number | null, media_type: string) => {
        if (!id) {
            return;
        }

        let ids: ExternalIDs = await invoke('tmdb_get_external_ids', {
            id: { id: id, media_type: media_type }
        });
        if (media_type == 'tv') {
            media_type = 'series';
        }
        if (ids.imdb_id) {
            let href = 'stremio:///detail/' + media_type + '/' + ids.imdb_id;
            console.log(href, ids);
            open(href);
        }
    };

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
            if (selected + elements_per_row < $search_results.results.length) {
                selected += elements_per_row;
            }
        } else if (event.key == 'a') {
            // await add_tag_button();
            // event.preventDefault();
        } else if (event.key == '/') {
            $search_query = "";
            search_input.focus();
            event.preventDefault();
        } else if (false && event.key == "Escape") {
            // event.preventDefault();
        }

        if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].indexOf(event.key) > -1) {
            event.preventDefault();
        }
    };
</script>

<cl class={"inputs"}>
    <input bind:value={$search_query} on:input={search} bind:this={search_input}/>
    <button on:click={search}>Search</button>
    <button
        on:click={() => {
            $include_adult = !$include_adult;
            search();
        }}>include mature: {$include_adult}</button
    >
    <button on:click={() =>{
        console.log($search_results.results, collisions);
        let ids = $search_results.results.map(e => e.id);
        console.log(collisions.filter(e => !ids.includes(e.id)));
    }} >{($search_results).results.length}</button>
</cl>

<cl style="" use:fastScroll >
    {#each $search_results.results as media, i (media.id)}
        <div>
            <ImageCard
                title={media.media_type == 'tv' ? media.name : media.title}
                img_source={media.poster_path
                    ? 'https://image.tmdb.org/t/p/w200/' + media.poster_path
                    : ''}
                lazy={hasAPI}
                width={window_width / 5}
                aspect_ratio={2 / 3}
                selected={selected == i}
            />
            <button
                class="nice_button"
                on:click={() => {
                    open_in_stremio(media.id, media.media_type);
                }}
            />
        </div>
    {/each}

    <!-- observer -->
    <Observer enter_screen={end_reached} bind:visible={end_is_visible} />
</cl>

<svelte:window bind:innerHeight={window_height} bind:innerWidth={window_width} on:keydown={on_keydown} />

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
    cl div {
        position: relative;
    }

    .nice_button {
        --width: 30px;
        --height: 20px;
        position: absolute;
        z-index: 2;
        float: left;
        height: var(--height);
        width: var(--width);
        top: calc(9% - var(--height) / 2);
        left: calc(16% - var(--width) / 2);
        background-color: #ffffffaf;
        border: 2px solid;
        border-radius: 8px;
        border-color: #885555;
        padding: 0px;
        margin: -px;
        transition: width 0.2s ease;
    }

    .nice_button:hover {
        background-color: #558855af;
        width: calc(2 * var(--width));
        transition: width 0.2s ease;
    }
</style>
