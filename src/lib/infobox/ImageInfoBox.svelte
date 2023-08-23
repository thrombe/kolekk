<script lang="ts">
    import InfoBox from '$lib/infobox/InfoBox.svelte';
    import TagBox from '$lib/TagBox.svelte';
    import type { Unique } from '$lib/virtual.ts';
    import MetaBox from '$lib/infobox/MetaBox.svelte';
    import TagsBox from '$lib/infobox/TagsBox.svelte';
    import TitleBox from '$lib/infobox/TitleBox.svelte';
    import type { Image, Tag } from 'types';
    import type { RObject, RSearcher } from '$lib/searcher/searcher.ts';
    import { get_path } from '$lib/commands.ts';

    export let tag_searcher: RSearcher<Tag>;
    export let item: Unique<RObject<Image>, number>;
    export let info_width: number;
    export let info_margin: number;
    export let on_tag_add_button: (() => void) | (() => Promise<void>);
</script>

<info-box style="--info-width: {info_width}px; --info-margin: {info_margin}px;">
    <InfoBox
        {item}
        width={info_width - info_margin}
        get_img_source={async () => {
            return await get_path(item.data.data.data.path);
        }}
    >
        <info>
            <TitleBox bind:title={item.data.data.data.title} />
            <MetaBox item={item.data} />

            <TagsBox
                item={item.data}
                {tag_searcher}
                add_button_callback={on_tag_add_button}
                let:tag
            >
                <TagBox tag={tag.data} highlight={false} />
                <button slot="add_button">+</button>
            </TagsBox>
        </info>
    </InfoBox>
</info-box>

<style>
    info {
        display: flex;
        flex-direction: column;
        row-gap: 15px;

        --margin: 15px;
        margin: var(--margin);
        width: calc(100% - var(--margin) * 2);
        height: calc(100% - var(--margin) * 2);
    }

    info-box {
        display: block;
        width: calc(var(--info-width) - var(--info-margin));
        margin-left: auto;

        height: calc(100% - 20px);
        margin-top: auto;
        margin-bottom: auto;
    }
</style>
