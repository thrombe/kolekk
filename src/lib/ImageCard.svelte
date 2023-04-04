<script lang="ts">
    import Observer from './Observer.svelte';
    const hasAPI = 'IntersectionObserver' in window;

    export let img_source = '';
    export let width = 200;
    export let lazy = false;
    export let aspect_ratio = 1.0;
    export let bg_color = 'transparent';

    let ele: HTMLElement;
    let insides: HTMLElement;
    $: if (ele) {
        ele.style.setProperty('--bg-color', bg_color);
    }
    $: if (insides) {
        insides.style.width = width.toString() + 'px';
        let height = width / aspect_ratio;
        insides.style.height = height.toString() + 'px';
    }

    let lazy_img_src = '';

    if (!lazy) {
        lazy_img_src = img_source;
    }

    function on_intersect() {
        if (!lazy_img_src) {
            lazy_img_src = img_source;
        }
    }
</script>

<cl bind:this={ele}>
    {#if lazy && hasAPI}
        <Observer enter_screen={on_intersect} />
    {/if}

    <card-div bind:this={insides}>
        <card-insides>
            <image-div style={'background-image: url(' + lazy_img_src + ');'} />
        </card-insides>
    </card-div>
</cl>

<style>
    image-div {
        width: 100%;
        height: 100%;
        background-size: cover;
        background-position: center;
        background-color: var(--bg-color);
    }

    card-insides {
        height: 100%;
        display: flex;
        flex-direction: column;
        align-items: center;
        overflow: hidden;
    }

    cl {
        display: flex;
        width: 100%;
    }
</style>
