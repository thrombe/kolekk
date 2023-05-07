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
    import InfoBox from '$lib/infobox/InfoBox.svelte';
    import type { Unique } from '$lib/virtual';
    import MetaBox from '$lib/infobox/MetaBox.svelte';
    import TitleBox from '$lib/infobox/TitleBox.svelte';
    import TagsBox from '$lib/infobox/TagsBox.svelte';
    import TagBox from '$lib/TagBox.svelte';
    import TagSearchBox from '$lib/TagSearchBox.svelte';

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
        await $searcher.reload_reader();
        await $searcher.set_query($search_query);
        end_reached();
    };

    $: $searcher.set_query($search_query);
    let items = new Array<Unique<RObject<Image>, number>>();

    let end_is_visible = true;
    let search_input: any;
    let tag_search_input: any;
    const on_keydown = async (event: KeyboardEvent, scroll_selected_into_view: any) => {
        if (event.key == 'Enter') {
            await copy_selected();
        } else if (event.key == 'i') {
            show_item_info = !show_item_info;
        } else if (event.key == 'a') {
            event.preventDefault();
            await show_tag_searchbox();
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

    const show_tag_searchbox = async () => {
        tag_box_show = true;
        await tick();
        tag_search_input.focus();
        $tag_query = '';
        await $tag_searcher.set_query($tag_query);
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
    let selected_item: Unique<RObject<Image>, number>;
    $: if (show_item_info) {
        info_width = 350;
        info_margin = 20;
    } else {
        info_width = 0;
        info_margin = 0;
    }

    $tag_searcher.set_query($tag_query);
    const tag_box_input_handle = async (ev: KeyboardEvent) => {
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
                await $tag_searcher.add_tag_to_object(selected_item.data.id, tag_id);
                selected_item.data.data.tags.push(tag_id);
                $tag_query = '';

                await $searcher.reload_reader();
                await $tag_searcher.set_query($tag_query);
            } else if ($tag_searcher.search_results.length > 0) {
                let tag_id = $tag_searcher.search_results[0].id;
                if (!selected_item.data.data.tags.includes(tag_id)) {
                    await $tag_searcher.add_tag_to_object(selected_item.data.id, tag_id);
                    selected_item.data.data.tags.push(tag_id);
                } else {
                    await $tag_searcher.remove_tag_from_object(
                        selected_item.data.id,
                        $tag_searcher.search_results[0].id
                    );
                    selected_item.data.data.tags = selected_item.data.data.tags.filter(
                        (e) => e != tag_id
                    );
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

    const on_search_box_tag_click = async (tag: RObject<Tag>) => {
        if (selected_item.data.data.tags.includes(tag.id)) {
            await $tag_searcher.remove_tag_from_object(selected_item.data.id, tag.id);
            selected_item.data.data.tags = selected_item.data.data.tags.filter((e) => e != tag.id);
        } else {
            await $tag_searcher.add_tag_to_object(selected_item.data.id, tag.id);
            selected_item.data.data.tags.push(tag.id);
        }

        selected_item.data.data.tags = selected_item.data.data.tags;
    };
    const searchbox_tag_highlight = (tag: RObject<Tag>) => {
        let highlight = selected_item.data.data.tags.includes(tag.id);
        return highlight;
    };
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
    >
        show item info
    </button>
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
        {#key selected_item.data.data.tags.length}
            <info-box>
                <InfoBox
                    item={selected_item}
                    width={info_width - info_margin}
                    get_img_source={async () => {
                        return await get_path(selected_item.data.data.data.path);
                    }}
                >
                    <info>
                        <TitleBox title={selected_item.data.data.data.title} />
                        <MetaBox item={selected_item.data} />

                        <TagsBox
                            item={selected_item.data}
                            tag_searcher={$tag_searcher}
                            add_button_callback={show_tag_searchbox}
                            let:tag
                        >
                            <TagBox tag={tag.data} highlight={false} />
                            <button slot="add_button">+</button>
                        </TagsBox>
                    </info>
                </InfoBox>
            </info-box>
        {/key}
    {/if}

    {#if tag_box_show}
        {#key selected_item.data.id}
            <TagSearchBox
                bind:tag_searcher={$tag_searcher}
                bind:search_query={$tag_query}
                bind:tag_search_input
                on_input={async () => await $tag_searcher.set_query($tag_query)}
                on_keydown={tag_box_input_handle}
                tag_highlight={searchbox_tag_highlight}
                on_tag_click={on_search_box_tag_click}
            />
        {/key}
    {/if}
</cl>

<style>
    info {
        display: flex;
        flex-direction: column;
        row-gap: 15px;

        --margin: 15px;
        margin: var(--margin);
        width: calc(100% - var(--margin) * 2);
        height: calc(100% - var(--margin) * 2);
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
