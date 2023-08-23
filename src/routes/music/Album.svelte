<script lang="ts" context="module">
    import { writable, type Writable } from 'svelte/store';

    let fac: Writable<RFactory<AlbumListResult>> = writable(LastFm.factory());
    let searcher: Writable<RSearcher<AlbumListResult>> = writable(LastFm.new(""));
    let selected = writable(0);
    let search_query = writable('');
</script>

<script lang="ts">
    import type { AlbumListResult } from 'types';
    import Card from '$lib/Card.svelte';
    import type { Unique } from '$lib/virtual.ts';
    import Explorer from '$lib/Explorer.svelte';
    import { LastFm } from '$lib/searcher/lastfm.ts';
    import type { RFactory, RObject, RSearcher } from '$lib/searcher/searcher.ts';
    import LastFmAlbum from '$lib/infobox/LastFmAlbum.svelte';

    let selected_item: Unique<RObject<AlbumListResult>, unknown>;

    let t: AlbumListResult;

    let now_playing: any;
</script>

<cl>
<Explorer
    {t}
    bind:fac={$fac}
    searcher={searcher}
    bind:search_query={$search_query}
    bind:selected_item_index={$selected}
    bind:selected_item
    on_keydown={async (e) => {}}
    on_item_click={async () => {}}
    item_width={150}
    item_height={150}
    let:item
    let:item_width
    let:item_height
    let:selected
    let:root
    let:info_margin
    let:info_width
>
    <Card
        get_img_source={async () => {
            let image = item.image;
            let url = image[image.length - 1].url;
            return url;
        }}
        title={''}
        width={item_width}
        height={item_height}
        {selected}
        {item}
        {root}
    />

    <!-- - [A comprehensive guide to Svelte components with slots - LogRocket Blog](https://blog.logrocket.com/comprehensive-guide-svelte-components-slots/) -->
    <LastFmAlbum
        slot="infobox"
        item={selected_item}
        let:info_width
        let:info_margin
        info_width={info_width}
        info_margin={info_margin}
    />
</Explorer>
</cl>

<playing>
    <input
        type="range"
        on:mousedown={() => {}}
        on:mouseup={() => {}}
        min="0"
        max="100"
    />
</playing>


<style>
    playing {
        display: flex;
        flex-direction: column;
        height: 26px;
        width: 100%;
    }

    playing input {
        width: 100%;
        height: 20px;
        display: block;
        border: 0px;
        margin: 0px;
        padding: 0px;
        margin-top: auto;
        margin-bottom: auto;
    }

    cl {
        display: flex;
        flex-direction: column;
        height: calc(100% - 26px);
    }
</style>
