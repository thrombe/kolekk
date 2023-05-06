<script lang="ts" context="module">
    import { writable } from 'svelte/store';
    import { new_searcher, type RObject } from '$lib/commands';

    let searcher = writable(new_searcher<Bookmark>('Bookmark', 50));
    let selected = writable(0);
    let search_query = writable('');
</script>

<script lang="ts">
    import DataListener from '$lib/DataListener.svelte';
    import { files_to_bytearrays } from '$lib/data_listener';
    import Observer from '$lib/Observer.svelte';
    import { listen, type UnlistenFn, type Event } from '@tauri-apps/api/event';
    import { invoke } from '@tauri-apps/api/tauri';
    import { tick } from 'svelte';
    import type { Bookmark, DragDropPaste, Indexed, Tag } from 'types';

    interface TempTaggable<T> {
        data: T;
        tags: Array<string>;
    }
    let new_bookmarks = new Array<TempTaggable<Bookmark>>();
    const on_receive = async (e: DragDropPaste<File>) => {
        console.log(e);
        console.log(e.text_html);
        if (e.kolekk_text?.filter((e) => e.type == 'kolekk/ignore').length) {
            return;
        }
        let bks: [Bookmark] = await invoke('get_bookmarks', { data: await files_to_bytearrays(e) });
        console.log(bks, new_bookmarks);
        new_bookmarks = bks.map((t) => {
            return { data: t, tags: [] };
        });
    };

    const save_bookmarks = async (bks: [Bookmark]) => {
        await $searcher.add_item(
            ...bks.map((e) => {
                let searchable: Indexed[] = e.title ? [{ data: e.title, field: 'Text' }] : [];
                return { data: e, searchable };
            })
        );
    };

    let bookmarks = new Array<RObject<Bookmark>>();
    const search_bookmarks = async () => {
        bookmarks = await $searcher.set_query($search_query);
        $selected = 0;
    };
    const next_page = async () => {
        bookmarks = await $searcher.next_page();
    };

    search_bookmarks();
    const auto_update = (_node: any) => {
        // document.addEventListener("item-added", search_bookmarks, true);
        let unlisten: UnlistenFn | undefined;
        let destroyed = false;
        listen('item-added', (_event: Event<number>) => {
            // TODO: no such event sent from rust
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
        let tag: Tag = {
            object_type: 'main_tag',
            name: tag_search_query
        };
        let id: number = await invoke('save_new_tag', { tag });
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
            $search_query = '';
            bookmark_search_input.focus();
            event.preventDefault();
        } else if (new_bookmarks.length > 0 && event.key == 'Escape') {
            new_bookmarks = [];
            event.preventDefault();
        } else if (tag_box.show && event.key == 'Escape') {
            tag_box.show = false;
            event.preventDefault();
        }

        if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].indexOf(event.key) > -1) {
            event.preventDefault();
        }
    };

    let main_element: any;
    let bookmark_search_input: any;

    let tag_box: any = { show: false, element_rect: null };
    let tag_search_query = '';
    let tag_search_input: any;
    let searched_tags = new Array<RObject<Tag>>();
    const tag_box_tick = async () => {
        if (selected_element) {
            tag_box.element_rect = selected_element.getBoundingClientRect();
            tag_box.top = tag_box.element_rect.top;
            tag_box.left =
                tag_box.element_rect.x < 50
                    ? tag_box.element_rect.x + tag_box.element_rect.width
                    : 0;
            // console.log(tag_box);
            // console.log(tag_box.element);
        }
    };
    selected.subscribe((_) => {
        tag_box.show = false;
    });
    const search_tags = async () => {
        // TODO: use a searcher
        searched_tags = await invoke('search_tags', {
            query: tag_search_query,
            limit: 50,
            offset: 0
        });
        // console.log(searched_tags, tag_search_query);
    };
    const add_tag_to_bookmark = async (bk: RObject<Bookmark>, tag_id: number) => {
        if (!bk.data.tags.includes(tag_id)) {
            await invoke('add_tag_to_object', { id: bk.id, tagId: tag_id });
            bk.data.tags.push(tag_id);
        }
    };
    const get_tags_from_ids = async (tags: number[]) => {
        return await invoke<[RObject<Tag>]>('get_tags_from_ids', { ids: tags });
    };
    const remove_tag_from_bookmark = async (bk: RObject<Bookmark>, tag_id: number) => {
        await invoke('remove_tag_from_object', { id: bk.id, tagId: tag_id });
        bk.data.tags = bk.data.tags.filter((e) => e != tag_id);
    };
    const add_tag_button = async () => {
        tag_search_query = '';
        tag_box.show = true;
        await tick();
        await tag_box_tick();
        tag_search_input.focus();
    };
    const tag_box_input_handle = async (ev: KeyboardEvent, bk: RObject<Bookmark>) => {
        if (ev.key == 'Enter') {
            if (ev.ctrlKey && !searched_tags.map((t) => t.data.name).includes(tag_search_query)) {
                let id = await add_tag();
                await add_tag_to_bookmark(bk, id);
                await $searcher.reload_reader();
                await search_tags();
                bk.data.tags = bk.data.tags;
            } else if (searched_tags.length > 0) {
                if (!bk.data.tags.includes(searched_tags[0].id)) {
                    await add_tag_to_bookmark(bk, searched_tags[0].id);
                } else {
                    await remove_tag_from_bookmark(bk, searched_tags[0].id);
                }
                tag_search_query = '';
                bk.data.tags = bk.data.tags;
            }
        }
    };
    const tag_click_handle = async (bk: RObject<Bookmark>, tag: RObject<Tag>) => {
        if (!bk.data.tags.includes(tag.id)) {
            await add_tag_to_bookmark(bk, tag.id);
        } else {
            await remove_tag_from_bookmark(bk, tag.id);
        }
        bk.data.tags = bk.data.tags;
    };
    const dragstart = async (e: DragEvent, bk: RObject<Bookmark>) => {
        e.dataTransfer?.setData('text/plain', bk.data.data.url);
        e.dataTransfer?.setData('kolekk/ignore', 'ignore');
    };
