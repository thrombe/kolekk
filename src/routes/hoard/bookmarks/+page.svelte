<script lang="ts">
    import DataListener from '$lib/DataListener.svelte';
    import { files_to_bytearrays } from '$lib/data_listener';
    import { fastScroll } from '$lib/fast_scroll';
    import Observer from '$lib/Observer.svelte';
    import { listen, type UnlistenFn, type Event } from '@tauri-apps/api/event';
    import { invoke } from '@tauri-apps/api/tauri';
    import { onMount, tick } from 'svelte';
    import type { Bookmark, DragDropPaste, Tag } from 'types';
    import { scroll_pos, selected } from './state';

    let new_bookmarks = new Array<Bookmark>();
    const on_receive = async (e: DragDropPaste<File>) => {
        console.log(e);
        console.log(e.text_html);
        let bks: [Bookmark] = await invoke('get_bookmarks', { data: await files_to_bytearrays(e) });
        console.log(bks, new_bookmarks);
        new_bookmarks = bks;
    };

    const save_bookmarks = async (bks: [Bookmark]) => {
        await invoke('save_bookmarks', { data: bks });
    };

    let bookmarks = new Array<Bookmark>();
    let query = '';
    const search_bookmarks = async () => {
        let list: [Bookmark] = await invoke('search_bookmarks', {
            query: query,
            limit: 50,
            offset: 0
        });
        bookmarks = list;
        $selected = 0;
    };
    const next_page = async () => {
        let list: [Bookmark] = await invoke('search_bookmarks', {
            query: query,
            limit: 50,
            offset: bookmarks.length
        });
        bookmarks.push(...list);
        bookmarks = bookmarks;
    }

    search_bookmarks();
    const auto_update = (_node: any) => {
        // document.addEventListener("item-added", search_bookmarks, true);
        let unlisten: UnlistenFn | undefined;
        let destroyed = false;
        listen('item-added', (_event: Event<number>) => {
            search_bookmarks();
        }).then((e) => {
            if (destroyed) {
                e();
            } else {
                unlisten = e;
            }
        });
        return {
            destroy() {
                // document.removeEventListener("item-added", search_bookmarks, true);
                destroyed = true;
                if (unlisten) {
                    unlisten();
                }
            }
        };
    };

    let tag_name = '';
    const add_tag = async () => {
        let id: number = await invoke('save_tag', { name: tag_search_query });
        console.log(id);
        await search_tags();
        tag_search_query = '';
        return id;
    };
    const remove_tag = async () => {};

    let elements_per_row = 2;
    let selected_element: Element;
    const on_keyup = async (_event: KeyboardEvent) => {
        if (document.activeElement?.tagName == 'INPUT') {
            return;
        }
    };
    selected.subscribe(async (_) => {
        await tick();
        if (selected_element) {
            selected_element.scrollIntoView({ block: 'nearest' });
        }
    });
    const on_keydown = async (event: KeyboardEvent) => {
        if (document.activeElement?.tagName == 'INPUT') {
            if (event.key == 'Escape') {
                (document.activeElement as HTMLElement).blur();
                tag_box.show = false;
            }
            return;
        }

        if (event.key == 'ArrowLeft') {
            if ($selected - 1 >= 0) {
                $selected -= 1;
            }
        } else if (event.key == 'ArrowRight') {
            if ($selected + 1 < bookmarks.length) {
                $selected += 1;
            }
        } else if (event.key == 'ArrowUp') {
            if ($selected - elements_per_row >= 0) {
                $selected -= elements_per_row;
            }
        } else if (event.key == 'ArrowDown') {
            if ($selected + elements_per_row < bookmarks.length) {
                $selected += elements_per_row;
            }
        } else if (event.key == 'a') {
            await add_tag_button();
            event.preventDefault();
        } else if (event.key == '/') {
            query = "";
            bookmark_search_input.focus();
            event.preventDefault();
        } else if (tag_box.show && event.key == "Escape") {
            tag_box.show = false;
            event.preventDefault();
        }

        if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].indexOf(event.key) > -1) {
            event.preventDefault();
        }
    };

    let cached_scroll_pos = $scroll_pos;
    onMount(() => {
        setTimeout(async () => {
            await tick();
            main_element.scrollTo(0, cached_scroll_pos);
        }, 20);
    });

    let main_element: any;
    let bookmark_search_input: any;

    let tag_box: any = { show: false, element_rect: null };
    let tag_search_query = '';
    let tag_search_input: any;
    let searched_tags = new Array<Tag>();
    const tag_box_tick = async () => {
        if (selected_element) {
            tag_box.element_rect = selected_element.getBoundingClientRect();
            tag_box.top = tag_box.element_rect.top;
            tag_box.left =
                tag_box.element_rect.x < 50
                    ? tag_box.element_rect.x + tag_box.element_rect.width
                    : 0;
            // console.log(tag_box);
        }
    };
    scroll_pos.subscribe((_) => {
        tag_box_tick();
    });
    selected.subscribe((_) => {
        tag_box.show = false;
    });
    const search_tags = async () => {
        searched_tags = await invoke('search_tags', {
            query: tag_search_query,
            limit: 50,
            offset: 0
        });
    };
    const add_tag_to_bookmark = async (bk: Bookmark, tag_id: number) => {
        if (!bk.tags.includes(tag_id)) {
            await invoke('add_tag_to_bookmark', { id: bk.id, tagId: tag_id });
            bk.tags.push(tag_id);
        }
    };
    const get_tags_from_ids = async (tags: number[]) => {
        return await invoke<[Tag]>('get_tags_from_ids', { ids: tags });
    };
    const remove_tag_from_bookmark = async (bk: Bookmark, tag_id: number) => {
        await invoke('remove_tag_from_bookmark', { id: bk.id, tagId: tag_id });
        bk.tags = bk.tags.filter((e) => e != tag_id);
    };
    const add_tag_button = async () => {
        tag_search_query = '';
        tag_box.show = true;
        await tick();
        await tag_box_tick();
        tag_search_input.focus();
    };
    const tag_box_input_handle = async (ev: KeyboardEvent, bk: Bookmark) => {
        if (ev.key == 'Enter') {
            if (ev.ctrlKey && !searched_tags.map((t) => t.name).includes(tag_search_query)) {
                await add_tag_to_bookmark(bk, await add_tag());
                bk.tags = bk.tags;
            } else if (searched_tags.length > 0) {
                if (!bk.tags.includes(searched_tags[0].id)) {
                    await add_tag_to_bookmark(bk, searched_tags[0].id);
                } else {
                    await remove_tag_from_bookmark(bk, searched_tags[0].id);
                }
                tag_search_query = '';
                bk.tags = bk.tags;
            }
        }
    };
    const tag_click_handle = async (bk: Bookmark, tag: Tag) => {
        if (!bk.tags.includes(tag.id)) {
            await add_tag_to_bookmark(bk, tag.id);
        } else {
            await remove_tag_from_bookmark(bk, tag.id);
        }
        bk.tags = bk.tags;
    };
