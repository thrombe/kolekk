<script lang="ts">
    import { tick } from "svelte";


    let show_search = false;
    let search_val = "";
    let search_input: any;

    const control_state = async (event: KeyboardEvent) => {
        if (event.key == "/") {
          if (document.activeElement?.tagName == "INPUT") {return}
            show_search = true;
            await tick();
            search_input.focus();
        } else if (event.key == "Escape") {
            show_search = false;
        }
        // console.log(search_input.value);
    };
</script>

<svelte:window on:keyup={control_state} />

{#if show_search}
    <input bind:value={search_val} bind:this={search_input}/>
{:else}
    <slot />
{/if}