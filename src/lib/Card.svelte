<script lang="ts">
    import ImageCard from '$lib/ImageCard.svelte';

    export let width: number;
    export let height: number;
    export let item: any;
    export let title: string;
    export let get_img_source: () => Promise<string>;
    export let border_radius = '15px';

    export let selected = false;
    export let img_scale = '100%';
    export let on_click: (() => void) | (() => Promise<void>) = () => {};
    export let root: HTMLElement | null = null;

    let img_source = '';
    (async () => {
        img_source = await get_img_source();
    })();

    let ele: HTMLElement;
    $: if (ele) {
        let color = selected ? '#558855' : '#885555';
        ele.style.setProperty('--color', color);
        ele.style.setProperty('--color-transparent', color + '00');
        ele.style.setProperty('--border', '2px');
        ele.style.setProperty('--border-radius', border_radius);

        let w = width - 4;
        ele.style.width = w.toString() + 'px';
        let h = height - 0;
        ele.style.height = h.toString() + 'px';
    }

    $: wrap = selected ? 'normal' : 'nowrap';
    $: shade_height = selected ? '60%' : '25%';
    let bg_color = '#282828';
</script>

<this-helps-position-the-title>
    <cl bind:this={ele} draggable="true" on:click={on_click} on:keydown={() => {}}>
        <ImageCard
            scale={img_scale}
            {img_source}
            width={width - 4}
            {height}
            {bg_color}
            lazy={false}
            dynamic_thumbnail={true}
            {root}
        />

        {#if title && title.length > 0}
            <title-box style={'height: ' + shade_height}>
                <span style={'white-space: ' + wrap}>{title}</span>
            </title-box>
        {/if}

        <slot />
    </cl>
</this-helps-position-the-title>

<style>
    this-helps-position-the-title {
        position: relative;
    }

    title-box {
        position: absolute;
        bottom: 0;
        width: calc(100% - 1 * var(--border));
        background-image: linear-gradient(to top, var(--color), var(--color-transparent));
        border-radius: var(--border-radius);
        overflow: hidden;
    }

    title-box span {
        position: absolute;
        bottom: 0;
        width: calc(100% - 6 * var(--border));
        padding-left: calc(2 * var(--border));
        padding-right: calc(2 * var(--border));
        border-radius: var(--border-radius);
        text-align: center;
        text-overflow: ellipsis;
        overflow: hidden;
        font-size: 1.37ch;
        padding-bottom: 0.456ch;
        font-weight: 550;
        color: #cccccc;
    }

    cl {
        display: flex;
        flex-direction: column;
        align-items: center;
        color: var(--color);

        border: var(--border) solid;
        border-radius: var(--border-radius);

        overflow: hidden;
    }
</style>
