<script lang="ts">
    import { convertFileSrc, invoke } from '@tauri-apps/api/tauri';
    import type { ThumbnailSize } from 'types';
    import Observer from './Observer.svelte';
    const hasAPI = 'IntersectionObserver' in window;

    export let width: number;
    export let aspect_ratio: number;
    export let lazy: boolean;

    export let img_source = '';
    export let bg_color = 'transparent';
    export let scale = '100%';
    export let root: HTMLElement | null = null;

    $: height = width / aspect_ratio;
    let abs: HTMLElement;
    $: if (abs) {
        abs.style.left = (width / 2).toString() + 'px';
        abs.style.top = (height / 2).toString() + 'px';
    }
    let scalable: HTMLElement;
    $: if (scalable) {
        scalable.style.scale = scale;
        scalable.style.setProperty('--bg-color', bg_color);

        scalable.style.width = width.toString() + 'px';
        scalable.style.height = height.toString() + 'px';
    }

    let thumbnail_size: ThumbnailSize;
    $: try_update_thumbnail_size(visible, lazy, width);
    let try_update_thumbnail_size = async (visible: boolean, lazy: boolean, width: number) => {
        if (!visible && lazy && hasAPI) {
            return;
        }
        thumbnail_size = await invoke("get_thumbnail_size", {width});
    };
    $: set_img(img_source, thumbnail_size);

    let lazy_img_src = '';
    let set_img = async (uri: string, thumbnail_size: ThumbnailSize) => {
        if (!thumbnail_size) {
            return;
        }
        console.log(thumbnail_size);
        if (!uri) {
            lazy_img_src = uri;
        } else {
            let src: string = await invoke("image_thumbnail", { uri, thumbnailSize: thumbnail_size });
            lazy_img_src = convertFileSrc(src);
        }
    };

    let visible = false;
</script>

<cl bind:this={scalable}>
    {#if lazy && hasAPI}
        <rel>
            <abs bind:this={abs}>
                <Observer {root} margin={height} bind:visible />
            </abs>
        </rel>
    {/if}

    <image-div style={'background-image: url(' + lazy_img_src + ');'} />
</cl>

<style>
    abs {
        position: absolute;
    }
    rel {
        position: relative;
        z-index: 5;
    }

    image-div {
        width: 100%;
        height: 100%;
        background-size: cover;
        background-position: center;
        background-color: var(--bg-color);
    }

    cl {
        display: flex;
        flex-direction: column;
        align-items: center;
        overflow: hidden;
    }

</style>
