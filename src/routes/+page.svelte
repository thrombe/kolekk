<h>
    <a href="/musi">Local Music Player</a>
</h>
<h>
    <a href="/mal_fetch">Mal Fetch</a>
</h>

<script lang="ts">
    import DataListener from "../DataListener.svelte";
</script>

<DataListener/>

<style>
    h {
        display: flex;
        flex-direction: column;
    }
    
    a {
        color: #aaa;
        text-decoration: none;
    }

    :global(body) {
        /* background: rgb(15, 24, 49); */
        background: #101935;
        margin: 0px;
    }
</style>

