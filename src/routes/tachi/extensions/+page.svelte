<script lang="ts" context="module">
    import type { Extension, ExtensionAction, Indexed } from 'types';
    import { new_searcher, type RSearcher } from '$lib/commands';
    import { writable } from 'svelte/store';

    const facet = { Temp: '/temp/tachi/extension' };

    let searcher = writable(new_searcher<Extension>(facet, 50));
    let selected = writable(0);
    let search_query = writable('');
</script>

<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri';
    import { onMount, tick } from 'svelte';
    import Card from '$lib/Card.svelte';
    import VirtualScrollable from '$lib/VirtualScrollable.svelte';

    const get_all_extensions = async () => {
        let exts: Extension[] = await invoke('tachidesk_get_all_extensions');
        await invoke('delete_facet_objects', { facet });
        await $searcher.add_item(
            ...exts.map((e) => {
                let searchable: Indexed[] = [{ data: e.name, field: 'Text' }];
                return { data: e, searchable };
            })
        );
    };

    $searcher.on_update = async (e: RSearcher<Extension>) => {
        items = e.search_results.map((e) => {
            return { id: e.id, data: e };
        });
    };

    onMount(async () => {
        await invoke('init_tachidesk_client');
        if ($searcher.search_results.length == 0) {
            await get_all_extensions();
        }
    });

    let search_images = async () => {
        await $searcher.set_query($search_query);
        await end_reached();
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
    const tachidesk_action = async (pkgName: string, action: ExtensionAction) => {
        // this await waits till the action is complete in the backend
        // https://github.com/Suwayomi/Tachidesk-WebUI/blob/d51150b7848cf7a6596bbba7c015328a578dfd16/src/components/ExtensionCard.tsx#L91
        await invoke('tachidesk_extension_action', { pkgName, action });
    };

    const on_keydown = async (
        _: KeyboardEvent,
        _scroll_selected_into_view: () => Promise<void>
    ) => {};

    let window_width = 100;
    let end_is_visible = false;

    let items = new Array();
    $searcher.on_update($searcher);
</script>

<svelte:window bind:innerWidth={window_width} />
<cl class={'inputs'}>
    <button on:click={get_all_extensions}>get all extensions</button>
    <input bind:value={$search_query} on:input={search_images} />
</cl>

<cl>
    <VirtualScrollable
        gap={15}
        bind:items
        item_width={150}
        item_height={150 + 40}
        bind:selected={$selected}
        {on_keydown}
        {end_reached}
        bind:end_is_visible
        let:item_width={width}
        let:item_height
        let:root
        let:item={ext}
        let:selected={s}
    >
        <flek>
            <Card
                img_scale={'127%'}
                get_img_source={async () => {
                    return 'http://0.0.0.0:4567' + ext.data.data.iconUrl;
                }}
                title={ext.data.data.name}
                {width}
                height={width}
                selected={s}
                item={ext}
                {root}
            >
                {#if ext.data.data.isNsfw}
                    <nsfw-tag>
                        <span>nsfw</span>
                    </nsfw-tag>
                {/if}
            </Card>

            <buttons-box>
                <button
                    on:click={() =>
                        tachidesk_action(
                            ext.data.data.pkgName,
                            ext.data.data.installed ? 'uninstall' : 'install'
                        )}
                >
                    {ext.data.data.installed ? 'uninstall' : 'install'}
                </button>
                <button>{'browse'}</button>
                <button>{'latest'}</button>
                {#if ext.data.data.hasUpdate}
                    <button on:click={() => tachidesk_action(ext.data.data.pkgName, 'update')}>
                        {'update'}
                    </button>
                {/if}
            </buttons-box>
        </flek>
    </VirtualScrollable>
</cl>

<style>
    nsfw-tag {
        --width: 20px;
        --height: 20px;
        position: absolute;
        z-index: 2;
        float: left;
        height: min-content;
        top: 5%;
        left: calc(var(--width) / 2);
        border: 2px solid;
        border-radius: 8px;
        border-color: var(--color);
        background-color: var(--color);
        padding: 0px;
        margin: 0px;
        text-align: center;
        line-height: calc(var(--height) / 1.2);
    }

    nsfw-tag span {
        font-size: 1.3ch;
        font-weight: 700;
        margin-left: 3px;
        margin-right: 3px;
        color: #cccccc;
    }

    flek {
        display: flex;
        flex-direction: column;
    }

    buttons-box {
        --color: #aa5555;
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        background-color: var(--color);
        border-radius: 15px;
        overflow: hidden;
        height: 40px;
    }
    buttons-box button {
        width: 50%;
        margin: 0px;
        padding: 0px;
        border: 0px;
        background-color: var(--color);
        color: #cccccc;
        font-weight: 600;
    }
    buttons-box button:hover {
        background-color: #55aa55;
    }

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
        overflow: auto;
        width: 100%;
        height: calc(100% - var(--input-height));
    }
</style>
