
<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri'
    import Folder from './Folder.svelte'
    import Player from "./Player.svelte"

    export async function get_folder(path: string): Promise<any> {
        return invoke("get_folder", {path})
    }

    let path = "/home/issac/daata/phon-data/.musi"
    let folders = new Array<any>()
    const add_folder = async () => {
        let folder = await get_folder(path);
        console.log(folder);
        folders = [folder, ...folders];
    }
</script>

<dw>
    <Player></Player>
</dw>


<dw>
    <cl>
        <input type="text" bind:value={path}/>
        <button on:click={add_folder} style="border-left: 1px">add folder '{path}'</button>
    </cl>
</dw>

{#each folders as folder}
    <dw>
        <Folder path={folder.name} files={folder.files}/>
    </dw>
{/each}


<style>
    dw {
        display:flex;
        flex-direction: column;
    }
    cl {
        display:flex;
        flex-direction: row;
    }
</style>
