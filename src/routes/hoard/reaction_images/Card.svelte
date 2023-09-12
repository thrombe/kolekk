<script lang="ts">
    import { invoke } from "@tauri-apps/api";
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import type { ThumbnailSize } from "types";

    export let item: any;
    export let get_img_source: () => Promise<string>;

    export let selected = false;
    export let on_click: (() => void) | (() => Promise<void>) = () => {};
    export let whatever_thumbnail: boolean = false;
    export let dynamic_thumbnail: boolean;
    export let image_div: HTMLElement | null = null;


    let img_source = '';
    (async () => {
        img_source = await get_img_source();
    })();


    let thumbnail_size: ThumbnailSize;
    $: if (image_div) {
        try_update_thumbnail_size(image_div.clientWidth);
    }
    let try_update_thumbnail_size = async (width: number) => {
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
</script>

<div class="h-full w-full rounded-lg overflow-hidden select-none
    { selected ? 'border-green-700 border-opacity-70 border-4 scale-110' : '' }
    "
>
    <image-div bind:this={image_div} class='block bg-center w-full h-full bg-cover' style={'background-image: url(' + lazy_img_src + ');'} />
</div>
