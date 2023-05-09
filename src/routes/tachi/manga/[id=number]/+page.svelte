<script lang="ts">
    import { tick } from 'svelte';
    import { invoke } from '@tauri-apps/api/tauri';
    import { page } from '$app/stores';
    import VirtualScrollable from '$lib/VirtualScrollable.svelte';
    import ImageCard from '$lib/ImageCard.svelte';
    import type { Chapter, Manga } from 'types';
    import type { Unique } from '$lib/virtual';

    let end_is_visible = true;
    let window_width = 100;
    let window_height = 100;
    let selected = 0;
    let search_input: any;
    const on_keydown = async (
        event: KeyboardEvent,
        scroll_selected_into_view: () => Promise<void>
    ) => {
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

    $: item_aspect_ratio = window_width / 50;

    let manga: Manga;
    invoke('tachidesk_get_manga', { mangaId: parseInt($page.params.id) }).then((m) => {
        manga = m as Manga;
        console.log(manga);
    });
    let chapters: Chapter[];
    let items: Unique<Chapter, number>[] = new Array();
    invoke('tachidesk_get_manga_chapter_list', { mangaId: parseInt($page.params.id) }).then((c) => {
        chapters = c as Chapter[];
        console.log(chapters);
    });
    $: if (chapters) {
        items = chapters.map((c) => {
            return { id: Number(c.id), data: c };
        });
    }
</script>

<cl class="body">
    <manga>
        <bg>
            {#if manga}
                <text-box>Title: {manga.title}</text-box>
                <text-box>Author: {manga.author}</text-box>
                <text-box>Id: {manga.id}</text-box>
            {/if}
        </bg>
        {#if manga}
            <ImageCard
                width={window_width}
                aspect_ratio={2 / 3}
                lazy={false}
                img_source={'http://0.0.0.0:4567' + manga.thumbnailUrl}
            />
        {/if}
    </manga>

    <cl>
        <VirtualScrollable
            bind:items
            item_width={window_width}
            item_height={window_width / item_aspect_ratio}
            gap={15}
            bind:selected
            {on_keydown}
            bind:end_is_visible
            let:item_width
            let:root
            let:item={chapter}
            let:index={i}
            let:selected={s}
        >
            <a class="read-button" href="/tachi/manga/{chapter.mangaId}/chapter/{chapter.index}">
                {chapter.name}
            </a>
        </VirtualScrollable>
    </cl>
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
