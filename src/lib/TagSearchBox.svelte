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

<tag-box class='flex flex-col backdrop-blur-sm max-h-full w-full p-4 gap-y-4 bg-gray-900 bg-opacity-90 rounded-lg border-2 border-gray-700'>
    <input
        bind:this={tag_search_input}
        bind:value={search_query}
        placeholder="Search"
        on:input={async () => {
            await on_input();
            key += 1;
        }}
        on:keydown={_on_keydown}
        class='px-8 py-1 h-10 rounded-lg font-normal flex-grow bg-opacity-40 bg-gray-600 text-gray-400 text-xl w-full'
    />
    {#key key}
        <tags class='flex flex-row flex-wrap overflow-y-auto overscroll-contain gap-2'>
            {#each $tag_searcher.search_results as tag (tag.id)}
                <TagBox
                    tag={tag.data.name}
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

