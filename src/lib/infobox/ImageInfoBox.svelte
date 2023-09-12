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
    export let on_tag_click: ((t: RObject<Tag>) => void) | ((t: RObject<Tag>) => Promise<void>);
</script>

<info-box 
    class='block ml-auto my-auto h-full w-full overflow-hidden rounded-lg'
>
    <InfoBox
        {item}
        width={info_width - info_margin}
        get_img_source={async () => {
            return await get_path(item.data.data.data.path);
        }}
    >
        <info class='flex flex-col gap-y-4 px-4 pt-4'>
            <TitleBox bind:title={item.data.data.data.title} />
            <MetaBox item={item.data} />

            <TagsBox
                item={item.data}
                {tag_searcher}
                add_button_callback={on_tag_add_button}
                let:tag
            >
                <TagBox tag={tag.data.name} highlight={false} on_click={() => on_tag_click(tag)}/>
                <div slot="add_button">
                    <TagBox tag={'+'} highlight={false} />
                </div>
            </TagsBox>
        </info>
    </InfoBox>
</info-box>

