<script lang="ts">
	import { listen } from '@tauri-apps/api/event';

	interface DragDropPaste {
    // priority in the same order
    file_uris: [string] | null; // "http://" "ftp://" "smb://" "/home/"
    text: string | null; // anything. links, just text, whatever
    text_html: string | null; // <img href=""> <span>
    files: [File] | null;

    uri_list: string | null; // link drops. (link is also available in self.text)
  };

	export let file_drop = (e: any) => {
		console.log(e);
		console.log(e.payload);
	};
	export let on_paste = (e: any) => {
		let data: DataTransfer;
		if (e.dataTransfer) {
		  // when text is dropped on the screen
			data = e.dataTransfer;
		} else {
		  // when stuff is pasted using ctrl-v
			data = e.clipboardData;
		}
		console.log(data);
		console.log(data.getData('text/plain'));
		console.log(data.getData('text/html'));
		console.log(data.getData('text/uri-list'));
		console.log(data.files);
		console.log(data.files[0]);
	};

	listen('tauri://file-drop', file_drop);
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
	on:drop|preventDefault={on_paste}
	on:paste|preventDefault={on_paste}
/>
