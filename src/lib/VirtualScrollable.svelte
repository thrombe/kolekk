<script lang="ts">
    import Observer from '$lib/Observer.svelte';
    import { tick } from 'svelte';

    export let width: number;
    export let item_height: number;
    export let columns: number;
    export let items: Array<any>;
    export let selected: number;

    export let end_reached = async () => {};
    export let on_keydown = async (_: KeyboardEvent, _a: any) => {};
    export let end_is_visible = true;
    export let keyboard_control = true;


    $: item_width = width / columns;
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
    const _on_keydown = async (event: KeyboardEvent) => {
        if (!keyboard_control) {
            return;
        }
        if (document.activeElement?.tagName == 'INPUT') {
            if (event.key == 'Escape') {
                (document.activeElement as HTMLElement).blur();
            }
            return;
        }

        if (event.key == 'ArrowLeft') {
            if (selected - 1 >= 0) {
                selected -= 1;
            }
        } else if (event.key == 'ArrowRight') {
            if (selected + 1 < items.length) {
                selected += 1;
            }
        } else if (event.key == 'ArrowUp') {
            if (selected - columns >= 0) {
                selected -= columns;
            }
        } else if (event.key == 'ArrowDown') {
            if (selected + 1 < items.length) {
                selected += columns;
            }
        } else {
            await on_keydown(event, try_scroll_into_view);
        }

        if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].indexOf(event.key) > -1) {
            event.preventDefault();
        }
    };
    $: if (width && root && items) {
        on_update();
    }
    let selected_item: HTMLElement;
    let try_scroll_into_view = async () => {
        await tick();
        if (selected_item) {
            selected_item.scrollIntoView({ block: "nearest" });
        } else {
            let row = selected/columns;
            // if (row*item_height > root.scrollTop + root.clientHeight) {
            if (row*item_height > root.scrollTop) {
                root.scrollTo(0, Math.floor(row)*item_height - root.clientHeight + item_height);
            } else {
                root.scrollTo(0, Math.floor(row)*item_height);
            }
            on_update();
        }
    };
    $: if (selected) {
        try_scroll_into_view();
    }
</script>

<cl on:scroll={on_update}  bind:this={root}>
    <pad style="height: {top_padding}px; width: 100%;" />
    {#each visible as item, i (item.id)}
        {#if (selected == i + start*columns) || (i + start*columns == items.length - 1 && selected >= items.length)}
            <sel bind:this={selected_item} style="width: {item_width}px; height: {item_height}px;">
                <slot {item_width} {item_height} {root} item={item.data} index={i + start*columns} selected={true} />
            </sel>
        {:else}
            <clk on:click={() => {selected = i + start*columns}} on:keydown={() => {}} style="width: {item_width}px; height: {item_height}px;" >
                <slot {item_width} {item_height} {root} item={item.data} index={i + start*columns} selected={false} />
            </clk>
        {/if}
    {/each}
    <pad  style="height: {bottom_padding}px; width: 100%;" />

    <!-- observer -->
    <div style="height: 10px; width: 100%;">
        <Observer enter_screen={end_reached} bind:visible={end_is_visible} {root} {margin} />
    </div>
</cl>

<svelte:window on:keydown={_on_keydown} />

<style>
    cl {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        overflow: auto;
        width: 100%;
        height: 100%;
    }

    sel,clk {
        width: 100%;
        height: 100%;
    }
</style>