</script>

<DataListener {on_receive} />
<div use:auto_update />
<svelte:window on:keyup={on_keyup} on:keydown={on_keydown} />

<buttons>
    <input bind:value={query} on:input={search_bookmarks} bind:this={bookmark_search_input} />
    <button on:click={search_bookmarks}>refresh</button>

    <input bind:value={tag_name} />
    <button on:click={add_tag}>add tag</button>
    <button on:click={remove_tag}>remove tag</button>
</buttons>
<buttons-blok />

{#if new_bookmarks.length > 0}
    <some-box>
        <cl class={'new-items'} >
            {#each new_bookmarks as bk, i}
                <bookmark>
                    <div class={'bookmark-buttons'}>
                        <button on:click={() => (bk.tags = [...bk.tags, tag_name])}>add tag</button>
                        <button
                            on:click={() => {
                                save_bookmarks([bk]);
                                new_bookmarks = new_bookmarks.filter((e) => bk.id != e.id);
                            }}>add to db</button
                        >
                        <button
                            on:click={() => {
                                new_bookmarks = new_bookmarks.filter((e) => bk.id != e.id);
                            }}>remove</button
                        >
                    </div>
                    <div class={'content'}>
                        <span>{bk.title}</span>
                        <tags>
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

<cl
    bind:this={main_element}
    on:scroll={() => ($scroll_pos = main_element.scrollTop)}
>
    {#each bookmarks as bk, i (bk.id)}
        {#if i == $selected}
            <bookmark
                style={'width: 50%;border-radius: 15px;overflow:hidden;'}
                bind:this={selected_element}
            >
                <div class={'bookmark-buttons'}>
                    <button on:click={add_tag_button}
                        >{tag_box.show ? tag_search_query : 'add tag'}</button
                    >
                    <button
                        on:click={() => {
                            save_bookmarks([bk]);
                            new_bookmarks = new_bookmarks.filter((e) => bk.id != e.id);
                        }}>add to db</button
                    >
                    <button
                        on:click={() => {
                            new_bookmarks = new_bookmarks.filter((e) => bk.id != e.id);
                        }}>remove</button
                    >
                </div>
                <div class={'content'}>
                    <span class={''}>{bk.title}</span>
                    {#if tag_box.show}
                        <floating-tag-box
                            style={'top: ' + tag_box.top + 'px;left: ' + tag_box.left + 'px;'}
                            bind:this={tag_box.element}
                        >
                            <input
                                style={'width: 0px;height: 0px; margin: 0px; padding: 0px; border: 0px;'}
                                bind:value={tag_search_query}
                                bind:this={tag_search_input}
                                on:input={search_tags}
                                on:keydown={async (e) => {
                                    await tag_box_input_handle(e, bk);
                                    bk.tags = bk.tags;
                                }}
                            />
                            <tags>
                                {#each searched_tags as tag}
                                    <tag
                                        on:keyup={() => {}}
                                        on:click={async () => {
                                            await tag_click_handle(bk, tag);
                                            bk.tags = bk.tags;
                                        }}
                                        style={'background-color: ' +
                                            (bk.tags.includes(
                                                tag.object_type == 'alias_tag'
                                                    ? tag.alias_to
                                                    : tag.id
                                            )
                                                ? '#dd8a8a'
                                                : '#8add8a') +
                                            ';'}>{tag.name}</tag
                                    >
                                {/each}
                            </tags>
                        </floating-tag-box>
                    {/if}
                    <tags>
                        {#await get_tags_from_ids(bk.tags) then tags}
                            {#each tags as tag}
                                <tag
                                    on:click={async () => {
                                        if (!tag_box.show) {
                                            return;
                                        }
                                        await remove_tag_from_bookmark(bk, tag.id);
                                        bk.tags = bk.tags;
                                    }}
                                    on:keydown={() => {}}>{tag.name}</tag
                                >
                            {/each}
                        {/await}
                    </tags>
                </div>
            </bookmark>
        {:else}
            <span
                class={'title'}
                on:click={() => {
                    $selected = i;
                }}
                on:keyup={() => {}}>{bk.title ? bk.title : bk.url}</span
            >
        {/if}
    {/each}
    <Observer enter_screen={next_page} />
</cl>

<style>
    * {
        --buttons-height: 33px;
    }

    floating-tag-box {
        position: absolute;
        display: flex;
        flex-direction: row;
        width: 50%;
        background-color: #282828;
        z-index: 2;
    }
    floating-tag-box tags {
        max-height: 75px;
        width: 100%;
        flex-wrap: wrap;
        overflow: hidden;
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
        flex-direction: row;
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
        flex-direction: column;
        width: calc(100% - 80px);
    }

    .bookmark-buttons {
        display: flex;
        flex-direction: column;
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
        align-content: flex-start;
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
