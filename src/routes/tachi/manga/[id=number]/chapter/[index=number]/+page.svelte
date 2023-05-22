<script lang="ts">
    import { tick } from 'svelte';
    import { invoke } from '@tauri-apps/api/tauri';
    import { page } from '$app/stores';
    import VirtualScrollable from '$lib/VirtualScrollable.svelte';
    import Image from '$lib/Image.svelte';
    import type { Chapter } from 'types';
    import type { Unique } from '$lib/virtual';

    let end_is_visible = true;
    let window_width = 100;
    let window_height = 100;
    let selected = 0;
    let columns = 1;
    let search_input: any;
    const on_keydown = async (event: KeyboardEvent, scroll_selected_into_view: any) => {
        if (event.key == 'a') {
            // await add_tag_button();
            // event.preventDefault();
        } else if (event.key == 'F1') {
            // window.location.pathname = $page.url.pathname
            window.history.back();
        } else if (event.key == 'F2') {
            if (selected - columns >= 0) {
                selected -= columns;
            }
        } else if (event.key == 'F3') {
            if (selected + 1 < items.length) {
                selected += columns;
            }
        } else if (event.key == '/') {
            selected = 0;
            await tick();
            await scroll_selected_into_view();
            search_input.focus();
            event.preventDefault();
        }
    };

    interface Thumbnail {
        uri: string;
        fetched: boolean;
    }
    let chapter: Chapter;
    let items: Unique<Thumbnail, number>[] = new Array();
    invoke('tachidesk_get_chapter', {
        mangaId: parseInt($page.params.id),
        chapterIndex: parseInt($page.params.index)
    }).then((c) => {
        chapter = c as Chapter;
    });
    let get_page_urls = async (chapter: Chapter | undefined) => {
        if (chapter) {
            let things = Array.from({ length: Number(chapter.pageCount) }).map(async (_, i) => {
                return {
                    id: i,
                    data: {
                        uri: await invoke<string>('tachidesk_get_manga_page_url', {
                            mangaId: chapter.mangaId,
                            chapterIndex: chapter.index,
                            page: i
                        }),
                        fetched: false
                    }
                };
            });
            items = await Promise.all(things);
        }
    };
    $: get_page_urls(chapter);
    $: if (items) {
        let start = Math.max(0, selected - 5);
        let end = Math.min(selected + 10, items.length + 1);
        for (let ch of items.slice(start, end)) {
            if (!ch.data.fetched) {
                ch.data.fetched = true;
                invoke('image_thumbnail', { uri: ch.data.uri, thumbnailSize: 'original' });
            }
        }
    }
</script>

<cl>
    <VirtualScrollable
        bind:items
        gap={0}
        item_width={window_width}
        item_height={window_height}
        bind:selected
        {on_keydown}
        bind:end_is_visible
        let:item={page}
        let:item_width
        let:item_height
    >
        <Image
            width={item_width}
            height={item_height}
            lazy={false}
            img_source={page.uri}
            dynamic_thumbnail={false}
            scale="98%"
        />
    </VirtualScrollable>
</cl>

<svelte:window bind:innerHeight={window_height} bind:innerWidth={window_width} />

<style>
    * {
        --manga-info-height: 300px;
    }

    cl {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        overflow: hidden;
        width: 100%;
        height: 100%;
    }
</style>
