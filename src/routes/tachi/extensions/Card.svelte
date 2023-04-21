<script lang="ts">
    import ImageCard from '$lib/ImageCard.svelte';
    import { invoke } from '@tauri-apps/api/tauri';
    import type { Extension, ExtensionAction } from 'types';

    export let width: number;
    export let aspect_ratio: number;
    export let ext: Extension;

    export let selected = false;
    export let on_click: any = () => {};
    export let root: HTMLElement | null = null;

    let img_source = 'http://0.0.0.0:4567' + ext.iconUrl;

    let ele: HTMLElement;
    $: if (ele) {
        let color = selected ? '#558855' : '#885555';
        ele.style.setProperty('--color', color);
        ele.style.setProperty('--color-transparent', color + '00');
        ele.style.setProperty('--border', '2px');
        ele.style.setProperty('--border-radius', '15px');
    }
    let insides: HTMLElement;
    $: if (insides) {
        insides.style.width = width.toString() + 'px';
        let height = width / aspect_ratio;
        insides.style.height = height.toString() + 'px';
    }

    $: wrap = selected ? 'normal' : 'nowrap';

    const tachidesk_action = async (pkgName: string, action: ExtensionAction) => {
        // this await waits till the action is complete in the backend
        // https://github.com/Suwayomi/Tachidesk-WebUI/blob/d51150b7848cf7a6596bbba7c015328a578dfd16/src/components/ExtensionCard.tsx#L91
        await invoke('tachidesk_extension_action', { pkgName, action });
    };
</script>

<this-helps-position-the-tag>
    <cl bind:this={ele} draggable="true" on:click={on_click} on:keydown={() => {}}>
        <card-div bind:this={insides}>
            <card-insides>
                <title-box>
                    <span style={'white-space: ' + wrap}>{ext.name}</span>
                </title-box>
                <cropper>
                    <ImageCard
                        {img_source}
                        {width}
                        aspect_ratio={1}
                        scale={'130%'}
                        {root}
                        lazy={false}
                    />
                </cropper>

                <buttons-box>
                    <button
                        on:click={() =>
                            tachidesk_action(ext.pkgName, ext.installed ? 'uninstall' : 'install')}
                    >
                        {ext.installed ? 'uninstall' : 'install'}
                    </button>
                    <button>{'browse'}</button>
                    <button>{'latest'}</button>
                    {#if ext.hasUpdate}
                        <button on:click={() => tachidesk_action(ext.pkgName, 'update')}
                            >{'update'}</button
                        >
                    {/if}
                </buttons-box>
                {#if ext.isNsfw}
                    <nsfw-tag>
                        <span>nsfw</span>
                    </nsfw-tag>
                {/if}
            </card-insides>
        </card-div>
    </cl>
</this-helps-position-the-tag>

<style>
    this-helps-position-the-tag {
        position: relative;
    }

    nsfw-tag {
        --width: 20px;
        --height: 20px;
        position: absolute;
        z-index: 2;
        float: left;
        height: min-content;
        top: 17%;
        left: calc(var(--width) / 2);
        border: 2px solid;
        border-radius: 8px;
        border-color: #aa5555;
        background-color: #aa5555;
        padding: 0px;
        margin: 0px;
        text-align: center;
        line-height: calc(var(--height) / 1.2);
    }

    nsfw-tag span {
        font-size: 1.2ch;
        font-weight: 700;
        margin-left: 3px;
        margin-right: 3px;
        color: #cccccc;
    }

    buttons-box {
        display: flex;
        width: 100%;
        flex-direction: row;
        flex-wrap: wrap;
        height: 19%;
        background-color: var(--color);
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
        background-color: #aa5555;
    }

    cropper {
        overflow: hidden;
    }

    title-box {
        width: 100%;
        height: 14%;
        background-color: var(--color);
        text-align: center;
        overflow: hidden;
    }

    title-box span {
        width: calc(100% - 6 * var(--border));
        width: 100px;
        padding-left: calc(3 * var(--border));
        padding-right: calc(3 * var(--border));
        text-overflow: ellipsis;
        white-space: nowrap;
        font-size: 1.37ch;
        font-weight: 550;
        color: #cccccc;
    }

    card-insides {
        width: calc(100% - 2 * var(--border) - 6px);
        height: calc(100% - 2 * var(--border) - 6px);

        display: flex;
        flex-direction: column;
        align-items: center;
        color: var(--color);
        margin-left: 3px;
        margin-right: 3px;
        margin-top: 3px;
        margin-bottom: 3px;

        border: var(--border) solid;
        border-radius: var(--border-radius);

        overflow: hidden;
    }

    cl {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        overflow: auto;
    }
</style>
