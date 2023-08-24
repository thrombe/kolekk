<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri';
    import { playing, progress, paused } from './Player';

    invoke('plugin:musiplayer|stop_song').then(() => {});

    $: {
        if ($playing != '') {
            invoke('plugin:musiplayer|play_song', { path: $playing }).then(() => {
                console.log('now playing song: ', $playing);
            });
            paused.set(false);
        }
    }

    let mouse_down = false;

    // clearInterval
    setInterval(() => {
        invoke('plugin:musiplayer|get_song_progress').then((value) => {
            let v = value as number;
            if (mouse_down === false) {
                progress.set(v * 100);
            }
        });
    }, 500);

    const set = () => {
        console.log($progress);
        invoke('plugin:musiplayer|seek_perc', { t: $progress / 100.0 }).then(() => {
            console.log('seek complete');
        });
    };

    $: {
        invoke('plugin:musiplayer|set_stat', { pause: $paused }).then(() => {
            console.log('stat updated');
        });
    }
</script>

<span>{$playing.split('/').pop()?.replace('.m4a', '').replace('.mp3', '')}</span>

<cl>
    <input
        class="progress-bar"
        type="range"
        on:mousedown={() => {
            mouse_down = true;
        }}
        on:mouseup={() => {
            mouse_down = false;
        }}
        on:change={set}
        bind:value={$progress}
        min="0"
        max="100"
    />

    <button
        on:click={() => {
            paused.set(!$paused);
        }}>paused {$paused}</button
    >
</cl>

<style>
    .progress-bar {
        width: 80%;
    }

    cl {
        display: flex;
        flex-direction: row;
    }
</style>
