<script lang="ts" context="module">
    import { Searcher } from '$lib/commands';
    import { writable } from 'svelte/store';

    let searcher = writable(new Searcher<Image>('Image', 50));
    let selected = writable(0);
    let search_query = writable('');
</script>

<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri';
    import type { DragDropPaste, Image, Indexed } from 'types';
    import DataListener from '$lib/DataListener.svelte';
    import { files_to_bytearrays } from '$lib/data_listener';
    import VirtualScrollable from '$lib/VirtualScrollable.svelte';
    import Card from './Card.svelte';
    import { tick } from 'svelte';

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
    $searcher.on_update = async (e: Searcher<Image>) => {
        items = e.search_results.map((e) => {
            return { id: e.id, data: e };
        });
    };

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

    let window_width = 100;
    let search_images = async () => {
        await $searcher.set_query($search_query);
        end_reached();
    };

    $: $searcher.set_query($search_query);
    let items = new Array();

    let end_is_visible = true;
    let search_input: any;
    const on_keydown = async (event: KeyboardEvent, scroll_selected_into_view: any) => {
        if (event.key == 'Enter') {
            await copy_selected();
            // await add_tag_button();
            // event.preventDefault();
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
        await invoke('copy_image_to_clipboard', { imgPath: img.path });
        console.log("copied image", img.title);
    };
    const copy_selected = async () => {
        await copy(items[$selected].data.data.data);
    };
    const on_enter = async (event: KeyboardEvent) => {
        if (event.key == 'Enter') {
            search_input.blur();
        }
    };
</script>

<DataListener on_receive={file_drop} />

<svelte:window bind:innerWidth={window_width} />

<cl class="inputs">
    <input bind:value={$search_query} on:input={search_images} bind:this={search_input} on:keydown={on_enter}/>
    <button on:click={search_images}>refresh</button>

    <input bind:value={tag_name} />
    <button on:click={add_tag}>add tag</button>
    <button on:click={remove_tag}>remove tag</button>
    <button>{end_is_visible}</button>
</cl>

<cl>
    <VirtualScrollable
        bind:items
        item_width={150}
        item_height={170}
        on_item_click={copy_selected}
        {end_reached}
        bind:selected={$selected}
        {on_keydown}
        bind:end_is_visible
        let:item_width={width}
        let:item_height
        let:root
        let:item={image}
        let:selected={s}
    >
        <Card
            {width}
            aspect_ratio={width / item_height}
            selected={s}
            {image}
            {root}
        />
    </VirtualScrollable>
</cl>

<style>
    * {
        --input-height: 33px;
    }

    .inputs {
        height: var(--input-height);
    }

    cl {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        overflow: hidden;
        width: 100%;
        height: calc(100% - var(--input-height));
    }
</style>