</script>

<DataListener {on_receive} />
<div use:auto_update />
<svelte:window on:keyup={on_keyup} on:keydown={on_keydown} />

<buttons>
    <input
        bind:value={$search_query}
        on:input={search_bookmarks}
        bind:this={bookmark_search_input}
    />
    <button on:click={search_bookmarks}>refresh</button>

    <input bind:value={tag_name} />
    <button on:click={add_tag}>add tag</button>
    <button on:click={remove_tag}>remove tag</button>
    <button
        on:click={async () => {
            await invoke('delete_facet_objects', { facet: 'Bookmark' });
            await search_bookmarks();
            await invoke('delete_facet_objects', { facet: 'Tag' });
            await search_tags();
        }}
    >
        delete objects
    </button>
</buttons>
<buttons-blok />

{#if new_bookmarks.length > 0}
    <some-box>
        <cl class={'new-items'}>
            {#each new_bookmarks as bk, i}
                <bookmark>
                    <div class={'bookmark-buttons'}>
                        <button on:click={() => (bk.tags = [...bk.tags, tag_name])}>
                            add tag
                        </button>
                        <button
                            on:click={async () => {
                                await save_bookmarks([bk.data]);
                                new_bookmarks = new_bookmarks.filter((e) => !Object.is(bk, e));
                                await search_bookmarks();
                            }}>add to db</button
                        >
                        <button
                            on:click={() => {
                                new_bookmarks = new_bookmarks.filter((e) => !Object.is(bk, e));
                            }}
                        >
                            remove
                        </button>
                    </div>
                    <div class={'content'}>
                        <span>
                            {bk.data.title ? bk.data.title : bk.data.url}
                        </span>
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

<cl bind:this={main_element}>
    {#each bookmarks as bk, i (bk.id)}
        {#if i == $selected}
            <bookmark
                style={'width: 50%;border-radius: 15px;overflow:hidden;'}
                bind:this={selected_element}
                draggable="true"
                on:dragstart={async (e) => {
                    await dragstart(e, bk);
                }}
                on:dragend={() => {}}
            >
                <div class={'bookmark-buttons'}>
                    <button on:click={add_tag_button}>
                        {tag_box.show ? tag_search_query : 'add tag'}
                    </button>
                    <button
                        on:click={() => {
                            console.warn('this button does nothing!!!!');
                        }}
                        >add to db
                    </button>
                    <button
                        on:click={() => {
                            console.warn('this button does nothing!!!!');
                        }}
                    >
                        remove
                    </button>
                </div>
                <div class={'content'}>
                    <span class={''}>
                        {bk.data.data.title ? bk.data.data.title : bk.data.data.url}
                    </span>
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
                                    bk.data.tags = bk.data.tags;
                                }}
                            />
                            <tags>
                                {#each searched_tags as tag}
                                    <tag
                                        on:keyup={() => {}}
                                        on:click={async () => {
                                            await tag_click_handle(bk, tag);
                                            bk.data.tags = bk.data.tags;
                                        }}
                                        style={'background-color: ' +
                                            (bk.data.tags.includes(
                                                tag.data.object_type == 'alias_tag'
                                                    ? tag.data.alias_to
                                                    : tag.id
                                            )
                                                ? '#dd8a8a'
                                                : '#8add8a') +
                                            ';'}
                                    >
                                        {tag.data.name}
                                    </tag>
                                {/each}
                            </tags>
                        </floating-tag-box>
                    {/if}
                    <tags>
                        {#await get_tags_from_ids(bk.data.tags) then tags}
                            {#each tags as tag}
                                <tag
                                    on:click={async () => {
                                        if (!tag_box.show) {
                                            return;
                                        }
                                        await remove_tag_from_bookmark(bk, tag.id);
                                        bk.data.tags = bk.data.tags;
                                    }}
                                    on:keydown={() => {}}
                                >
                                    {tag.data.name}
                                </tag>
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
                on:keyup={() => {}}
            >
                {bk.data.data.title ? bk.data.data.title : bk.data.data.url}
            </span>
        {/if}
    {/each}
    <Observer enter_screen={next_page} margin={200} />
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
        height: 75px;
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
