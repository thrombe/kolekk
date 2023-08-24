<script lang="ts">
    import type { Tag } from 'types';
    import TagBox from '$lib/TagBox.svelte';
    import type { RObject, RSearcher } from './searcher/searcher.ts';
    import type { Writable } from 'svelte/store';

    export let tag_searcher: Writable<RSearcher<Tag>>;
    export let search_query: string;
    export let tag_search_input: HTMLElement;

    export let on_input: () => Promise<void>;
    export let on_keydown: (e: KeyboardEvent) => Promise<void>;

    export let tag_highlight: (t: RObject<Tag>) => boolean;
    export let on_tag_click: (t: RObject<Tag>) => Promise<void>;

    export let rerender_on_update: any = 0;
    $: if (rerender_on_update || true) {
        key += 1;
    }

    const _on_keydown = async (e: KeyboardEvent) => {
        await on_keydown(e);
        key += 1;
    };

    let key = 1;
</script>

<tag-box>
    <input
        bind:this={tag_search_input}
        bind:value={search_query}
        placeholder="Search"
        on:input={async () => {
            await on_input();
            key += 1;
        }}
        on:keydown={_on_keydown}
    />
    {#key key}
        <tags>
            {#each $tag_searcher.search_results as tag (tag.id)}
                <TagBox
                    tag={tag.data}
                    highlight={tag_highlight(tag)}
                    on_click={async () => {
                        await on_tag_click(tag);
                        key += 1;
                    }}
                />
            {/each}
        </tags>
    {/key}
</tag-box>

<style>
    tag-box {
        display: flex;
        flex-direction: column;
        --padding: 15px;

        position: absolute;
        top: calc(var(--input-height) + var(--top-margin));
        left: var(--gap);
        width: calc(100% - var(--info-width) - var(--gap) * 2 - var(--padding) * 2);
        height: calc(100% - var(--gap) - var(--input-height) - 5px - var(--padding) * 2);
        padding: var(--padding);
        row-gap: var(--padding);

        border: 1px solid;
        border-color: #cccccc;
        background-color: #443944cc;
        -webkit-backdrop-filter: blur(3px);
    }

    tag-box input {
        margin-left: 7%;
        margin-right: 7%;
        height: var(--input-height);
        border: 1px solid;
        border-color: #666666;
        background-color: #00000055;
        border-radius: 9px;
        padding-left: 20px;
        padding-right: 20px;
        font-size: 1rem;
        color: #aaaaaa;
        -webkit-backdrop-filter: blur(10px);
    }
    tag-box input:focus {
        outline: none;
        border-color: #999999;
    }
    tag-box input::-webkit-input-placeholder {
        color: #777777;
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
</style>
