<script lang="ts">
    import { convertFileSrc, invoke } from '@tauri-apps/api/tauri';
    import Observer from './Observer.svelte';
    const hasAPI = 'IntersectionObserver' in window;

    export let width: number;
    export let aspect_ratio: number;
    export let lazy: boolean;

    export let img_source = '';
    export let bg_color = 'transparent';
    export let scale = '100%';
    export let root: HTMLElement | null = null;

    let insides: HTMLElement;
    $: height = width / aspect_ratio;
    $: if (insides) {
        insides.style.setProperty('--bg-color', bg_color);

        insides.style.width = width.toString() + 'px';
        insides.style.height = height.toString() + 'px';
    }
    let abs: HTMLElement;
    $: if (abs) {
        abs.style.left = (width / 2).toString() + 'px';
        abs.style.top = (height / 2).toString() + 'px';
    }
    let scalable: HTMLElement;
    $: if (scalable) {
        scalable.style.scale = scale;
    }

    let lazy_img_src = '';
    let set_img = async (uri: string, width: number) => {
        if (!uri) {
            lazy_img_src = img_source;
        } else {
            let src: string = await invoke("image_thumbnail", { uri, width });
            lazy_img_src = convertFileSrc(src);
        }
    };
    $: if (!lazy) {
        set_img(img_source, width);
    }

    function on_intersect() {
        if (!lazy_img_src) {
            set_img(img_source, width);
        }
    }
</script>

<cl bind:this={scalable}>
    {#if lazy && hasAPI}
        <rel>
            <abs bind:this={abs}>
                <Observer enter_screen={on_intersect} {root} margin={height} />
            </abs>
        </rel>
    {/if}

    <card-div bind:this={insides}>
        <card-insides>
            <image-div style={'background-image: url(' + lazy_img_src + ');'} />
        </card-insides>
    </card-div>
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

    card-insides {
        height: 100%;
        width: 100%;
        display: flex;
        flex-direction: column;
        align-items: center;
        overflow: hidden;
    }

    cl {
        display: flex;
    }
</style>
