<script lang="ts" context="module">
    import DataListener from '$lib/DataListener.svelte';
    import ObjectExplorer from '$lib/ObjectExplorer.svelte';
    import { new_db, new_factory } from '$lib/searcher/database';
    import { writable } from 'svelte/store';
    import type { Bookmark, DragDropPaste, Indexed, Tag } from 'types';
    import BookmarkCard from './bookmarks/BookmarkCard.svelte';
    import BookmarkInfoBox from './bookmarks/BookmarkInfoBox.svelte';
    import { invoke } from '@tauri-apps/api';
    import { files_to_bytearrays } from '$lib/data_listener';
    import type { RObject } from '$lib/searcher/searcher';
    import type { Unique } from '$lib/virtual';
    import { tick } from 'svelte';
    import Toasts from '$lib/toast/Toasts.svelte';
    import { toaster } from '$lib/toast/Toasts.svelte';

    let fac = writable(new_factory<Bookmark>('Bookmark'));
    let searcher = writable(new_db<Bookmark>('Bookmark', ''));
    let selected = writable(0);
    let search_query = writable('');
</script>

<script lang="ts">
    let selected_item: Unique<RObject<Bookmark>, number>;
    let search_objects: () => Promise<void>;

    const bookmark_drop = async (e: DragDropPaste<File>) => {
        console.log(e);
        console.log(e.text_html);
        if (e.kolekk_text?.filter((e) => e.type == 'kolekk/ignore').length) {
            return;
        }
        let bks: [Bookmark] = await invoke('get_bookmarks', { data: await files_to_bytearrays(e) });
        await save_bookmarks(bks);
        await search_objects();

        return;
        console.log(bks, new_bookmarks);
        new_bookmarks = bks.map((t) => {
            return { data: t, tags: [] };
        });
    };

    interface TempTaggable<T> {
        data: T;
        tags: Array<string>;
    }
    let new_bookmarks = new Array<TempTaggable<Bookmark>>();

    const save_bookmarks = async (bks: [Bookmark]) => {
        await $searcher.add_item(
            ...bks.map((e) => {
                let searchable: Indexed[] = e.title ? [{ data: e.title, field: 'Text' }] : [];
                return { data: e, searchable };
            })
        );
    };

    const on_keydown = async (
        event: KeyboardEvent,
        scroll_selected_into_view: () => Promise<void>
    ) => {
        if (event.key == 'Enter') {
            await invoke('copy_text', { text: selected_item.data.data.data.url });
            console.log(selected_item.data.data.data.title, selected_item.data.data.data.title);
            await toaster.toast({
                message: "url copied",
                classes: "whitespace-nowrap block bg-blue-400 rounded-lg p-2 text-sm",
                timeout: 1000,
            });
        }
    };

    const on_tag_click = async (t: RObject<Tag>) => {
        $search_query = t.data.name; 
        await tick();
        await search_objects();
    };

    let width: number;
    let height: number;

    // TODO: BookmarkCard should cache tags and fetch only when needed :/
    // maybe create a class for this
    // class object initialized where needed (scoped for the lifetime of a page)
    let tag_cache: Map<number, Tag>;
</script>

<DataListener on_receive={bookmark_drop} />

<ObjectExplorer
    bind:fac={$fac}
    {searcher}
    bind:search_query={$search_query}
    bind:selected_item_index={$selected}
    bind:selected_item
    bind:search_objects
    bind:width
    bind:height
    gap={'gap-1'}
    item_width={width}
    item_height={60}
    info_box_width={500}
    on_item_click={async () => {}}
    {on_keydown}
    let:item
    let:selected
>
    <BookmarkCard
        {selected}
        {item}
    />

    <div
        slot="infobox"
        class="pr-4 pb-4 h-full" 
        let:tag_searcher
        let:show_tag_searchbox
    >
        <BookmarkInfoBox
            {tag_searcher}
            item={selected_item}
            on_tag_add_button={show_tag_searchbox}
            on_tag_click={on_tag_click}
        />
    </div>
</ObjectExplorer>

<Toasts
/>

