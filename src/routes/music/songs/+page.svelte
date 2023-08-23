<script lang="ts" context="module">
    import { writable, type Writable } from 'svelte/store';

    let fac: Writable<RFactory<MusicResponsiveListItem>> = writable(SongTube.factory(tube));
    let searcher: Writable<RSearcher<MusicResponsiveListItem>> = writable(SongTube.new("", tube));
    let selected = writable(0);
    let search_query = writable('');
</script>

<script lang="ts">
    import Card from '$lib/Card.svelte';
    import type { Unique } from '$lib/virtual';
    import Explorer from '$lib/Explorer.svelte';
    import { SongTube, type MusicResponsiveListItem, type RFactory, type RObject, type RSearcher } from '$lib/searcher/searcher';
    import { tube } from '$lib/searcher/Tube.svelte';

    let selected_item: Unique<RObject<MusicResponsiveListItem>, unknown>;

    let t: MusicResponsiveListItem;

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
            let image = item.thumbnails[0];
            return image.url;
        }}
        title={''}
        width={item_width}
        height={item_height}
        {selected}
        {item}
        {root}
    />

    <!-- <LastFmAlbum
        slot="infobox"
        item={selected_item}
        {info_width}
        {info_margin}
    /> -->
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

