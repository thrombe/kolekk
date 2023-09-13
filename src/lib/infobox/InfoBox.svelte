<script lang="ts">
    import ImageCard from '$lib/ImageCard.svelte';
    import type { Unique } from '$lib/virtual.ts';
    import Card from '../../routes/hoard/reaction_images/Card.svelte';

    export let item: Unique<unknown, unknown>;
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
</script>

<info-box
    bind:clientHeight={height}
    class="flex flex-col w-full h-full relative"
>
    <abs class="absolute w-full h-full -z-10 block brightness-75 blur-sm scale-105">
        <!-- <Card
            {get_img_source}
            selected={false}
            {item}
            dynamic_thumbnail={true}
            whatever_thumbnail={true}
        /> -->
        <ImageCard
            whatever_thumbnail={true}
            lazy={false}
            dynamic_thumbnail={true}
            {img_source}
        />
    </abs>
    <abs class="absolute -z-10 h-full w-full bg-black bg-opacity-20" />

    <fg class="h-full w-full overflow-y-scroll select-none">
        <slot {item} />
    </fg>
</info-box>
