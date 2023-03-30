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

    const search_tmdb_multi = async (query: string, page: number, include_adult: Boolean): Promise<ListResults<MultiSearchResult>> => {
        console.log('searched for page', page, 'with query', query);
        return await invoke('search_tmdb_multi', {
            query: query,
            page: page,
            includeAdult: include_adult
        })
    };

    const search = async () => {
        if ($search_query == "") {
            $search_results.results.length = 0;
            $search_results.page = null;
        } else {
            $search_results = await search_tmdb_multi($search_query, 1, $include_adult);

            await end_reached();
        }
    };

    let id_set = new Set();
    const end_reached = async () => {
        await tick();

        if (!$search_results.page) {
            id_set = new Set();
            return;
        }
        if (!end_is_visible) {
            return;
        }

        if ($search_results.page! < $search_results.total_pages!) {
            let old_res = $search_results;
            let new_res = await search_tmdb_multi($search_query, $search_results.page! + 1, $include_adult);

            // tmdb returns duplicates for some reason :(
            old_res.results.map((item) => id_set.add(item.id));
            new_res.results = new_res.results.filter((item) => !id_set.has(item.id));

            old_res.results.push(...new_res.results);
            new_res.results = old_res.results;
            $search_results = new_res;

            await end_reached();
        }
    };

    let end_is_visible = true;
    let window_width = 100;
    let window_height = 100;
</script>

<input bind:value={$search_query} on:input={search} />
<button
    on:click={() => {
        $include_adult = !$include_adult;
        search();
    }}>Search</button
>

<cl style="" use:fastScroll>
    {#each $search_results.results as media (media.id)}
        <div
            on:click={() => {
                open_in_stremio(media.id, media.media_type);
            }}
        >
            <ImageCard
                title={media.media_type == 'tv' ? media.name : media.title}
                img_source={media.poster_path
                    ? 'https://image.tmdb.org/t/p/w200/' + media.poster_path
                    : ''}
                lazy={hasAPI}
                width={window_width / 5}
                aspect_ratio={2 / 3}
            />
        </div>
    {/each}

    <!-- observer -->
    <Observer enter_screen={end_reached} bind:visible={end_is_visible} />
</cl>

<svelte:window bind:innerHeight={window_height} bind:innerWidth={window_width} />

<style>
    cl {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        overflow: auto;
        width: 100%;
    }
</style>
