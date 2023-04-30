<script lang="ts">
    import { get_path } from '$lib/commands';
    import ImageCard from '$lib/ImageCard.svelte';
    import { invoke } from '@tauri-apps/api';
    import type { Meta, Taggable, Image } from 'types';

    export let width: number;
    export let aspect_ratio: number;
    export let image: Meta<Taggable<Image>>;
    
    export let selected = false;
    export let on_click: any = () => {};
    export let root: HTMLElement | null = null;

    let title = image.data.data.title;
    let img_source = '';
    (async () => {
        img_source = await get_path(image.data.data.path);
        console.log(img_source);
    })();

    let ele: HTMLElement;
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
    let bg_color = '#282828';

    const copy = async () => {
        await invoke('copy_image_to_clipboard', { imgPath: image.data.data.path });
    };
</script>

<this-helps-position-the-title>
    <cl bind:this={ele} draggable="true" on:click={on_click} on:keydown={() => {}}>
        <card-div bind:this={insides}>
            <card-insides>
                <ImageCard {img_source} {width} {aspect_ratio} {bg_color} lazy={false} {root} />

                {#if title && title.length > 0}
                    <title-box style={'height: ' + shade_height}>
                        <span style={'white-space: ' + wrap}>{title}</span>
                    </title-box>
                {/if}

                <button class="copy-button" on:click={copy}>
                    <span>copy</span>
                </button>
            </card-insides>
        </card-div>
    </cl>
</this-helps-position-the-title>

<style>
    .copy-button {
        --width: 40px;
        --height: 20px;
        position: absolute;
        z-index: 2;
        float: left;
        height: var(--height);
        width: var(--width);
        top: calc(var(--height) / 2);
        left: calc(var(--height) / 2);
        border: 2px solid;
        border-radius: 8px;
        border-color: var(--color);
        padding: 0px;
        margin: 0px;
        transition: width 0.2s ease;
        text-align: center;
        line-height: calc(var(--height) / 2);
        font-size: 1.37ch;
        font-weight: 700;
        color: #282828;
        background-color: var(--color);
    }

    .copy-button:hover {
        color: #d8d8d8;
        background-color: #558855af;
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
        width: calc(100% - 2 * var(--border) - 6px);
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
