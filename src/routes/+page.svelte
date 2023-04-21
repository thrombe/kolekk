<script lang="ts" context="module">
    import { invoke } from '@tauri-apps/api/tauri';

    invoke('initialise_app');
</script>

<script lang="ts">
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
<h>
    <a href="/tachi/extensions_virtual_test">Tachidesk extensions virtual scroll test</a>
</h>
<h>
    <a href="/tachi/sources_virtual_test">Tachidesk sources virtual test</a>
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
        background: #28282890;
        margin: 0px;
        width: 100%;
        height: 100%;
    }
</style>
