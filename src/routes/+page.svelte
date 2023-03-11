<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri';

    invoke('initialise_app');
</script>

<h>
    <a href="/musi">Local Music Player</a>
</h>
<h>
    <a href="/mal_fetch">Mal Fetch</a>
</h>
<h>
    <a href="/hoard/reaction_images">Hoard em Images</a>
</h>
<h>
    <a href="/hoard/bookmarks">Hoard em Bookmarks</a>
</h>
<h>
    <a href="/hoard/wallpapers">Hoard em Wallpapers</a>
</h>
<h>
    <a href="/watch">Watch stuff</a>
</h>

<h><a>_________________________</a></h>

<h>
    <a href="/test/css">css test</a>
</h>
<h>
    <a href="/test/audio">audio test</a>
</h>
<h>
    <a href="/test/open_link">open link test</a>
</h>
<h>
    <a href="/test/drag_img_test">drag image test</a>
</h>

<style>
    h {
        display: flex;
        flex-direction: column;
    }

    a {
        color: #aaa;
        text-decoration: none;
    }

    :global(html, body) {
        /* background: rgb(15, 24, 49); */
        background: #101935;
        margin: 0px;
        width: 100%;
        height: 100%;
    }
</style>
