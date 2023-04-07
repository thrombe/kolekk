<script lang="ts">
    import Observer from './Observer.svelte';

    export let width: number;
    export let aspect_ratio: number;

    export let root: HTMLElement | null = null;
    export let enabled = true;

    $: height = width / aspect_ratio;
    $: margin = height * 2;

    let visible = false;
</script>

{#if enabled}
    <rel style={'width: ' + width + 'px; height: ' + height + 'px;'}>
        <abs style={'left: ' + width / 2 + 'px; top: ' + height / 2 + 'px;'}>
            <Observer bind:visible {root} {margin} />
        </abs>
        {#if visible}
            <slot />
        {/if}
    </rel>
{:else}
    <slot />
{/if}

<style>
    rel {
        position: relative;
    }
    abs {
        position: absolute;
        left: 0px;
        top: 0px;
    }
</style>
