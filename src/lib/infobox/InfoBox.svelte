<script lang="ts">
    import ImageCard from '$lib/ImageCard.svelte';
    import type { Unique } from '$lib/virtual.ts';

    export let item: Unique<unknown, unknown>;
    export let width: number;
    export let get_img_source = async (): Promise<string> => {
        return '';
    };

    let img_source = '';
    $: if (item.id) {
        (async () => {
            img_source = await get_img_source();
        })();
    }

    let height = 100;
    let border_radius = 1;
</script>

<info-box
    draggable="true"
    bind:clientHeight={height}
    style="--border-radius: {border_radius}px;
        --width: {width - border_radius * 2}px;
        --height: {height - border_radius * 2}px;"
    class='flex flex-col w-full h-full relative'
>
        <abs class='absolute w-full h-full'>
            <krop class='block overflow-hidden -z-10 blur-sm h-full w-full brightness-50'>
                <ImageCard
                    whatever_thumbnail={false}
                    width={width - border_radius * 2}
                    height={height - border_radius * 2}
                    lazy={false}
                    dynamic_thumbnail={false}
                    {img_source}
                />
            </krop>
        </abs>
        <abs class="bg absolute h-full w-full" />

    <fg class='z-10 h-full w-full overflow-y-scroll'>
        <slot {item} />
    </fg>
</info-box>

<style>
    .bg {
        z-index: 1;
        background-color: #22222288;
    }
</style>
