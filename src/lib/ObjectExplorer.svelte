<script lang="ts" context="module">
    import { writable, type Writable } from 'svelte/store';

    let tag_fac = writable(TagSearch.factory());
    export let tag_searcher = writable(TagSearch.new(""));
    let tag_query = writable('');
</script>

<script lang="ts">
    import type { Tag } from 'types';
    import VirtualScrollable from '$lib/VirtualScrollable.svelte';
    import { tick } from 'svelte';
    import type { Unique } from '$lib/virtual.ts';
    import TagSearchBox from '$lib/TagSearchBox.svelte';
    import { TagSearch, type ForceDb, type RFactory, type RObject, type RSearcher } from './searcher/searcher.ts';

    export let fac: RFactory<ForceDb<T>>;
    export let searcher: Writable<RSearcher<ForceDb<T>>>;
    export let selected_item_index: number;
    export let selected_item: Unique<RObject<ForceDb<T>>, number>;
    export let item_width: number;
    export let item_height: number;
    export let search_query: string;
    export let on_item_click: () => Promise<void>;
    export let on_keydown: (
        e: KeyboardEvent,
        scroll_selected_into_view: () => Promise<void>
    ) => Promise<void>;
    export let width: number = 200;
    export let height: number = 200;
    export let info_box_width: number = 400;
    export let gap: string = 'gap-4';

    type T = $$Generic;
    interface $$Slots {
        default: {
            item: RObject<ForceDb<T>>;
            item_width: number;
            item_height: number;
            selected: boolean;
            root: HTMLElement;
        };
        infobox: {
            tag_searcher: RSearcher<Tag>;
            info_width: number;
            show_tag_searchbox: () => Promise<void>;
        };
    }

    export let search_objects = async () => {
        $searcher = await fac.with_query(search_query);
        await next_page();
        await tick();
        selected_item_index = 0;
        await try_scroll_into_view();
        end_reached();
    };
    search_objects();

    let items = new Array<Unique<RObject<ForceDb<T>>, number>>();

    let end_is_visible = true;
    let search_input: HTMLElement;
    let tag_search_input: HTMLElement;
    const _on_keydown = async (
        event: KeyboardEvent,
        scroll_selected_into_view: () => Promise<void>
    ) => {
        if (event.key == 'i') {
            show_item_info = !show_item_info;
        } else if (event.key == 'a') {
            event.preventDefault();
            await show_tag_searchbox();
        } else if (event.key == 'Escape') {
            tag_box_show = false;
        } else if (event.key == '?') {
            selected_item_index = 0;
            await tick();
            await scroll_selected_into_view();
            search_input.focus();
            event.preventDefault();
        } else if (event.key == '/') {
            search_query = '';
            search_input.focus();
            event.preventDefault();
            await search_objects();
        } else {
            await on_keydown(event, scroll_selected_into_view);
        }
    };

    const new_tag_searcher = async () => {
        let ts = await $tag_fac.with_query($tag_query);
        await ts.next_page();
        $tag_searcher = ts;
    };
    const show_tag_searchbox = async () => {
        tag_box_show = true;
        await tick();
        tag_search_input.focus();
        $tag_query = '';
        await new_tag_searcher();
    };

    const end_reached = async () => {
        while (true) {
            if (!end_is_visible || !$searcher.has_next_page) {
                break;
            }
            await next_page();
            await tick();
            await new Promise<void>(r => setTimeout(() => r(), 100));
            await tick();
        }
    };
    const next_page = async () => {
        let r = await $searcher.next_page();
        items = r.map(e => {
            return { id: e.id, data: e } as Unique<RObject<ForceDb<T>>, number>;
        });
    };
    const on_enter = async (event: KeyboardEvent) => {
        if (event.key == 'Enter') {
            search_input.blur();
        }
    };

    let info_width = 0;
    let show_item_info = false;
    $: if (show_item_info) {
        info_width = info_box_width;
    } else {
        info_width = 0;
    }

    const tag_box_input_handle = async (ev: KeyboardEvent) => {
        if (ev.key == 'Enter') {
            if (
                ev.ctrlKey &&
                $tag_query.trim() &&
                !$tag_searcher.search_results.map((t) => t.data.name).includes($tag_query.trim())
            ) {
                let tag: Tag = {
                    object_type: 'main_tag',
                    name: $tag_query
                };
                let tag_id = await $tag_searcher.add_tag(tag);
                await $tag_searcher.add_tag_to_object(selected_item.data.id, tag_id);
                selected_item.data.data.tags.push(tag_id);

                $tag_query = '';
                await new_tag_searcher();
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

                $tag_query = '';
                await new_tag_searcher();
            }
        } else if (ev.key == 'Escape') {
            tag_box_show = !tag_box_show;
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
    let try_scroll_into_view: () => Promise<void>;
</script>

<div class='flex flex-col w-full h-full'>
<cl class="flex flex-row h-9 px-4 max-w-96 my-3 gap-x-4 self-center">
    <input
        bind:value={search_query}
        on:input={search_objects}
        bind:this={search_input}
        on:keydown={on_enter}
            placeholder="Search"
        class='px-8 rounded-lg font-normal flex-grow bg-opacity-20 bg-blue-800 text-gray-400 text-xl'
    />
    <button on:click={search_objects}
        class='w-30 px-2 h-full block text-center rounded-lg bg-opacity-20 bg-blue-800 text-gray-400 font-bold'
    >refresh</button>
</cl>

<cl class="flex flex-col flex-wrap flex-grow h-1 relative" >
    <scrollable class='h-full' style='width: calc(100% - {info_width}px)'>
        <VirtualScrollable
            bind:items
            {item_width}
            {item_height}
            {on_item_click}
            {end_reached}
            bind:try_scroll_into_view
            bind:selected={selected_item_index}
            on_keydown={_on_keydown}
            bind:end_is_visible
            bind:selected_item
            bind:width
            bind:height
            bind:gap
            let:item_width
            let:item_height
            let:root
            let:item
            let:selected
        >
            <slot
                {item}
                {item_width}
                {item_height}
                {selected}
                {root}
            />
        </VirtualScrollable>
    </scrollable>

    {#if selected_item && show_item_info}
        <div class='h-full' style='width: {info_width}px;'>
        <slot name="infobox"
                tag_searcher={$tag_searcher}
                {info_width}
                {show_tag_searchbox}
        />
        </div>
    {/if}

    {#if tag_box_show}
        <div class="absolute flex flex-col justify-start z-10 h-full pb-4 px-4" style="width: calc(100% - {info_width}px)">
        <TagSearchBox
            tag_searcher={tag_searcher}
            bind:search_query={$tag_query}
            bind:tag_search_input
            rerender_on_update={selected_item.data.id}
            on_input={new_tag_searcher}
            on_keydown={tag_box_input_handle}
            tag_highlight={searchbox_tag_highlight}
            on_tag_click={on_search_box_tag_click}
        />
        </div>
    {/if}
</cl>

</div>

