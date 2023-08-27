<script lang="ts">
    import { convertFileSrc, invoke } from '@tauri-apps/api/tauri';
    import type { ThumbnailSize } from 'types';
    import Observer from './Observer.svelte';
    const hasAPI = 'IntersectionObserver' in window;

    export let width: number;
    export let height: number;
    export let lazy: boolean;
    export let dynamic_thumbnail: boolean;

    export let img_source = '';
    export let bg_color = 'transparent';
    export let scale = '100%';
    export let root: HTMLElement | null = null;
    export let whatever_thumbnail: boolean = false;

    let abs: HTMLElement;
    $: if (abs) {
        abs.style.left = (width / 2).toString() + 'px';
        abs.style.top = (height / 2).toString() + 'px';
    }
    let scalable: HTMLElement;
    $: if (scalable) {
        scalable.style.scale = scale;
        scalable.style.setProperty('--bg-color', bg_color);

        // scalable.style.width = width.toString() + 'px';
        //scalable.style.height = height.toString() + 'px';
    }

    let thumbnail_size: ThumbnailSize;
    $: try_update_thumbnail_size(visible, lazy, width);
    let try_update_thumbnail_size = async (visible: boolean, lazy: boolean, width: number) => {
        if (!visible && lazy && hasAPI) {
            return;
        }
        thumbnail_size = await invoke("get_thumbnail_size", {width});
    };
    $: set_img(img_source, thumbnail_size, dynamic_thumbnail);

    let lazy_img_src = '';
    let set_img = async (uri: string, thumbnail_size: ThumbnailSize, dynamic_thumbnail: boolean) => {
        if (!thumbnail_size) {
            return;
        }
        console.log(thumbnail_size);
        if (!dynamic_thumbnail) {
            let src: string = await invoke("image_thumbnail", { uri, thumbnailSize: 'original' });
            lazy_img_src = convertFileSrc(src);
        } else if (!uri) {
            lazy_img_src = uri;
        } else {
            if (whatever_thumbnail) {
                let src: string | null = await invoke("whatever_thumbnail", { uri });
                if (src) {
                    lazy_img_src = convertFileSrc(src);
                } else {
                    let src: string = await invoke("image_thumbnail", { uri, thumbnailSize: 'original' });
                    lazy_img_src = convertFileSrc(src);
                }
            } else {
                let src: string = await invoke("image_thumbnail", { uri, thumbnailSize: thumbnail_size });
                lazy_img_src = convertFileSrc(src);
            }
        }
    };

    let visible = false;
</script>

<cl bind:this={scalable} class='flex flex-col items-center overflow-hidden w-full h-full'>
    {#if lazy && hasAPI}
        <rel class='relative z-10'>
            <abs bind:this={abs} class='absolute'>
                <Observer {root} margin={height} bind:visible />
            </abs>
        </rel>
    {/if}

    <image-div class='bg-center w-full h-full bg-cover' style={'background-image: url(' + lazy_img_src + ');'} />
</cl>

