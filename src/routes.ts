import { wrap } from 'svelte-spa-router/wrap';

import NotFound from './routes/NotFound.svelte'
import Home from './routes/Home.svelte'
import Loading from './routes/Loading.svelte'

import ReactionImages from "./routes/hoard/ReactionImages.svelte";

export default {
    '/': Home,

    // '/hoard/reaction_images/': wrap({
    //     asyncComponent: () => import('./routes/hoard/ReactionImages.svelte'),
    //     loadingComponent: Loading,
    //     loadingParams: { message: "loading ig :/" },
    // }),

    '/hoard/reaction_images/': ReactionImages,

    '/music/album/': wrap({
        asyncComponent: () => import('./routes/music/Album.svelte'),
        loadingComponent: Loading,
        loadingParams: { message: "loading ig :/" },
    }),

    // Catch-all, must be last
    '*': NotFound,
}
