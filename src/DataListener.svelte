<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
    
    export let file_drop = (e: any) => {
        console.log(e);
    };
    export let on_paste = (e: any) => {
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
    };

    listen("tauri://file-drop", file_drop);
</script>

<svelte:window
    on:dragover|preventDefault
    on:drop|preventDefault={on_paste}
    on:paste|preventDefault={on_paste}
></svelte:window>
