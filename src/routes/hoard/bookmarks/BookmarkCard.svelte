<script lang="ts">
    import TagBox from '$lib/TagBox.svelte';
    import type { RObject } from '$lib/searcher/searcher';
    import type { Bookmark } from 'types';
    import { tag_searcher } from '$lib/ObjectExplorer.svelte';

    export let item: RObject<Bookmark>;
    export let selected: boolean;

    const dragstart = (e: DragEvent, url: string) => {
        e.dataTransfer?.setData('kolekk/ignore', 'kolekk/ignore');
        e.dataTransfer?.setData('text', url);
    };
</script>

<div
    class="flex flex-col w-full h-full px-2 py-[2px] rounded-lg gap-y-[2px] bg-opacity-40
    {selected ? 'bg-gray-500 text-gray-300' : 'bg-gray-700 text-gray-400'}"
>
    <txt class="text-[0.95rem] font-medium">
        {item.data.data.title ?? ''}
    </txt>
    <!-- <txt class='text-[0.68rem] text-gray-400'>
        <a href={item.data.data.url} target='_blank'>{item.data.data.url}</a>
    </txt> -->
    <txt>
        {item.data.data.description ?? ''}
    </txt>

    <div class="flex flex-row overflow-auto gap-x-2">
        <div class="tag" draggable={true} on:dragstart={(e) => dragstart(e, item.data.data.url)}>
            <div>{item.data.data.url}</div>
        </div>

        {#await $tag_searcher.get_tags_from_ids(...item.data.tags) then tags}
            {#each tags as tag (tag.id)}
                <div class="tag">
                    {tag.data.name}
                </div>
            {/each}
        {/await}
    </div>
</div>

<style lang="postcss">
    .tag {
        @apply block rounded-t-xl font-normal text-[0.75rem] px-2 py-[0.09rem] text-gray-400 bg-gray-400 bg-opacity-20 h-5 whitespace-nowrap;
    }
    txt {
        @apply whitespace-nowrap overflow-hidden overflow-ellipsis;
    }
</style>
