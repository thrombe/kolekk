<script lang="ts">
    import Observer from './Observer.svelte';

    export let title: string | null = null;
    export let tags = new Array();
    export let img_source = '';
    export let width = 200;
    export let lazy = false;
    export let aspect_ratio = 1.0;

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

<cl>
    {#if lazy}
        <Observer enter_screen={on_intersect} />
    {/if}

    <card-div draggable="true" style="height:{width / aspect_ratio}px; width: {width}px">
        <card-insides draggable="true">
            <image-div>
                <img draggable="false" src={lazy_img_src} alt="" />
            </image-div>
            {#if title && title.length > 0}
                <span class="title">
                    {title}
                </span>
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
    image-div {
        width: 100%;
        /* height: 100%; */
        max-height: 100%;
        height: calc(100%);
        overflow: hidden;
    }

    image-div img {
        /* border-radius: 15px; */
        /* border-bottom-left-radius: 0px; */
        /* border-bottom-right-radius: 0px; */

        width: calc(100%);
        height: calc(100%);
        object-fit: cover;
    }

    .title {
        font-size: 1.17ch;
        padding-bottom: 0.556ch;
        font-weight: 500;
        width: calc(100%);
        height: min-content;

        text-align: center;
        text-overflow: ellipsis;
        overflow: hidden;
        white-space: nowrap;

        color: aquamarine;
        /* background-color: blue; */
    }

    card-div {
        overflow: hidden;
    }

    card-insides {
        /* width: 100%; */
        /* height: calc(100% - 2px - 6px); */
        height: calc(100% - 0px - 5px);

        display: flex;
        flex-direction: column;
        align-items: center;
        color: rgb(179, 179, 179);
        margin-left: 3px;
        margin-right: 3px;
        margin-top: 3px;
        margin-bottom: 3px;

        background-color: blueviolet;

        border: 1px solid;
        border-radius: 15px;
        border-color: red;

        overflow: hidden;
    }

    card-div + card-div {
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

    buttons {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        width: 100%;
        height: 33px;
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
    rw {
        display: flex;
        flex-direction: column;
        width: 100%;
    }
</style>
