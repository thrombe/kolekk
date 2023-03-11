<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri';
    import type { ExternalIDs, ListResults, MultiSearchResult } from 'types';
    import Anime from '../mal_fetch/Anime.svelte'; // TODO: generalise this component
    import {search_results, search_query, include_adult, curr_page} from "./media";
    const hasAPI = 'IntersectionObserver' in window;
    import { open } from '@tauri-apps/api/shell';
    import Observer from '$lib/Observer.svelte';
    import { tick } from 'svelte';

    const search_tmdb_multi = async () => {
        $curr_page = 1;
        let res: ListResults<MultiSearchResult> = await invoke('search_tmdb_multi', {
            query: $search_query,
            page: $curr_page,
            includeAdult: $include_adult,
        });
        $search_results = res;

        id_set = new Set();

        await end_reached();
    };

    const open_in_stremio = async (id: number | null, media_type: string) => {
        if (!id) {
            return;
        }

        let ids: ExternalIDs = await invoke("tmdb_get_external_ids", {id: {id: id, media_type: media_type}});
        if (media_type == "tv") {
            media_type = "series";
        }
        if (ids.imdb_id) {
            let href = "stremio:///detail/" + media_type + "/" + ids.imdb_id;
            console.log(href, ids);
            open(href);
        }
    };

    let id_set = new Set();
    const end_reached = async () => {
        await tick();
        if (!end_is_visible) {
            return;
        }

        if ($search_results.page! < $search_results.total_pages!) {
            $curr_page = $curr_page + 1;
            console.log("searched for page", $curr_page);
            let res: ListResults<MultiSearchResult> = await invoke('search_tmdb_multi', {
                query: $search_query,
                page: $curr_page,
                includeAdult: $include_adult,
            });

            // tmdb returns duplicates for some reason :(
            $search_results.results.map(item => id_set.add(item.id));
            res.results = res.results.filter(item => !id_set.has(item.id));

            // $search_results.results = [...$search_results.results, ...res.results];
            $search_results.results.push(...res.results);
            $search_results = $search_results;

            await end_reached();
        }
    };

    let end_is_visible = true;
    let window_width = 100;
    let window_height = 100;
</script>

<input bind:value={$search_query} on:input={search_tmdb_multi}>
<button
    on:click={() => {
        $include_adult = !$include_adult;
        search_tmdb_multi();
    }}
>Search</button>

<cl style="grid: auto-flow / {'1fr '.repeat(4)}">
    {#each $search_results.results as media (media.id)}
            <Anime
                title={media.media_type == "tv"? media.name:media.title}
        <div on:click={() => {open_in_stremio(media.id, media.media_type)}} >
                img_source={media.poster_path? "https://image.tmdb.org/t/p/w200/" + media.poster_path:""}
                lazy={hasAPI}
            />
        </div>
    {/each}

    <!-- observer -->
    <Observer enter_screen={end_reached} bind:visible={end_is_visible}/>
</cl>

<svelte:window bind:innerHeight={window_height} bind:innerWidth={window_width} />

<style>
    dw {
        display: flex;
        flex-direction: column;
    }
    cl {
        display: grid;
        /* grid: auto-flow / 1fr 1fr 1fr 1fr; */
        /* grid-column-start: 2; */
        /* flex-direction: row; */
        /* grid-column: 2; */
    }
</style>
