<script lang="ts">
    import File from './File.svelte';
    import { invoke } from '@tauri-apps/api/tauri';

    async function get_folder(path: string): Promise<any> {
        return invoke('get_folder', { path });
    }

    export let expanded = false;
    export let path: string;
    export let files: any[];

    async function toggle() {
        if (files == null) {
            console.log(path);
            let folder = await get_folder(path);
            console.log(folder);
            if (folder != null) {
                files = folder.files;
            }
        }
        if (files != null) {
            expanded = !expanded;
        }
    }

    $: name = path.split('/').pop();
</script>

<button class:expanded on:click={toggle}>{name}</button>

{#if expanded}
    <ul>
        {#each files as file}
            <li>
                {#if file.kind === 'Folder'}
                    <svelte:self path={file.name} files={file.files} />
                {:else}
                    <File path={file.name} />
                {/if}
            </li>
        {/each}
    </ul>
{/if}

<style>
    button {
        padding: 0 0 0 1.5em;
        /* background: url(/tutorial/icons/folder.svg) 0 0.1em no-repeat; */
        background-size: 1em 1em;
        font-weight: bold;
        cursor: pointer;
        border: none;
        margin: 0;
        border-left: 1px solid #eee;
    }

    /* .expanded {
		background-image: url(/tutorial/icons/folder-open.svg);
	} */

    ul {
        padding: 0.2em 0 0 0.5em;
        margin: 0 0 0 0.5em;
        list-style: none;
        border-left: 1px solid #eee;
    }

    li {
        /* padding: 0.2em 0; */
    }
</style>
