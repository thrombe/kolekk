<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
    listen("tauri://file-drop", (e) => {console.log(e)})
    const print = (e: any) => {
        let data;
        if (e.dataTransfer) {
            data = e.dataTransfer
        } else {
            data = e.clipboardData
        }
        console.log(data)
        console.log(data.getData("text"))
        console.log(data.getData("text/html"))
        console.log(data.getData("text/uri-list"))
        console.log(data.files)
        console.log(data.files[0])
    }
</script>

<svelte:window
    on:dragover|preventDefault
    on:drop|preventDefault={print}
    on:paste|preventDefault={print}
></svelte:window>
