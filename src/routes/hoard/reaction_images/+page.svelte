<script lang="ts" context="module">
    import { get_path, new_searcher, type RSearcher, type RObject } from '$lib/commands';
    import { writable } from 'svelte/store';

    let searcher = writable(new_searcher<Image>('Image', 50));
    let selected = writable(0);
    let search_query = writable('');

    let tag_searcher = writable(new_searcher<Tag>('Tag', 50));
    let tag_query = writable('');
</script>

<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri';
    import type { DragDropPaste, Image, Indexed, Path, Tag } from 'types';
    import DataListener from '$lib/DataListener.svelte';
    import { files_to_bytearrays } from '$lib/data_listener';
    import VirtualScrollable from '$lib/VirtualScrollable.svelte';
    import Card from '$lib/Card.svelte';
    import { tick } from 'svelte';
    import InfoBox from '$lib/InfoBox.svelte';
    import type { Unique } from '$lib/virtual';

    const file_drop = async (e: DragDropPaste<File>) => {
        let images: Image[] = await invoke('get_images', { data: await files_to_bytearrays(e) });
        console.log(images);
        $searcher.add_item(
            ...images.map((img) => {
                let searchable: Indexed[] = img.title ? [{ data: img.title, field: 'Text' }] : [];
                return { data: img, searchable };
            })
        );
    };

    $searcher.next_page();
    $searcher.on_update = async (e: RSearcher<Image>) => {
        items = e.search_results.map((e) => {
            return { id: e.id, data: e };
        });
    };

    let window_width = 100;
    let search_images = async () => {
        await $searcher.set_query($search_query);
        end_reached();
    };

    $: $searcher.set_query($search_query);
    let items = new Array();

    let end_is_visible = true;
    let search_input: any;
    let tag_search_input: any;
    const on_keydown = async (event: KeyboardEvent, scroll_selected_into_view: any) => {
        if (event.key == 'Enter') {
            await copy_selected();
        } else if (event.key == 'i') {
            show_item_info = !show_item_info;
        } else if (event.key == 'a') {
            tag_box_show = true;
            await tick();
            tag_search_input.focus();
            $tag_query = '';
            event.preventDefault();
        } else if (event.key == 'Escape') {
            tag_box_show = false;
        } else if (event.key == '?') {
            $selected = 0;
            await tick();
            await scroll_selected_into_view();
            search_input.focus();
            event.preventDefault();
        } else if (event.key == '/') {
            $selected = 0;
            await tick();
            await scroll_selected_into_view();
            $search_query = '';
            search_input.focus();
            event.preventDefault();
        }
    };

    const end_reached = async () => {
        while (true) {
            if (!end_is_visible || !$searcher._has_next_page) {
                break;
            }
            await $searcher.next_page();
            await tick();
        }
    };
    const copy = async (img: Image) => {
        let p: Path = {
            base: 'AbsolutePath',
            path: await invoke('image_thumbnail', {
                uri: await get_path(img.path),
                thumbnailSize: 'w350'
            })
        };
        await invoke('copy_image_to_clipboard', { imgPath: p });
        console.log('copied image', img.title);
    };
    const copy_selected = async () => {
        await copy(selected_item.data.data.data);
    };
    const on_enter = async (event: KeyboardEvent) => {
        if (event.key == 'Enter') {
            search_input.blur();
        }
    };

    let info_width = 0;
    let info_margin = 0;
    let show_item_info = true;
    let selected_item: Unique<RObject<Image>, string>;
    $: if (show_item_info) {
        info_width = 350;
        info_margin = 20;
    } else {
        info_width = 0;
        info_margin = 0;
    }

    let searched_tags = new Array();
    $tag_searcher.on_update = async (e: RSearcher<Tag>) => {
        searched_tags = e.search_results;
    };
    $tag_searcher.set_query($tag_query);
    const tag_box_input_handle = async (ev: KeyboardEvent, ob: RObject<Image>) => {
        if (ev.key == 'Enter') {
            if (
                ev.ctrlKey &&
                !$tag_searcher.search_results.map((t) => t.data.name).includes($tag_query)
            ) {
                let tag: Tag = {
                    object_type: 'main_tag',
                    name: $tag_query
                };
                let tag_id = await $tag_searcher.add_tag(tag);
                await $tag_searcher.add_tag_to_object(ob.id, tag_id);
                ob.data.tags.push(tag_id);
                $tag_query = '';

                await $searcher.reload_reader();
                searched_tags = await $tag_searcher.set_query($tag_query);
            } else if (searched_tags.length > 0) {
                let tag_id = searched_tags[0].id;
                if (!ob.data.tags.includes(tag_id)) {
                    await $tag_searcher.add_tag_to_object(ob.id, tag_id);
                    ob.data.tags.push(tag_id);
                } else {
                    await $tag_searcher.remove_tag_from_object(ob.id, searched_tags[0].id);
                    ob.data.tags = ob.data.tags.filter((e) => e != tag_id);
                }

                await $tag_searcher.reload_reader();
                $tag_query = '';
                await $tag_searcher.set_query($tag_query);
            }
        }

        // to make svelte detect updates to tags
        selected_item.data.data.tags = selected_item.data.data.tags;
    };

    let tag_box_show = false;
