<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri';
    import { onDestroy } from 'svelte';
    import Folder from './Folder.svelte';
    import Player from './Player.svelte';

    export async function get_folder(path: string): Promise<any> {
        return invoke('get_folder', { path });
    }

    let path = '/home/issac/daata/phon-data/.musi';
    let folders = new Array<any>();
    const add_folder = async () => {
        let folder = await get_folder(path);
        console.log(folder);
        folders = [folder, ...folders];
    };

    let video: any;
    let player: YT.Player;
    const do_somethin = () => {
        console.log(video);

        // - [Play iFrame embedded YouTube Video on click](https://codepen.io/martinwolf/pen/DrPWXw)
        // - [Control yt Video Player with js](https://tutorialzine.com/2015/08/how-to-control-youtubes-video-player-with-javascript)
        // - [Control yt Video Player with js](https://demo.tutorialzine.com/2015/08/how-to-control-youtubes-video-player-with-javascript/)
        // this YT thing comes from the youtube iframe api script
        // - [youtube.d.ts File for the youtube-iframe-api](https://stackoverflow.com/questions/42352944/youtube-d-ts-file-for-the-youtube-iframe-api-to-use-in-angular-2-needed)
        player = new YT.Player('video', {
            width: 0,
            height: 0,
            videoId: 'KZvVWnRUrkU',
            playerVars: {
                color: 'white',
                controls: 0,
                // autoplay: 1,
                showinfo: 0,
                disablekb: 1,
                modestbranding: 1,
                playlist: 'KZvVWnRUrkU,vJnCiySv1Nw'
            },
            events: {
                onReady: (eve: any) => {
                    console.log(player);
                    // player.playVideo();
                    eve.target.playVideo();
                    eve.target.playVideo();
                    // let newTime = player.getDuration() * (40 / 100);
                    // player.seekTo(newTime);
                }
            }
        });
    };
    $: if (video) {
        // do_somethin();
    }
    let pos = 0;

    onDestroy(() => {
        if (player) {
            player.destroy();
        }
    })
</script>

<!-- <iframe
    bind:this={video}
    id="video"
    width="420"
    height="315"
    src="//www.youtube.com/embed/vJnCiySv1Nw?rel=0&autoplay=1"
    frameborder="0"
    allowfullscreen
/> -->

<svelte:head>
    <script src="https://www.youtube.com/iframe_api"></script>
</svelte:head>

<button on:click={do_somethin}>start player</button>

<button
    on:click={() => {
        console.log(player);
        player.seekTo(100, true);
        player.playVideo();
        setTimeout(() => {
            console.log('yaaaaaaaaaaaaaaaaah');
            player.nextVideo();
            setTimeout(() => {
                player.seekTo(50, true);
            }, 5000);
        }, 5000);
    }}
>
    seek
</button>
<div bind:this={video} id="video" />
<input
    type="range"
    on:mousedown={() => {}}
    on:mouseup={() => {}}
    on:change={() => {}}
    bind:value={pos}
    min="0"
    max="100"
/>

<dw>
    <Player />
</dw>

<dw>
    <cl>
        <input type="text" bind:value={path} />
        <button on:click={add_folder} style="border-left: 1px">add folder '{path}'</button>
    </cl>
</dw>

{#each folders as folder}
    <dw>
        <Folder path={folder.name} files={folder.files} />
    </dw>
{/each}

<style>
    dw {
        display: flex;
        flex-direction: column;
    }
    cl {
        display: flex;
        flex-direction: row;
    }
</style>
