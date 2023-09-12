<script lang="ts" context="module">
    import { writable } from 'svelte/store';

    let fac = writable(new_factory<Image>('Image'));
    let searcher = writable(new_db<Image>('Image', ''));
    let selected = writable(0);
    let search_query = writable('');
</script>

<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri';
    import type {
        DragDropPaste,
        DdpInfo,
        DirFiles,
        ByteArrayFile,
        Image,
        Indexed,
        Path,
        Tag
    } from 'types';
    import DataListener from '$lib/DataListener.svelte';
    import { files_to_bytearrays } from '$lib/data_listener.ts';
    // import Card from '$lib/Card.svelte';
    import Card from "./reaction_images/Card.svelte";
    import type { Unique } from '$lib/virtual.ts';
    import ImageInfoBox from '$lib/infobox/ImageInfoBox.svelte';
    import ObjectExplorer from '$lib/ObjectExplorer.svelte';
    import { new_db, type RObject, type RSearcher } from '$lib/searcher/searcher.ts';
    import { get_path } from '$lib/commands.ts';
    import { new_factory } from '$lib/searcher/database.ts';
    import { tag_searcher } from '$lib/ObjectExplorer.svelte';

    let selected_item: Unique<RObject<Image>, number>;
    let search_objects: () => Promise<void>;

    const file_drop = async (e: DragDropPaste<File>) => {
        let d = (await invoke('get_ddp_info', {
            data: await files_to_bytearrays(e)
        })) as DdpInfo<ByteArrayFile>;
        console.log(d);
        let fs: DirFiles[] = await invoke('get_image_paths_from_dirs', {
            paths: d.dirs,
            recursive: true
        });
        console.log(fs);
        let files: Image[] = await invoke('save_images_from_bytes', { files: d.files });
        console.log(files);
        let paths: Image[] = await invoke('save_images_from_paths', { paths: d.image_paths });
        console.log(paths);
        let uris: Image[] = await invoke('save_images_from_uris', { links: d.image_uris });
        console.log(uris);

        const save_dirs = async (fs: DirFiles) => {
            let tags = [fs.dir_name, ...fs.files.flatMap((f) => f.split('/').slice(0, -1))];
            let tag_map = await Promise.all(
                [...new Set(tags)].map((name) =>
                    $tag_searcher.search_or_create_tag(name).then((id) => ({ name: name, id: id }))
                )
            ).then((tags) => Object.fromEntries(tags.map((t) => [t.name, t])));
            let dirs = await Promise.all(
                fs.files.map((f) =>
                    invoke('save_images_from_paths', { paths: [fs.dir.path + "/" + f] }).then((i) => {
                        let img = (i as Image[])[0];
                        let searchable: Indexed[] = img.title
                            ? [{ data: img.title, field: 'Text' }]
                            : [];
                        searchable.push(
                            { data: tag_map[fs.dir_name].id, field: 'Tag' },
                            ...f
                                .split('/')
                                .slice(0, -1)
                                .map((name) => ({ data: tag_map[name].id, field: 'Tag' } as Indexed))
                        );
                        return { data: img, searchable };
                    })
                )
            );
            console.log(dirs);
            return dirs;
        };

        let dirs = await Promise.all(fs.map(df => save_dirs(df)));

        let imgs = files.concat(paths, uris).map((img) => {
            let searchable: Indexed[] = img.title ? [{ data: img.title, field: 'Text' }] : [];
            return { data: img, searchable };
        });

        if (dirs.length + imgs.length < 1) {
            return;
        }

        await $searcher.add_item(...dirs.flat(), ...imgs);
        console.log("added items :}")
        await search_objects();
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
    {searcher}
    bind:search_query={$search_query}
    bind:selected_item_index={$selected}
    bind:selected_item
    bind:search_objects
    item_width={150}
    item_height={170}
    on_item_click={copy_selected}
    {on_keydown}
    let:item
    let:selected
>
    <Card
        get_img_source={async () => {
            return await get_path(item.data.data.path);
        }}
        {selected}
        {item}
        dynamic_thumbnail={true}
    />

    <div
        slot="infobox"
        class="pl-4 pb-4 h-full"
        let:tag_searcher
        let:info_width
        let:info_margin
        let:show_tag_searchbox
    >
        <ImageInfoBox
            {tag_searcher}
            item={selected_item}
            {info_width}
            {info_margin}
            on_tag_add_button={show_tag_searchbox}
        />
    </div>
</ObjectExplorer>
