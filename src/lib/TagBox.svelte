<script lang="ts">
    import type { Tag } from 'types';

    export let tag: string;
    export let highlight: boolean;

    export let on_click: (() => void) | (() => Promise<void>) = () => {};
</script>

<tag draggable="true" class:highlight on:click={on_click} on:keydown={() => {}}
    class='block rounded-2xl font-light text-sm px-4 py-2 text-gray-200
        {highlight ? 'bg-green-600 bg-opacity-40' : 'bg-gray-400 bg-opacity-20'}
    '
>
    {tag}
</tag>

