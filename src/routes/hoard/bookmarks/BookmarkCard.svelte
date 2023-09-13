<script lang="ts">
    import TagBox from "$lib/TagBox.svelte";
    import type { RObject } from "$lib/searcher/searcher";
    import type { Bookmark } from "types";
    import { tag_searcher } from "$lib/ObjectExplorer.svelte";

    export let item: RObject<Bookmark>;
    export let selected: boolean;
</script>

<div class='flex flex-col w-full h-full p-3 rounded-lg gap-y-1 bg-opacity-40 
    {selected ? 'bg-gray-400 text-gray-300' : 'bg-gray-700 text-gray-400'}'
>
    <txt>
        {item.data.data.title ?? ''}
    </txt>
    <txt class='text-sm text-gray-400'>
        <a href={item.data.data.url} target='_blank'>{item.data.data.url}</a>
    </txt>
    <txt>
        {item.data.data.description ?? ''}
    </txt>

    {#await $tag_searcher.get_tags_from_ids(...item.data.tags)}
    {:then tags}
        <div class='flex flex-row overflow-auto gap-x-2'>
            {#each tags as tag}
                <div class='block rounded-xl font-thin text-xs px-2 py-1 text-gray-200 bg-gray-400 bg-opacity-20 h-6 whitespace-nowrap'>
                    {tag.data.name}
                </div>
            {/each}
        </div>
    {/await}
</div>

<style lang="postcss">
    txt {
        @apply whitespace-nowrap overflow-hidden overflow-ellipsis;
    }
</style>
