<script lang="ts">
    import DataListener from '$lib/DataListener.svelte';
    import { files_to_bytearrays } from '$lib/data_listener';
    import { fastScroll } from '$lib/fast_scroll';
    import { listen, type UnlistenFn, type Event } from '@tauri-apps/api/event';
    import { invoke } from '@tauri-apps/api/tauri';
    import { onMount, tick } from 'svelte';
    import type { Bookmark, DragDropPaste } from 'types';
    import { scroll_pos, selected } from "./state";

    const on_receive = async (e: DragDropPaste<File>) => {
        console.log(e);
        console.log(e.text_html);
        await invoke('save_bookmarks_from_drop', { data: await files_to_bytearrays(e) });
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

    let elements_per_row = 2;
    let selected_element: any;
    const on_keyup = async (_event: KeyboardEvent) => {
        if (document.activeElement?.tagName == "INPUT") {return}
    };
    const on_keydown = async (event: KeyboardEvent) => {
        if (document.activeElement?.tagName == "INPUT") {return}

        if (event.key == "ArrowLeft") {
            if ($selected-1 >= 0) {
                $selected -= 1;
            }
        } else if (event.key == "ArrowRight") {
            if ($selected+1 < bookmarks.length) {
                $selected += 1;
            }
        } else if (event.key == "ArrowUp") {
            if ($selected-elements_per_row >= 0) {
                $selected -= elements_per_row;
            }
        } else if (event.key == "ArrowDown") {
            if ($selected+elements_per_row < bookmarks.length) {
                $selected += elements_per_row;
            }
        }
        await tick();
        selected_element.scrollIntoView({ block: "nearest" });

        if(["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].indexOf(event.key) > -1) {
            event.preventDefault();
        }
    };

    let cached_scroll_pos = $scroll_pos;
    onMount(() => {
        setTimeout(async () => {
            await tick();
            main_element.scrollTo(0, cached_scroll_pos);
        }, 20);
    })

    let main_element:any;
</script>

<DataListener {on_receive} />
<div use:auto_update />
<svelte:window on:keyup={on_keyup} on:keydown={on_keydown} />

    <buttons>
        <input bind:value={query} on:input={search_bookmarks} />
        <button on:click={search_bookmarks}>refresh</button>

        <input bind:value={tag_name} />
        <button on:click={add_tag}>add tag</button>
        <button on:click={remove_tag}>remove tag</button>
    </buttons>
    <buttons-blok />
<cl use:fastScroll bind:this={main_element} on:scroll={() => $scroll_pos = main_element.scrollHeight} >
    {#each bookmarks as bk, i}
        {#if i == $selected}
            <span class={"title"} style={"background-color: #282828;"} bind:this={selected_element} >{bk.title?bk.title:bk.url}</span>
        {:else}
            <span class={"title"} on:click={() => {$selected = i}} on:keyup={()=>{}} >{bk.title?bk.title:bk.url}</span>
        {/if}
    {/each}
</cl>

<style>
    * {
        --buttons-height: 33px;
    }

    .title {
        font-size: 1.87ch;
        padding-bottom: 0.556ch;
        font-weight: 500;
        width: calc(50%);
        height: min-content;

        text-align: center;
        text-overflow: ellipsis;
        overflow: hidden;
        white-space: nowrap;

        color: aquamarine;
        /* background-color: blue; */
    }

    buttons-blok,
    buttons {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        width: 100%;
        height: var(--buttons-height);
    }

    buttons {
        position: fixed;
        top: 0px;
    }

    cl {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        overflow: auto;
        width: 100%;
        height: calc(100% - var(--buttons-height));
    }
</style>
