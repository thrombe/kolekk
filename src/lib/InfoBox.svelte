<script lang="ts">
    import ImageCard from './ImageCard.svelte';
    import type { Unique } from './virtual';

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
>
    <rel>
        <abs>
            <krop>
                <ImageCard
                    width={width - border_radius * 2}
                    aspect_ratio={(width - border_radius * 2) / (height - border_radius * 2)}
                    lazy={false}
                    {img_source}
                />
            </krop>
        </abs>
        <abs class="bg" />
    </rel>

    <fg>
        <slot {item} />
    </fg>
</info-box>

<style>
    info-box {
        width: 100%;
        height: 100%;

        display: flex;
        flex-direction: column;
    }

    rel {
        position: relative;
    }

    abs {
        width: var(--width);
        height: var(--height);
        position: absolute;
        top: 0px;
        left: 0px;

        border: var(--border-radius) solid;
        border-color: #aa99aa;
    }

    krop {
        display: block;
        -webkit-filter: brightness(50%) blur(8px);
        overflow: hidden;
        z-index: -2;
    }

    .bg {
        z-index: 1;
        background-color: #22222288;
    }

    fg {
        z-index: 2;
        width: var(--width);
        height: var(--height);
        margin: var(--border-radius);
    }
</style>
