<script lang="ts">
    import ImageCard from '$lib/ImageCard.svelte';
    import { invoke } from '@tauri-apps/api/tauri';
    import type { ExternalIDs, MultiSearchResult } from 'types';
    import { open } from '@tauri-apps/api/shell';

    export let width = 200;
    export let aspect_ratio = 1.0;
    export let selected = false;
    export let bg_color = 'transparent';
    export let media: MultiSearchResult;
    export let on_click: any = () => {};

    let title = media.media_type == 'tv' ? media.name : media.title;
    let img_source = media.poster_path
        ? 'https://image.tmdb.org/t/p/w200/' + media.poster_path
        : '';

    let ele: HTMLElement;
    $: if (selected && ele) {
        ele.scrollIntoView({ block: 'nearest' });
    }
    $: if (ele) {
        let color = selected ? '#558855' : '#885555';
        ele.style.setProperty('--color', color);
        ele.style.setProperty('--color-transparent', color + '00');
        ele.style.setProperty('--border', '2px');
        ele.style.setProperty('--border-radius', '15px');
    }
    let insides: HTMLElement;
    $: if (insides) {
        insides.style.width = width.toString() + 'px';
        let height = width / aspect_ratio;
        insides.style.height = height.toString() + 'px';
    }

    $: wrap = selected ? 'normal' : 'nowrap';
    $: shade_height = selected ? '60%' : '25%';

    let external_ids: ExternalIDs;
    const open_in_stremio = async () => {
        if (!media.id) {
            return;
        }

        if (!external_ids) {
            external_ids = await invoke('tmdb_get_external_ids', {
                id: { id: media.id, media_type: media.media_type }
            });
        }

        let media_type: string = media.media_type;
        if (media_type == 'tv') {
            media_type = 'series';
        }

        if (external_ids.imdb_id) {
            let href = 'stremio:///detail/' + media.media_type + '/' + external_ids.imdb_id;
            console.log(href, external_ids);
            open(href);
        }
    };
</script>

<this-helps-position-the-title>
    <cl bind:this={ele} draggable="true" on:click={on_click} on:keydown={() => {}}>
        <card-div bind:this={insides}>
            <card-insides>
                <ImageCard {img_source} {width} {aspect_ratio} {bg_color} lazy={true} />

                {#if title && title.length > 0}
                    <title-box style={'height: ' + shade_height}>
                        <span style={'white-space: ' + wrap}>{title}</span>
                    </title-box>
                {/if}

                <button class="stremio-button" on:click={open_in_stremio}>
                    <span>stremio</span>
                </button>
            </card-insides>
        </card-div>
    </cl>
</this-helps-position-the-title>

<style>
    .stremio-button {
        --width: 20px;
        --height: 20px;
        position: absolute;
        z-index: 2;
        float: left;
        height: var(--height);
        width: var(--width);
        top: calc(var(--height) / 2);
        left: calc(var(--width) / 2);
        border: 2px solid;
        border-radius: 8px;
        border-color: var(--color);
        padding: 0px;
        margin: 0px;
        transition: width 0.2s ease;
        text-align: center;
        line-height: calc(var(--height) / 2);
        color: transparent;
    }

    .stremio-button span {
        font-size: 1.87ch;
        font-weight: 700;
    }

    .stremio-button:hover {
        background-color: #558855af;
        width: calc(2.9 * var(--width));
        transition: width 0.2s ease;
        color: #d8d8d8;
    }

    this-helps-position-the-title {
        position: relative;
    }

    title-box {
        position: absolute;
        bottom: 0;
        width: calc(100% - 3 * var(--border));
        background-image: linear-gradient(to top, var(--color), var(--color-transparent));
        border-radius: var(--border-radius);
        margin-bottom: 3px;
        overflow: hidden;
        left: var(--border);
    }

    title-box span {
        position: absolute;
        bottom: 0;
        width: calc(100% - 6 * var(--border));
        padding-left: calc(3 * var(--border));
        padding-right: calc(3 * var(--border));
        border-radius: var(--border-radius);
        text-align: center;
        text-overflow: ellipsis;
        overflow: hidden;
        font-size: 1.37ch;
        padding-bottom: 0.456ch;
        font-weight: 550;
        color: #cccccc;
    }

    card-insides {
        height: calc(100% - 2 * var(--border) - 6px);

        display: flex;
        flex-direction: column;
        align-items: center;
        color: var(--color);
        margin-left: 3px;
        margin-right: 3px;
        margin-top: 3px;
        margin-bottom: 3px;

        border: var(--border) solid;
        border-radius: var(--border-radius);

        overflow: hidden;
    }

    cl {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        overflow: auto;
        width: 100%;
    }
</style>
