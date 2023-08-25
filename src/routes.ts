// import { wrap } from 'svelte-spa-router/wrap';

import NotFound from './routes/NotFound.svelte'
import Home from './routes/Home.svelte'
// import Loading from './routes/Loading.svelte'

import ReactionImages from './routes/hoard/ReactionImages.svelte';
import Album from './routes/music/Album.svelte';

export default {
    '/': Home,

    '/hoard/reaction_images/': ReactionImages,

    '/music/album/': Album,

    // Catch-all, must be last
    '*': NotFound,
}
