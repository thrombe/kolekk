<script lang="ts">
    import type { RObject, RSearcher } from '$lib/searcher/searcher';
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

<tags>
    <field>Tags: </field>
    {#each tags as tag}
        <tag>
            <slot {tag} />
        </tag>
    {/each}

    <bu on:click={add_button_callback} on:keydown={() => {}}>
        <slot name="add_button" />
    </bu>
</tags>

<style>
    field {
        font-weight: 140;

        font-size: 1.2rem;
        word-wrap: break-all;
        color: #cccccc;
    }

    tags {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;

        column-gap: 7px;
        row-gap: 6px;

        overflow-y: auto;
        overscroll-behavior-block: contain;
    }

    tag {
        display: block;
    }

    bu {
        display: block;
    }
</style>
