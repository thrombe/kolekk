<script lang="ts">
    import { tick } from 'svelte';
    import { invoke } from '@tauri-apps/api/tauri';
    import { page } from '$app/stores';
    import VirtualScrollable from '$lib/VirtualScrollable.svelte';
    import ImageCard from '$lib/ImageCard.svelte';
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

    $: item_aspect_ratio = 2 / 3;

    let chapter: Chapter;
    let items: any[];
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
                    data: await invoke<string>('tachidesk_get_manga_page_url', {
                        mangaId: chapter.mangaId,
                        chapterIndex: chapter.index,
                        page: i
                    })
                };
            });
            items = await Promise.all(things);
        }
    };
    $: get_page_urls(chapter);
</script>

<cl>
    <VirtualScrollable
        bind:items
        columns={1}
        width={window_width}
        {item_aspect_ratio}
        bind:selected
        {on_keydown}
        bind:end_is_visible
        let:item_width={width}
        let:root
        let:item={page_url}
        let:index={i}
        let:selected={s}
    >
        <ImageCard
            {width}
            aspect_ratio={item_aspect_ratio}
            lazy={false}
            img_source={page_url}
        />
    </VirtualScrollable>
</cl>

<svelte:window bind:innerHeight={window_height} bind:innerWidth={window_width} />

<style>
    * {
        --manga-info-height: 300px;
    }

    .read-button {
        text-decoration: none;
        color: #cccccc;
    }

    manga {
        height: var(--manga-info-height);
        position: relative;
        width: 100%;
        display: flex;
        overflow: hidden;
    }

    bg {
        background-color: #28282888;
        position: absolute;
        top: 0px;
        left: 0px;
        width: 100vw;
        height: var(--manga-info-height);
        z-index: 2;
        backdrop-filter: none;
        -webkit-backdrop-filter: blur(4px);
        background-image: radial-gradient(#00000000, #000000ee);
        display: flex;
        flex-direction: column;
    }

    text-box {
        color: #888888;
        font-size: 2rem;
    }

    cl {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        overflow: hidden;
        width: 100%;
        height: 100%;
    }

    .body {
        overflow: auto;
    }
    .body::-webkit-scrollbar {
        display: none;
    }
</style>
