<script lang="ts">
    import type { RObject, RSearcher } from '$lib/searcher/searcher.ts';
    import type { Meta, Tag, Taggable } from 'types';

    export let item: Meta<Taggable<unknown>, unknown>;
    export let tag_searcher: RSearcher<Tag>;

    export let add_button_callback: (() => void) | (() => Promise<void>);

    let tags = new Array<RObject<Tag>>();
    $: if (item.data.tags.length || true) {
        (async () => {
            tags = await tag_searcher.get_tags_from_ids(...item.data.tags);
        })();
    }
</script>

<field class='break-all text-gray-400 font-thin'>Tags: </field>
<tags class='flex flex-row flex-wrap overscroll-contain gap-x-2 gap-y-2'>
    {#each tags as tag}
        <slot {tag} />
    {/each}

    <bu on:click={add_button_callback} on:keydown={() => {}} >
        <slot name="add_button" />
    </bu>
</tags>

