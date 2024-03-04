<script lang="ts" context="module">
    import DataListener from '$lib/DataListener.svelte';
    import ObjectExplorer, { tag_searcher } from '$lib/ObjectExplorer.svelte';
    import { new_db, new_factory } from '$lib/searcher/database';
    import { writable } from 'svelte/store';
    import type { Bookmark, DragDropPaste, Indexed, Path, Tag, Tagged, WithContext } from 'types';
    import BookmarkCard from './bookmarks/BookmarkCard.svelte';
    import BookmarkInfoBox from './bookmarks/BookmarkInfoBox.svelte';
    import { invoke } from '@tauri-apps/api';
    import { files_to_bytearrays } from '$lib/data_listener';
    import type { RObject } from '$lib/searcher/searcher';
    import type { Unique } from '$lib/virtual';
    import { tick } from 'svelte';
    import Toasts, { toast } from '$lib/toast/Toasts.svelte';

    let fac = writable(new_factory<Bookmark>('Bookmark'));
    let searcher = writable(new_db<Bookmark>('Bookmark', ''));
    let selected = writable(0);
    let search_query = writable('');
</script>

<script lang="ts">
    invoke('refresh_bookmark_sources')
    
    let selected_item: Unique<RObject<Bookmark>, number>;
    let search_objects: () => Promise<void>;

    const bookmark_drop = async (e: DragDropPaste<File>) => {
        // await invoke('delete_facet_objects', { facet: 'Bookmark' });
        // await invoke('delete_facet_objects', { facet: 'BookmarkSource' });
        // return;

        console.log(e);
        if (e.kolekk_text?.filter((e) => e.type == 'kolekk/ignore').length) {
            return;
        }
        let ddp = await files_to_bytearrays(e);

        let bks: Array<Tagged<Bookmark>>;

        if (ddp.text) {
            let res: [Tagged<Bookmark>[], WithContext<Tagged<string>, string>[]] = await invoke(
                'get_tagged_bookmarks_from_text',
                { text: ddp.text }
            );
            console.log(res[0], res[1]);
            bks = res[0];
            let errored = res[1];

            if (errored.length > 0) {
                await toast(`${errored.length} lines could not be parsed into bookmarks`, 'error');
                console.log(errored);
            }
        } else if (ddp.text_html) {
            let res: [Bookmark] = await invoke('bookmarks_from_html', { html: ddp.text_html });
            bks = res.map((bk) => ({ data: bk, tags: [] }));
        } else if (ddp.file_uris) {
            for (let p of ddp.file_uris) {
                let path: Path = { base: 'AbsolutePath', path: p };
                let parts = p.split('/');
                await invoke('add_bookmark_source', { title: parts[parts.length - 1], path });
            }
            return;
        } else {
            await toast('cannot work with pasted/dropped content', 'error');
            return;
        }

        // - go through each bookmark
        // - get all tag ids (add new tags if needed)
        // - check if an url already exists in db
        // - add bookmark if not already added
        // - add tags to bookmark not currently added to the bookmark
        for (let bk of bks) {
            let tags = await Promise.all(bk.tags
                .map(t => $tag_searcher.search_or_create_tag(t)))
            let bookmark = await $searcher.exact_search_taggable(bk.data.url);

            let id: number;
            if (bookmark) {
                id = bookmark.id;
                let present = new Set(bookmark.data.tags);
                tags = tags.filter(t => !present.has(t))
            } else {
                let searchable: Indexed[] = bk.data.title ? [{ data: bk.data.title, field: 'Text' }] : [];
                searchable.push({ data: bk.data.url, field: 'Text' });
                id = await $searcher.add_item({
                    data: bk.data,
                    searchable,
                });
            }
            console.log(tags);
            // await Promise.all(tags.map(t => $tag_searcher.add_tag_to_object(id, t)));
            for (let t of tags) {
                await $tag_searcher.add_tag_to_object(id, t);
            }
        }
        // TODO: manual commits

        await search_objects();
        await toast(`${bks.length} bookmarks added`, 'info');
    };

    let new_bookmarks = new Array<Tagged<Bookmark>>();

    // TODO: use exact_search(url) to implement adding tags to bookmarks in bulk
    const add_tag_to_bookmarks = async (tag: RObject<Tag>, bks: [Bookmark]) => {
        // for (let bk of bks) {
        //     // let res: RObject<Bookmark> = await invoke('exact_search_taggable', { query: bk.url, facet: 'Bookmark' });
        //     let res = await $searcher.exact_search_taggable(bk.url);
        //     console.log(bk);
        //     console.log(res);
        //     if (!res) {
        //         continue;
        //     }
        //     let deleted = await invoke('delete_from_id', { id: res.id });
        //     console.log(deleted);
        // }
    };

    const save_bookmarks = async (bks: Bookmark[]) => {
        let bookmarks = [];
        for (let bk of bks) {
            let res = await $searcher.exact_search_taggable(bk.url);
            if (!res) {
                bookmarks.push(bk);
            } else {
                // $tag_searcher.add_tag_to_object();
            }
        }
        await $searcher.add_items(
            ...bks.map((e) => {
                let searchable: Indexed[] = e.title ? [{ data: e.title, field: 'Text' }] : [];
                searchable.push({ data: e.url, field: 'Text' });
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
            await toast('url copied', 'info');
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
    on_item_click={async () => {
        console.log(selected_item);
    }}
    {on_keydown}
    let:item
    let:selected
>
    <BookmarkCard {selected} {item} />

    <div slot="infobox" class="pr-4 pb-4 h-full" let:tag_searcher let:show_tag_searchbox>
        <BookmarkInfoBox
            {tag_searcher}
            item={selected_item}
            on_tag_add_button={show_tag_searchbox}
            {on_tag_click}
        />
    </div>
</ObjectExplorer>

<Toasts />
