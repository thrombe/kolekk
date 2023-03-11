<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri';
    import type { ExternalIDs, ListResults, MultiSearchResult } from 'types';
    import Anime from '../mal_fetch/Anime.svelte'; // TODO: generalise this component
    import {search_results, search_query, include_adult} from "./media";
    const hasAPI = 'IntersectionObserver' in window;
    import { open } from '@tauri-apps/api/shell';

    const search_tmdb_multi = async () => {
        let res: ListResults<MultiSearchResult> = await invoke('search_tmdb_multi', {
            query: $search_query,
            page: 1,
            includeAdult: $include_adult,
        });
        console.log(res);
        $search_results = res;
    };

    let window_width = 100;
    let window_height = 100;

    // $: calculated_width = window_width / 160;

    const open_in_stremio = async (id: number, media_type: string) => {
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
        <div style="" on:click={() => {open_in_stremio(media.id, media.media_type)}} >
            <Anime
                title={media.media_type == "tv"? media.name:media.title}
                img_source={media.poster_path? "https://image.tmdb.org/t/p/w200/" + media.poster_path:""}
                lazy={hasAPI}
            />
        </div>
    {/each}
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
