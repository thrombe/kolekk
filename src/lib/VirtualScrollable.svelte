<script lang="ts">
    import Observer from '$lib/Observer.svelte';

    export let width: number;
    export let item_aspect_ratio: number;
    export let columns: number;
    export let items: Array<any>;

    export let end_reached = async () => {};
    export let end_is_visible = true;

    $: item_width = width / columns;
    $: item_height = item_width / item_aspect_ratio;
    $: margin = item_height * 2;

    let start = 0;
    let end = 0;
    let visible = new Array();

    let root: HTMLElement;

    let top_padding = 0;
    let bottom_padding = 0;
    let on_update = async () => {
        // console.log(root.scrollTop, root.clientHeight, start, end, top_padding, bottom_padding);
        let s = Math.floor(root.scrollTop/item_height);
        top_padding = s*item_height;
        let e = start + Math.ceil(root.clientHeight/item_height) + 1;
        bottom_padding = (Math.ceil(items.length/columns) - e)*item_height;

        if ((start != s || end != e) && items.length != 0) {
            start = s;
            end = e;
            visible = items.slice(start*columns, Math.ceil(end)*columns);
        }
        // console.log("update", end, start, e, s, visible.length, items.length);
    };
    $: if (width && root && items) {
        on_update();
    }
</script>

<cl on:scroll={on_update}  bind:this={root}>
    <pad style="height: {top_padding}px; width: 100%;" />
        {#each visible as item, i (item.id)}
            <slot {item_width} {root} item={item.data} index={i + start*columns}/>
        {/each}
    <pad  style="height: {bottom_padding}px; width: 100%;" />

    <!-- observer -->
    <div style="height: 10px; width: 100%;">
        <Observer enter_screen={end_reached} bind:visible={end_is_visible} {root} {margin} />
    </div>
</cl>

<style>
    cl {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        overflow: auto;
        width: 100%;
        height: 100%;
    }
</style>
