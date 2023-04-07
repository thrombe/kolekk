<script lang="ts">
    export let selected: boolean;
    export let width: number;
    export let item_aspect_ratio: number;

    $: height = width / item_aspect_ratio;

    let ele: HTMLElement;
    $: if (selected && ele) {
        ele.scrollIntoView({ block: 'nearest' });
    }
    $: if (ele) {
        ele.style.width = width.toString() + 'px';
        ele.style.height = height.toString() + 'px';
    }
</script>

<d bind:this={ele}>
    <slot {selected} />
</d>
