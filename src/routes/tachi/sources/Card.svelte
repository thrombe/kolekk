<script lang="ts">
    import ImageCard from '$lib/ImageCard.svelte';
    import type { MangaSource } from 'types';

    export let width: number;
    export let aspect_ratio: number;
    export let source: MangaSource;

    export let selected = false;
    export let on_click: any = () => {};
    export let root: HTMLElement | null = null;

    let img_source = 'http://0.0.0.0:4567' + source.iconUrl;

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
</script>

<this-helps-position-the-tag>
    <cl bind:this={ele} draggable="true" on:click={on_click} on:keydown={() => {}}>
        <card-div bind:this={insides}>
            <card-insides>
                <title-box>
                    <span style={'white-space: ' + wrap}>{source.name}</span>
                </title-box>
                <cropper>
                    <ImageCard
                        {img_source}
                        {width}
                        lazy={false}
                        aspect_ratio={1}
                        scale={'130%'}
                        {root}
                    />
                </cropper>

                <buttons-box>
                    <a href={'/tachi/sources/' + source.id + '/popular/'}>{'browse'}</a>
                    {#if source.supportsLatest}
                        <a href={'/tachi/sources/' + source.id + '/latest'}>{'latest'}</a>
                    {/if}
                </buttons-box>
            </card-insides>
        </card-div>
    </cl>
</this-helps-position-the-tag>

<style>
    this-helps-position-the-tag {
        position: relative;
    }

    buttons-box {
        display: flex;
        width: 100%;
        flex-direction: row;
        flex-wrap: wrap;
        height: 19%;
        background-color: var(--color);
    }
    buttons-box a {
        width: 50%;
        margin: 0px;
        padding: 0px;
        border: 0px;
        background-color: var(--color);
        color: #cccccc;
        font-weight: 600;

        text-decoration: none;
        display: flex;
        justify-content: center;
        line-height: calc(100% * 2.5);
    }
    buttons-box a:hover {
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
