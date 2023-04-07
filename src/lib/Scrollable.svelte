<script lang="ts">
    import Observer from '$lib/Observer.svelte';

    export let columns = 1;
    export let num_items = 1;
    export let selected = 0;
    export let on_keydown = async (_: KeyboardEvent) => {};
    export let width = 100;
    export let end_reached = async () => {};
    export let end_is_visible = true;
    export let keyboard_control = false;

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
            if (selected + 1 < num_items) {
                selected += 1;
            }
        } else if (event.key == 'ArrowUp') {
            if (selected - columns >= 0) {
                selected -= columns;
            }
        } else if (event.key == 'ArrowDown') {
            if (selected + 1 < num_items) {
                selected += columns;
            }
        } else {
            await on_keydown(event);
        }

        if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].indexOf(event.key) > -1) {
            event.preventDefault();
        }
    };

    $: item_width = width / columns;

    let root: HTMLElement | null = null;
</script>

<cl bind:this={root}>
    <slot {item_width} {root} />

    <!-- observer -->
    <Observer enter_screen={end_reached} bind:visible={end_is_visible} />
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
</style>
