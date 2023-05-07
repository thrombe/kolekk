<script lang="ts">
    import type { Tag } from 'types';

    export let tag: Tag;
    export let highlight: boolean;

    export let on_click: any = () => {};
</script>

<tag draggable="true" class:highlight on:click={on_click} on:keydown={() => {}}>
    {tag.name}
</tag>

<style>
    tag {
        display: block;
        border: 1px solid;
        border-color: #666666;
        background-color: #00000055;
        border-radius: 9px;
        font-size: 1rem;
        color: #aaaaaa;
        -webkit-backdrop-filter: blur(10px);

        padding-left: 12px;
        padding-right: 12px;
        padding-top: 3.5px;
        padding-bottom: 3.5px;
    }

    .highlight {
        border-color: #668866;
        background-color: #00220055;
    }
</style>
