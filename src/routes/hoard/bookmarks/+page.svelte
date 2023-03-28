<script lang="ts">
    import DataListener from '$lib/DataListener.svelte';
    import { files_to_bytearrays } from '$lib/data_listener';
    import { listen, type UnlistenFn, type Event } from '@tauri-apps/api/event';
    import { invoke } from '@tauri-apps/api/tauri';
    import type { Bookmark, DragDropPaste } from 'types';

    const on_receive = async (e: DragDropPaste<File>) => {
        await invoke('save_bookmark', { data: await files_to_bytearrays(e) });
    };

    let bookmarks = new Array();
    let query = '';
    const search_bookmarks = async () => {
        let list: [Bookmark] = await invoke('search_bookmarks', { query: query, limit: 50, offset: 0 });
        console.log(list);
        bookmarks = list;
    };

    search_bookmarks();
    const auto_update = (_node: any) => {
        // document.addEventListener("item-added", search_bookmarks, true);
        let unlisten: UnlistenFn | undefined;
        let destroyed = false;
        listen('item-added', (event: Event<number>) => {
            console.log(event);
            console.log(event.payload);
            search_bookmarks();
        }).then(e => {
            if (destroyed) {
                e();
            } else {
                unlisten = e;
            }
        });
        return {destroy() {
            // document.removeEventListener("item-added", search_bookmarks, true);
            destroyed = true;
            if (unlisten) {
                unlisten();
            }
        }}
    };

    let tag_name = '';
    const add_tag = async () => {};
    const remove_tag = async () => {};
</script>

<DataListener {on_receive} />
<div use:auto_update />

<cl>
    <buttons>
        <input bind:value={query} on:input={search_bookmarks} />
        <button on:click={search_bookmarks}>refresh</button>

        <input bind:value={tag_name} />
        <button on:click={add_tag}>add tag</button>
        <button on:click={remove_tag}>remove tag</button>
    </buttons>
    {#each bookmarks as bk}
        <span class={"title"}>{bk.title?bk.title:bk.url}</span>
    {/each}
</cl>

<style>
    .title {
        font-size: 1.87ch;
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

    buttons {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        width: 100%;
        height: 33px;
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
