<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri';
    import Anime from './Anime.svelte';
    const hasAPI = 'IntersectionObserver' in window;

    const seasonal_anime_fetch = async () => {
        let list: any = await invoke('get_seasonal_anime');
        console.log(list);
        seasonal_anime = list.data.map((e: any) => {
            let t = {
                id: e.node.id,
                pic: e.node.main_picture.large,
                title: e.node.title
            };
            return t;
        });
    };
    const mal_auth_needed = async (): Promise<any> => {
        let data: any = await invoke('mal_auth_needed');
        console.log(data);
        return data;
    };
    const mal_auth = async (data = auth_data) => {
        await invoke('mal_auth', data);
    };

    let auth_data: any = null;
    let seasonal_anime = new Array();

    mal_auth_needed().then((data) => {
        if (data == null) {
            seasonal_anime_fetch();
        } else {
            auth_data = data;
            mal_auth(data);
        }
    });

    let window_width = 100;
    let window_height = 100;

    $: calculated_width = window_width / 160;
    $: console.log(calculated_width, window_width);
</script>

{#if auth_data != null}
    <h>
        Authentication required: {auth_data.auth_url}
    </h>
{/if}

<!-- <button disabled={auth_data == null} on:click={mal_auth}>authenticate</button> -->
<button on:click={seasonal_anime_fetch}>Fetch Seasonal Anime</button>

<cl style="grid: auto-flow / {'1fr '.repeat(4)}">
    <!-- <cl style=""> -->
    {#each seasonal_anime as anime}
        <div style="">
            <Anime title={anime.title} img_source={anime.pic} lazy={hasAPI} />
        </div>
    {/each}
</cl>

<svelte:window bind:innerHeight={window_height} bind:innerWidth={window_width} />

<style>
    dw {
        display: flex;
        flex-direction: column;
    }
    cl {
        display: grid;
        /* grid: auto-flow / 1fr 1fr 1fr 1fr; */
        /* grid-column-start: 2; */
        /* flex-direction: row; */
        /* grid-column: 2; */
    }
</style>
