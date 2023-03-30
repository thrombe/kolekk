<script lang="ts">
    import DataListener from '$lib/DataListener.svelte';
    import { files_to_bytearrays } from '$lib/data_listener';
    import { fastScroll } from '$lib/fast_scroll';
    import { listen, type UnlistenFn, type Event } from '@tauri-apps/api/event';
    import { invoke } from '@tauri-apps/api/tauri';
    import { onMount, tick } from 'svelte';
    import type { Bookmark, DragDropPaste } from 'types';
    import { scroll_pos, selected } from "./state";

    let new_bookmarks = new Array();
    const on_receive = async (e: DragDropPaste<File>) => {
        console.log(e);
        console.log(e.text_html);
        let bks: [Bookmark] = await invoke('get_bookmarks', { data: await files_to_bytearrays(e) });
        console.log(bks, new_bookmarks);
        new_bookmarks = bks;
    };

    const save_bookmarks = async (bks: [Bookmark]) => {
        await invoke('save_bookmarks', {data: bks});
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
        if (document.activeElement?.tagName == "INPUT") {
            if (event.key == "Escape") {
                (document.activeElement as HTMLElement).blur();
            }
            return;
        }

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

{#if new_bookmarks.length > 0}
    <some-box>
    <cl class={"new-items"} use:fastScroll>
        {#each new_bookmarks as bk, i}
            <bookmark>
                <div class={"bookmark-buttons"}>
                    <button on:click={() => bk.tags = [...bk.tags, tag_name]} >add tag</button>
                    <button on:click={() => {save_bookmarks([bk]);new_bookmarks = new_bookmarks.filter((e) => bk.id != e.id) }} >add to db</button>
                    <button on:click={() => {new_bookmarks = new_bookmarks.filter((e) => bk.id != e.id)}}>remove</button>
                </div>
                <div class={"content"} >
                    <span>{bk.title}</span>
                    <tags use:fastScroll>
                        {#each bk.tags as tag}
                            <tag>{tag}</tag>
                        {/each}
                    </tags>
                </div>
            </bookmark>
        {/each}
    </cl>
    </some-box>
{/if}

<cl use:fastScroll bind:this={main_element} on:scroll={() => $scroll_pos = main_element.scrollHeight} >
    {#each bookmarks as bk, i}
        {#if i == $selected}
            <bookmark style={"width: 50%;border-radius: 15px;overflow:hidden;"}>
                <div class={"bookmark-buttons"}>
                    <button on:click={() => bk.tags = [...bk.tags, tag_name]} >add tag</button>
                    <button on:click={() => {save_bookmarks([bk]);new_bookmarks = new_bookmarks.filter((e) => bk.id != e.id) }} >add to db</button>
                    <button on:click={() => {new_bookmarks = new_bookmarks.filter((e) => bk.id != e.id)}}>remove</button>
                </div>
                <div class={"content"} >
                    <span class={""} >{bk.title}</span>
                    <tags use:fastScroll>
                        {#each bk.tags as tag}
                            <tag>{tag}</tag>
                        {/each}
                    </tags>
                </div>
            </bookmark>
        {:else}
            <span class={"title"} on:click={() => {$selected = i}} on:keyup={()=>{}} >{bk.title?bk.title:bk.url}</span>
        {/if}
    {/each}
</cl>

<style>
    * {
        --buttons-height: 33px;
    }

    tags {
        background-color: #8add8a;
        width: 100%;
        height: calc(100% / 3);
        display: flex;
        flex-direction: row;
        overflow: auto;
    }

    tag {
        border: 1px solid;
        border-radius: 5px;
        padding-left: 4px;
        padding-right: 4px;
    }

    tag + tag {
        margin-left: 2px;
    }

    bookmark + bookmark {
        margin-top: 3px;
    }

    bookmark {
        width: 100%;
        display: flex;
        flex-direction:row;
        height: 75px;
    }

    bookmark .content span {
        width: calc(100%);
        background-color: #dd8a8a;
        overflow-wrap: break-word;
        text-overflow: ellipsis;
        height: calc(100% / 3 * 2);
    }

    bookmark .content {
        display: flex;
        flex-direction:column;
        width: calc(100% - 80px);
    }

    .bookmark-buttons {
        display: flex;
        flex-direction:column;
        height: 100%;
    }

    .bookmark-buttons button {
        width: 80px;
        margin: 0px;
        padding: 0px;
        border: 0px;
        height: calc(100% / 3);
    }

    .bookmark-buttons button:nth-of-type(1) {
        border-top-left-radius: 10px;
    }
    .bookmark-buttons button:nth-of-type(3) {
        border-bottom-left-radius: 10px;
    }

    .bookmark-buttons button:hover {
        background-color: #8add8a;
    }

    some-box {
        position: absolute;
        top: var(--buttons-height);
        height: calc(100% - var(--buttons-height));
        width: 100%;
        background-color: #282828aa;
        display: flex;
    }

    .new-items {
        position: absolute;
        top: 5%;
        left: 5%;
        max-height: calc(90% - 2px);
        background-color: #282828;
        width: 90%;
        border: 2px solid;
        border-color: #285528;
        border-radius: 15px;
        display: flex;
        align-content:flex-start;
        height: min-content;
    }

    .title {
        font-size: 1.87ch;
        padding-bottom: 0.556ch;
        font-weight: 500;
        width: calc(50%);
        height: 75px;

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
