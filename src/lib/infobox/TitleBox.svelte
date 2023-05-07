<script lang="ts">
    export let title: string | null;
</script>

{#if title && title.length > 0}
    <info-title>
        {@html title.replace(new RegExp('_', 'g'), '<wbr>_<wbr>')}
    </info-title>
{/if}

<style>
    info-title {
        display: block;
        font-size: 1.7rem;
        font-weight: 130;
        word-wrap: break-all;
        color: #cccccc;
    }
</style>
