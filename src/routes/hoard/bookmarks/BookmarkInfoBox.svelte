<script lang="ts">
    import InfoBox from '$lib/infobox/InfoBox.svelte';
    import MetaBox from '$lib/infobox/MetaBox.svelte';
    import TagsBox from '$lib/infobox/TagsBox.svelte';
    import TagBox from '$lib/TagBox.svelte';
    import TitleBox from '$lib/infobox/TitleBox.svelte';
    import type { RObject, RSearcher } from '$lib/searcher/searcher';
    import type { Unique } from '$lib/virtual';
    import type { Bookmark, Tag } from 'types';

    export let tag_searcher: RSearcher<Tag>;
    export let item: Unique<RObject<Bookmark>, number>;
    export let on_tag_add_button: (() => void) | (() => Promise<void>);
    export let on_tag_click: ((t: RObject<Tag>) => void) | ((t: RObject<Tag>) => Promise<void>);
</script>

<div class="block w-full h-full bg-gray-900 bg-opacity-80 rounded-lg border-2 border-gray-700 p-4">
    <InfoBox
        {item}
        get_img_source={async () => {
            return '';
        }}
    >
        <info class="flex flex-col gap-y-4 px-4 pt-4">
            <TitleBox bind:title={item.data.data.data.title} />
            <MetaBox item={item.data} />
            <a
                draggable={true}
                href={item.data.data.data.url}
                class="block w-full text-gray-300 text-sm py-2 px-3 text-center bg-gray-400 bg-opacity-20 rounded-xl break-words"
            >
                {item.data.data.data.url}
            </a>

            <TagsBox
                item={item.data}
                {tag_searcher}
                add_button_callback={on_tag_add_button}
                let:tag
            >
                <TagBox tag={tag.data.name} highlight={false} on_click={() => on_tag_click(tag)} />
                <div slot="add_button">
                    <TagBox tag={'+'} highlight={false} />
                </div>
            </TagsBox>
        </info>
    </InfoBox>
</div>
