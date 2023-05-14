<script lang="ts" context="module">
    import { writable } from 'svelte/store';

    let fac = writable(new_factory<Image>("Image"));
    let searcher = writable(new_db<Image>('Image', ""));
    let selected = writable(0);
    let search_query = writable('');
</script>

<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri';
    import type { DragDropPaste, Image, Indexed, Path } from 'types';
    import DataListener from '$lib/DataListener.svelte';
    import { files_to_bytearrays } from '$lib/data_listener';
    import Card from '$lib/Card.svelte';
    import type { Unique } from '$lib/virtual';
    import ImageInfoBox from '$lib/infobox/ImageInfoBox.svelte';
    import ObjectExplorer from '$lib/ObjectExplorer.svelte';
    import { new_db, type RObject } from '$lib/searcher/searcher';
    import { get_path } from '$lib/commands';
    import { new_factory } from '$lib/searcher/database';

    let selected_item: Unique<RObject<Image>, number>;

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

    const on_keydown = async (
        event: KeyboardEvent,
        scroll_selected_into_view: () => Promise<void>
    ) => {
        if (event.key == 'Enter') {
            await copy_selected();
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
</script>

<DataListener on_receive={file_drop} />

<ObjectExplorer
    bind:fac={$fac}
    searcher={searcher}
    bind:search_query={$search_query}
    bind:selected_item_index={$selected}
    bind:selected_item
    item_width={150}
    item_height={170}
    on_item_click={copy_selected}
    {on_keydown}
    let:item
    let:item_width
    let:item_height
    let:selected
    let:root
    let:tag_searcher
    let:info_margin
    let:info_width
    let:show_tag_searchbox
>
    <Card
        get_img_source={async () => {
            return await get_path(item.data.data.path);
        }}
        title={''}
        width={item_width}
        height={item_height}
        {selected}
        {item}
        {root}
    />

    <ImageInfoBox
        slot="infobox"
        {tag_searcher}
        item={selected_item}
        {info_width}
        {info_margin}
        on_tag_add_button={show_tag_searchbox}
    />
</ObjectExplorer>
