<script lang="ts">
    import { listen } from '@tauri-apps/api/event';
    import { onDestroy } from 'svelte';
    import type { DragDropPaste } from 'types';
    import { ddp_from_drop, ddp_from_event, type DropEvent } from './data_listener';

    const recv = (e: DropEvent | DragEvent | ClipboardEvent) => {
        let item: DragDropPaste<File> | null;
        if (e instanceof DragEvent || e instanceof ClipboardEvent) {
            item = ddp_from_event(e);
        } else {
            item = ddp_from_drop(e);
        }

        if (!item) {
            return;
        }

        on_receive(item);
    };

    export let on_receive = (e: DragDropPaste<File>) => {
        console.log(e);
    };

    // set listener and make sure it is unset on object destruction
    {
        let unlistened = false;
        let unlisten = () => {
            unlistened = true;
        };

        listen('tauri://file-drop', recv).then((un) => {
            unlisten = un;
            if (unlistened) {
                unlisten();
            }
        });

        // this func can only be called when this script is being initialised. so can't put it in .then() of listen()
        onDestroy(() => {
            unlisten();
        });
    }
</script>

<!--
https://html.spec.whatwg.org/multipage/dnd.html#dom-datatransfer-getdata

events:
  - on:drop
    - file/image drop
      - event.payload: [string] // paths/urls of files/images. url may also be "ftp://" "smb://" etc
  - on:paste
    - ctrl+v
      - text paste
        - data.getData("text/plain"): string // "text" "text/plain" "text/url-list" "url"
        - often also contains "text/html" <span>
      - file paste from filesystem
        - data.files: [File]
      - image paste from interwebs (right click + copy image)
        - data.getData("text/html") // needs parsing of the html and extracting link from <img> tag
    - text drop
      - <same as ctrl+v text paste>
    - link drops
      - "text/plain" link
      - "text/html" <span><a href=""></a></span>
      - "text/url-list" link
-->

<svelte:window
    on:dragover|preventDefault
    on:drop|preventDefault={recv}
    on:paste|preventDefault={recv}
/>
