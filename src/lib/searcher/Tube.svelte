<script lang="ts" context="module">
    import type Innertube from "youtubei.js/web";
    import { new_innertube_instance } from "./tube";


    // this component is just to export 'tube' as a singleton across the app
    // a svelte component's module='context' script runs async code without problems
    // i don't know of a better way to achieve similar results :P
    // TODO: OOF: this only works in vite
    export const tube: Innertube = await new_innertube_instance();
</script>
