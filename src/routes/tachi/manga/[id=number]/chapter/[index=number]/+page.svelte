<script lang="ts">
    import { tick } from 'svelte';
    import { invoke } from '@tauri-apps/api/tauri';
    import { page } from '$app/stores';
    import VirtualScrollable from '$lib/VirtualScrollable.svelte';
    import Image from '$lib/Image.svelte';
    import type { Chapter } from 'types';

    let end_is_visible = true;
    let window_width = 100;
    let window_height = 100;
    let selected = 0;
    let search_input: any;
    const on_keydown = async (event: KeyboardEvent, scroll_selected_into_view: any) => {
        if (event.key == 'a') {
            // await add_tag_button();
            // event.preventDefault();
        } else if (event.key == '/') {
            selected = 0;
            await tick();
            await scroll_selected_into_view();
            search_input.focus();
            event.preventDefault();
        }
    };

    let chapter: Chapter;
    let items: any[];
    invoke('tachidesk_get_chapter', {
        mangaId: parseInt($page.params.id),
        chapterIndex: parseInt($page.params.index)
    }).then((c) => {
        chapter = c as Chapter;
        console.log(chapter);
    });
    let get_page_urls = async (chapter: Chapter | undefined) => {
        if (chapter) {
            let things = Array.from({ length: Number(chapter.pageCount) }).map(async (_, i) => {
                return {
                    id: i,
                    data: await invoke<string>('tachidesk_get_manga_page_url', {
                        mangaId: chapter.mangaId,
                        chapterIndex: chapter.index,
                        page: i
                    })
                };
            });
            items = await Promise.all(things);
            console.log(items);
        }
    };
    $: get_page_urls(chapter);
</script>

<cl>
    <VirtualScrollable
        bind:items
        columns={1}
        width={window_width}
        item_height={window_height}
        bind:selected
        {on_keydown}
        bind:end_is_visible
        let:item={page_url}
        let:index={i}
        let:selected={s}
        let:item_width
        let:item_height
    >
        <Image
            width={item_width}
            height={item_height}
            lazy={false}
            img_source={page_url}
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
