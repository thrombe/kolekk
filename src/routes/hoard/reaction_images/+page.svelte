<script lang="ts" context="module">
    import { writable } from 'svelte/store';

    let searcher = writable(new Searcher<Image>("Image", 50));
</script>

<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri';
    import type { DragDropPaste, Image, Indexed } from 'types';
    import DataListener from '$lib/DataListener.svelte';
    import { files_to_bytearrays } from '$lib/data_listener';
    import { get_path, Searcher } from '$lib/commands';

    const file_drop = async (e: DragDropPaste<File>) => {
        let images: Image[] = await invoke('get_images', { data: await files_to_bytearrays(e) });
        console.log(images);
        $searcher.add_item(...images.map(img => {
            let searchable: Indexed[] = img.title ? [{ data: img.title, field: "Text" }] : [];
            return { data: img , searchable }
        }));
    };

    $searcher.next_page();

    let tag_name = '';
    const add_tag = async () => {
        if (tag_name == '') {
            return;
        }
        // await invoke('add_tag_to_image', { img: images[0], tag: tag_name });
        // search_images();
    };
    const remove_tag = async () => {
        if (tag_name == '') {
            return;
        }
        // await invoke('remove_tag_from_image', { img: images[0], tag: tag_name });
        // search_images();
    };

    let width = 100;
    let query = '';
    let search_images = () => {
        $searcher.query = query;
        console.log(images, $searcher.search_results.map(e => e.data.data.title), $searcher.search_results.map(e => e.id))
    };
    let images = new Array();

    $: $searcher.query = query;
    $: console.log($searcher.search_results.map(e => e.data.data.title));
    $: images = $searcher.search_results;
    $: console.log(images);
</script>

<DataListener on_receive={file_drop} />

<svelte:window bind:innerWidth={width} />

<cl>
    <buttons>
        <input bind:value={query} on:input={search_images} />
        <button on:click={search_images}>refresh</button>

        <input bind:value={tag_name} />
        <button on:click={add_tag}>add tag</button>
        <button on:click={remove_tag}>remove tag</button>
    </buttons>
    {#each images as img (img.id)}
        <card-div draggable="true" style="height:{width / 5}px; width: {width / 5}px">
            <card-insides draggable="true">
                <image-div>
                    {#await get_path(img.data.data.path)}
                        <img draggable="false" src={''} alt="" />
                    {:then path} 
                        <img draggable="false" src={path} alt="" />
                    {/await}
                </image-div>
                {#if img.data.data.title.length > 0}
                    <span class="title">
                        {img.data.data.title}
                    </span>
                {/if}
                {#if img.data.tags.length > 0}
                    <tags-div>
                        <tag-padding>{'a'}</tag-padding>
                        {#each img.data.tags as tag}
                            <tag>{tag}</tag>
                        {/each}
                    </tags-div>
                {/if}
            </card-insides>
        </card-div>
    {/each}
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