</script>

<DataListener on_receive={file_drop} />

<svelte:window bind:innerWidth={window_width} />

<cl class="inputs">
    <input
        bind:value={$search_query}
        on:input={search_images}
        bind:this={search_input}
        on:keydown={on_enter}
    />
    <button on:click={search_images}>refresh</button>
    <button>{end_is_visible}</button>
    <button
        on:click={() => {
            show_item_info = !show_item_info;
        }}
    >show item info</button>
</cl>

<cl class="main" style="--info-width: {info_width}px; --info-margin: {info_margin}px;">
    <scrollable>
        <VirtualScrollable
            bind:items
            gap={15}
            item_width={150}
            item_height={170}
            on_item_click={copy_selected}
            {end_reached}
            bind:selected={$selected}
            {on_keydown}
            bind:end_is_visible
            bind:selected_item
            let:item_width={width}
            let:item_height
            let:root
            let:item={image}
            let:selected={s}
        >
            <Card
                get_img_source={async () => {
                    return await get_path(image.data.data.path);
                }}
                title={''}
                {width}
                aspect_ratio={width / item_height}
                selected={s}
                item={image}
                {root}
            />
        </VirtualScrollable>
    </scrollable>

    {#if selected_item && show_item_info}
        {#key selected_item.id}
            <info-box>
                <InfoBox
                    item={selected_item}
                    width={info_width - info_margin}
                    get_img_source={async () => {
                        return await get_path(selected_item.data.data.data.path);
                    }}
                >
                    <info>
                        <info-title>
                            {@html selected_item.data.data.data.title?.replace(
                                new RegExp('_', 'g'),
                                '<wbr>_<wbr>'
                            )}
                        </info-title>

                        <tags>
                            {#key selected_item.data.data.tags.length}
                                {#await $tag_searcher.get_tags_from_ids(...selected_item.data.data.tags) then tags}
                                    {#each tags as tag}
                                        <tag
                                            on:click={async () => {
                                                if (!tag_box_show) {
                                                    return;
                                                }
                                                await $tag_searcher.remove_tag_from_object(
                                                    selected_item.data.id,
                                                    tag.id
                                                );
                                                selected_item.data.data.tags =
                                                    selected_item.data.data.tags.filter(
                                                        (e) => e != tag.id
                                                    );
                                            }}
                                            on:keydown={() => {}}
                                        >
                                            {tag.data.name}
                                        </tag>
                                    {/each}
                                {/await}
                            {/key}
                        </tags>
                    </info>
                </InfoBox>
            </info-box>
        {/key}
    {/if}

    {#if tag_box_show}
        <tag-box>
            <input
                bind:this={tag_search_input}
                bind:value={$tag_query}
                placeholder="Search"
                on:input={async () => await $tag_searcher.set_query($tag_query)}
                on:keydown={async (e) => {
                    await tag_box_input_handle(e, selected_item.data);
                }}
            />
            <tags>
                {#each searched_tags as tag (tag.id)}
                    {#if selected_item.data.data.tags.includes(tag.id)}
                        <tag
                            class="contains"
                            on:click={async () => {
                                await $tag_searcher.remove_tag_from_object(
                                    selected_item.data.id,
                                    searched_tags[0].id
                                );
                                selected_item.data.data.tags = selected_item.data.data.tags.filter(
                                    (e) => e != tag.id
                                );

                                // to make svelte detect updates to tags
                                selected_item.data.data.tags = selected_item.data.data.tags;
                            }}
                            on:keydown={() => {}}
                        >
                            {tag.data.name}
                        </tag>
                    {:else}
                        <tag
                            on:click={async () => {
                                await $tag_searcher.add_tag_to_object(
                                    selected_item.data.id,
                                    tag.id
                                );
                                selected_item.data.data.tags.push(tag.id);

                                // to make svelte detect updates to tags
                                selected_item.data.data.tags = selected_item.data.data.tags;
                            }}
                            on:keydown={() => {}}
                        >
                            {tag.data.name}
                        </tag>
                    {/if}
                {/each}
            </tags>
        </tag-box>
    {/if}
</cl>

<style>
    tag-box {
        display: flex;
        flex-direction: column;
        --padding: 15px;

        position: absolute;
        top: calc(var(--input-height) + var(--top-margin));
        left: var(--gap);
        width: calc(100% - var(--info-width) - var(--gap) * 2 - var(--padding) * 2);
        height: calc(100% - var(--gap) - var(--input-height) - 5px - var(--padding) * 2);
        padding: var(--padding);
        row-gap: var(--padding);

        border: 1px solid;
        border-color: #cccccc;
        background-color: #443944cc;
        -webkit-backdrop-filter: blur(3px);
    }

    tag-box input {
        margin-left: 7%;
        margin-right: 7%;
        height: var(--input-height);
        border: 1px solid;
        border-color: #666666;
        background-color: #00000055;
        border-radius: 9px;
        padding-left: 20px;
        padding-right: 20px;
        font-size: 1rem;
        color: #aaaaaa;
        -webkit-backdrop-filter: blur(10px);
    }
    tag-box input:focus {
        outline: none;
        border-color: #999999;
    }
    tag-box input::-webkit-input-placeholder {
        color: #777777;
    }

    tags {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;

        column-gap: 7px;
        row-gap: 6px;

        overflow-y: auto;
        overscroll-behavior-block: contain;
    }

    tag {
        display: block;
        border: 1px solid;
        border-color: #666666;
        background-color: #00000055;
        border-radius: 9px;
        font-size: 1rem;
        color: #aaaaaa;
        -webkit-backdrop-filter: blur(10px);

        padding-left: 12px;
        padding-right: 12px;
        padding-top: 3.5px;
        padding-bottom: 3.5px;
    }
    tags .contains {
        border-color: #668866;
        background-color: #00220055;
    }

    info {
        display: flex;
        flex-direction: column;
        row-gap: 15px;

        --margin: 15px;
        margin: var(--margin);
        width: calc(100% - var(--margin) * 2);
        height: calc(100% - var(--margin) * 2);
    }

    info-title {
        display: block;
        font-size: 1.7rem;
        font-weight: 130;
        word-wrap: break-all;
        color: #cccccc;
    }

    * {
        --input-height: 33px;
        --gap: 20px;
        --top-margin: 15px;
    }

    .inputs {
        height: var(--input-height);
    }

    .main {
        margin-left: var(--gap);
        margin-right: var(--gap);
        margin-top: var(--top-margin);
        width: calc(100% - var(--gap) * 2);
        height: calc(100% - var(--input-height) - var(--top-margin));

        flex-direction: column;
    }

    cl {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        width: 100%;
    }

    scrollable {
        width: calc(100% - var(--info-width));
        height: 100%;
    }

    info-box {
        width: calc(var(--info-width) - var(--info-margin));
        margin-left: auto;

        height: calc(100% - 20px);
        margin-top: auto;
        margin-bottom: auto;
    }
</style>
