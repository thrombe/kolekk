<script lang="ts">
    import { convertFileSrc, invoke } from '@tauri-apps/api/tauri';
    import type { Image } from '../../../rs_bindings';
    import DataListener from '../../../DataListener.svelte';

    let images = new Array<Image>();

    const file_drop = async (e: any) => {
        console.log(e);
        for (let path of e.payload) {
            console.log(e.payload);
            console.log(path);
            console.log(convertFileSrc(path));
            await invoke('add_image_from_path', { path: path, title: 'this one' });
            await get_images();
        }
    };

    const get_images = async () => {
        let list: [Image] = await invoke('get_images');
        console.log(list);
        images = list;
    };

    get_images();

    let tag_name = '';
    const add_tag = async () => {
        if (tag_name == '') {
            return;
        }
        await invoke('add_tag_to_image', { img: images[0], tag: tag_name });
        get_images();
    };
    const remove_tag = async () => {
        if (tag_name == '') {
            return;
        }
        await invoke('remove_tag_from_image', { img: images[0], tag: tag_name });
        get_images();
    };
</script>

<DataListener {file_drop} />

<button on:click={get_images}>refresh</button>

<input bind:value={tag_name} />
<button on:click={add_tag}>add tag</button>
<button on:click={remove_tag}>remove tag</button>

<cl>
    {#each images as img}
        <!-- <rw> -->
        <card-div draggable="true" on:dragstart={print}>
            <image-div>
                <img src={convertFileSrc(img.path)} alt="" />
            </image-div>
            <span class="title">
                {img.title}
            </span>
            <tags-div>
                <!-- <cl> -->
                {#each img.tags as tag}
                    <button>{tag}</button>
                {/each}
                <!-- </cl> -->
            </tags-div>
        </card-div>
        <!-- </rw> -->
    {/each}
</cl>

<style>
    image-div {
        width: 100%;
        /* height: 100%; */
        height: calc(100% - 2.7ch);
        overflow: hidden;
    }

    image-div img {
        margin-left: 6px;
        margin-right: 6px;
        margin-top: 6px;
        border-radius: 15px;

        /* contain image */
        width: calc(100% - 12px);
        /* width: calc(100% - padding-left - padding-right); */
        height: calc(100% - 10px);
        object-fit: cover;
    }

    .title {
        padding-left: 8px;
        padding-right: 8px;
        width: calc(100% - 16px);
        height: 2.7ch;

        text-align: center;

        text-overflow: ellipsis;
        overflow: hidden;
        white-space: nowrap;
    }

    card-div {
        display: flex;
        flex-direction: column;
        align-items: center;
        color: rgb(179, 179, 179);
        /* overflow:scroll; */
        margin-left: 10px;
        margin-top: 10px;

        /* - 2px for border */
        width: calc(25vw - 14px);
        /* height: 40vw; */
        aspect-ratio: 0.6;
        background-color: blueviolet;

        border: 1px solid;
        border-radius: 15px;
        border-color: red;
    }

    card-div + card-div {
    }

    tags-div {
        /* display:flex; */
        /* flex-direction: column; */
        width: 100%;
        height: 100%;
    }

    cl {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        width: 100%;
        /* margin: 10px; */
    }
    rw {
        display: flex;
        flex-direction: column;
    }
</style>
