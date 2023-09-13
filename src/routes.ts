// import { wrap } from 'svelte-spa-router/wrap';

import NotFound from './routes/NotFound.svelte'
import Home from './routes/Home.svelte'
import Loading from './routes/Loading.svelte'

import ReactionImages from './routes/hoard/ReactionImages.svelte';
import Album from './routes/music/Album.svelte';
import Bookmarks from './routes/hoard/Bookmarks.svelte';

export default {
    '/': Home,

    '/hoard/reaction_images/': ReactionImages,

    '/hoard/bookmarks/': Bookmarks,

    '/music/album/': Album,

    '/loading/': Loading,

    // Catch-all, must be last
    '*': NotFound,
}
