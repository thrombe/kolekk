<script lang="ts">
    import Observer from './Observer.svelte';

    export let title: string | null = null;
    export let tags = new Array();
    export let img_source = '';
    export let width = 200;
    export let lazy = false;
    export let aspect_ratio = 1.0;
    export let selected = false;
    let ele: any;
    $: if (selected && ele) {
        ele.scrollIntoView({ block: 'nearest' });
    };

    let lazy_img_src = '';

    if (!lazy) {
        lazy_img_src = img_source;
    }

    function on_intersect() {
        if (!lazy_img_src) {
            lazy_img_src = img_source;
        }
    }

    $: color = selected? "#558855" : "#885555";
    $: wrap = selected? "normal" : "nowrap";
    $: shade_height = selected? "60%" : "25%";
</script>

<cl bind:this={ele}>
    {#if lazy}
        <Observer enter_screen={on_intersect} />
    {/if}

    <card-div draggable="true" style="height:{width / aspect_ratio}px; width: {width}px">
        <card-insides draggable="true" style={"--color: " + color + ";" + "--color-transparent: " + color + "00;"}>
            <image-div style={"background-image: url(" + lazy_img_src + ");"} />
            {#if title && title.length > 0}
                <title-box style={"height: " + shade_height + ";"} >
                    <span style={"white-space: " + wrap + ";"} >{title}</span>
                </title-box>
            {/if}

            {#if tags.length > 0}
                <tags-div>
                    <tag-padding>{'a'}</tag-padding>
                    {#each tags as tag}
                        <tag>{tag}</tag>
                    {/each}
                </tags-div>
            {/if}
        </card-insides>
    </card-div>
</cl>

<style>
    * {
        --border: 2px;
        --border-radius: 15px;
    }

    image-div {
        width: 100%;
        height: 100%;
        overflow: hidden;
        background-size: cover;
        background-color: var(--color);
    }

    title-box {
        position: absolute;
        bottom: 0;
        width: calc(100% - 3 * var(--border));
        background-image: linear-gradient(to top, var(--color), var(--color-transparent));
        border-radius: var(--border-radius);
        margin-bottom: 3px;
        overflow: hidden;
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

    card-div {
        overflow: hidden;
    }

    card-insides {
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

    tags-div {
        display: flex;
        width: calc(100%);
        height: min-content;
        /* overflow-x: auto; */
        /* overflow-y: hidden; */

        padding: 0;
        margin: 0;
        background-color: #ffffff;
        height: min-content;

        /* flex-wrap: wrap; */
        /* overflow: hidden; */
        /* justify-content:space-evenly; */
    }

    tags-div::-webkit-scrollbar {
        /* background-color: #630424; */
        /* display: none; */
        width: 0;
        background: transparent;
        height: 0;
    }

    tags-div tag {
        font-size: 1.17ch;
        font-weight: 700;
        /* height: 3.0ch; */

        padding-left: 3px;
        padding-right: 3px;
        padding-bottom: 0.57ch;

        background-color: #5b931b;
        color: #630424;
        border-radius: 3px;
        width: min-content;
    }

    tags-div tag + tag {
        margin-left: 1.5%;
    }

    tag-padding {
        /* width: 12px; */
        height: 1px;
        /* height: 3.0ch; */
        color: transparent;
    }

    cl {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        overflow: auto;
        width: 100%;
        /* height: 100%; */
    }
</style>
