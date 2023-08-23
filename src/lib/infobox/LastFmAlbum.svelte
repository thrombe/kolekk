<script lang="ts">
    import InfoBox from '$lib/infobox/InfoBox.svelte';
    import TitleBox from '$lib/infobox/TitleBox.svelte';
    import type { AlbumInfo, AlbumListResult, AlbumTrack, InfoQuery, LfmTag } from 'types';
    import type { RObject } from '$lib/searcher/searcher.ts';
    import type { Unique } from '$lib/virtual.ts';
    import { invoke } from '@tauri-apps/api';
    import { Innertube } from 'youtubei.js/web';
    import {
        Body,
        fetch as tauri_fetch,
        ResponseType,
        type FetchOptions,
        type HttpVerb
    } from '@tauri-apps/api/http';
    import { logg } from '$lib/commands.ts';
    import type VideoInfo from 'youtubei.js/dist/src/parser/youtube/VideoInfo';
    import { onDestroy } from 'svelte';

    export let item: Unique<RObject<AlbumListResult>, unknown>;
    export let info_width: number;
    export let info_margin: number;

    let tracks: AlbumTrack[] = [];
    const get_album = async () => {
        let q: InfoQuery<string> = { Album: { album: item.data.name, artist: item.data.artist } };
        let a: AlbumInfo<LfmTag[], AlbumTrack[]> = await invoke('lfm_get_album_info', {
            album: q,
            autocorrect: false
        });
        // console.log(a);
        tracks = a.tracks;
    };

    $: if (item.id) {
        tracks = [];
        get_album();
    }

    let playing: VideoInfo;
    const play_id = async (id: string) => {
        let info = await yt.getBasicInfo(id);
        // console.log(info);
        playing = info;
        // opus makes it seekable :/ ?
        let format = info.chooseFormat({ type: 'audio', quality: 'best', format: 'opus' });
        // console.log(format);
        let url = format?.decipher(yt.session.player);
        await invoke('stop_song');
        await invoke('play_song', { path: url });
    };

    const play_video = async (track: AlbumTrack) => {
        console.log(track);
        let vids = await yt.search(`${track.name} ${track.artist.name}`, { type: 'video' });
        console.log(vids);
        let first = vids.videos[0];
        console.log(first);
        await logg(vids);
        if ('id' in first) {
            await play_id(first.id);
        }
    };

    const play_song = async (track: AlbumTrack) => {
        console.log(track);
        let songs = await yt.music.search(`${track.name} ${track.artist.name}`, { type: 'song' });
        console.log(songs);
        let first = songs.songs?.contents[0];
        console.log(first);
        await logg(songs, first);
        if (first && first.id) {
            await play_id(first.id);
        }
    };

    const play_track = async (track: AlbumTrack) => {
        await play_song(track);
        // await play_video(t);
    };

    let yt: Innertube;
    const get_innertube = async () => {
        console.log('yaaaaaaaaaaaaaaaaaaah');
        yt = await Innertube.create({
            fetch: async (input: RequestInfo | URL, init?: RequestInit) => {
                let url: string;
                if (typeof input === 'string') {
                    url = input;
                } else if (input instanceof URL) {
                    url = input.toString();
                } else {
                    url = input.url;
                }

                let method: HttpVerb = 'GET';
                if (input instanceof Request) {
                    method = input.method as HttpVerb;
                }

                let head: Headers = new Headers();
                if (init?.headers) {
                    new Headers(init.headers).forEach((v, k) => {
                        // console.log(k, v);
                        head.append(k, v);
                    });
                    // head = new Headers(init.headers);
                }
                if (input instanceof Request) {
                    input.headers.forEach((v, k) => {
                        // console.log(k, v);
                        head.append(k, v);
                    });
                    // head = input.headers;
                }

                let body: Body | undefined;
                if (init?.body) {
                    body = Body.text(init.body.toString());

                    // IDK: there is a body thing in 'input' too. input.body
                    // if (input instanceof Request && input.body) {
                    //     console.log(await input.json())
                    // }
                }

                let tauri_fetch_options: FetchOptions = {
                    method: method,
                    responseType: ResponseType.Text,
                    headers: head,
                    body: body
                };

                // head.forEach((v, k) => {
                //     console.log(k, v);
                // });
                // console.log(input, init);
                // console.log(url, tauri_fetch_options);

                // - [http | Tauri](https://tauri.app/v1/api/js/http/#fetch)
                let resp = await tauri_fetch(url, tauri_fetch_options);

                // console.log(resp);
                await logg(resp.data as string);
                if (typeof resp.data !== 'string') {
                    console.error('AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA');
                }

                let resp_options = {
                    status: resp.status,
                    headers: new Headers(resp.headers)
                };
                let r = new Response(resp.data as string, resp_options);
                return r;
            }
        });
    };
    get_innertube();

    let song_pos = 0;
    let progress = setInterval(async () => {
        song_pos = await invoke('get_song_progress');
        song_pos *= 100;
        // console.log(song_pos);
    }, 1000);
    onDestroy(() => {
        clearInterval(progress);
    });
    const set_progress = async () => {
        await invoke('seek_perc', { t: song_pos/100 });
    };
</script>

<info-box style="--info-width: {info_width}px; --info-margin: {info_margin}px;">
    <InfoBox
        {item}
        width={info_width - info_margin}
        get_img_source={async () => {
            return item.data.image[item.data.image.length - 1].url;
        }}
    >
        <info>
            <TitleBox bind:title={item.data.name} />
            <artist-name>
                <field>Artist: </field>
                <name>{item.data.artist}</name>
            </artist-name>

            <cl>
                {#each tracks as track, i (track.url)}
                    <trackk
                        on:keydown={() => {}}
                        on:click={async () => {
                            await play_track(track);
                        }}
                    >
                        <field>track {i}: </field>
                        <track-name>{track.name}</track-name>
                    </trackk>
                {/each}
            </cl>

            {#if playing}
                <playing>
                    <input
                        type="range"
                        on:mousedown={() => {}}
                        on:mouseup={() => {}}
                        on:change={set_progress}
                        bind:value={song_pos}
                        min="0"
                        max="100"
                    />
                </playing>
            {/if}
        </info>
    </InfoBox>
</info-box>

<style>
    playing input {
        width: 100%;
    }

    cl {
        display: flex;
        flex-direction: column;
        overflow: auto;
    }

    name,
    track-name {
        font-size: 1rem;
        font-weight: 150;
        word-wrap: break-all;
        color: #cccccc;
    }

    artist-name {
        text-align: right;
    }

    artist-name,
    trackk {
        font-size: 0.8rem;
        font-weight: 150;
        color: #cccccc;
    }

    info {
        display: flex;
        flex-direction: column;
        row-gap: 15px;

        --margin: 15px;
        margin: var(--margin);
        width: calc(100% - var(--margin) * 2);
        height: calc(100% - var(--margin) * 2);
    }

    info-box {
        display: block;
        width: calc(var(--info-width) - var(--info-margin));
        margin-left: auto;

        height: calc(100% - 20px);
        margin-top: auto;
        margin-bottom: auto;
    }
</style>
