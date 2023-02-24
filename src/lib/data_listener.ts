import { BaseDirectory, writeBinaryFile } from '@tauri-apps/api/fs';
import type { ByteArrayFile, DragDropPaste } from 'types';

export interface DropEvent {
    event: string;
    windowLabel: string;
    payload: string[];
    id: number;
}

export const ddp_from_drop = (e: DropEvent): DragDropPaste<File> => {
    let ddp: DragDropPaste<File> = {
        file_uris: e.payload,
        text: null,
        text_html: null,
        files: null,
        uri_list: null
    };
    return ddp;
};

export const ddp_from_event = (e: ClipboardEvent | DragEvent): DragDropPaste<File> | null => {
    let data: DataTransfer | null;
    if (e instanceof DragEvent) {
        // when text is dropped on the screen
        data = e.dataTransfer;
    } else {
        // when stuff is pasted using ctrl-v
        data = e.clipboardData;
    }
    if (!data) {
        return null;
    }

    let text = data.getData('text/plain');
    let text_html = data.getData('text/html');
    let files = [...data.files];
    let uri_list = data.getData('text/uri-list');

    let ddp: DragDropPaste<File> = {
        file_uris: null,
        text: text ? text : null,
        text_html: text_html ? text_html : null,
        files: files.length > 0 ? files : null,
        uri_list: uri_list ? uri_list : null
    };

    return ddp;
};

export const files_to_bytearrays = async (
    e: DragDropPaste<File>
): Promise<DragDropPaste<ByteArrayFile>> => {
    let files = Array();

    if (e.files) {
        for (let file of e.files) {
            let data = await file.stream().getReader().read();
            if (!data.done) {
                let baf = { name: file.name, data: Array.from(data.value) };
                files = [...files, baf];
            }
        }
    }

    return {
        text: e.text,
        files,
        text_html: e.text_html,
        uri_list: e.uri_list,
        file_uris: e.file_uris
    };
};

export const _files_to_disk = async (e: DragDropPaste<File>): Promise<DragDropPaste<string>> => {
    // e.file_uris = Array();
    let files = Array();
    if (e.files) {
        for (let file of e.files) {
            // TODO:
            writeBinaryFile(file.name, await file.slice().arrayBuffer(), {
                dir: BaseDirectory.AppCache
            });
            // e.file_uris = [...e.file_uris, file.name]; // !!! need to change file name to full path
        }
    }
    return {
        text: e.text,
        file_uris: e.file_uris,
        text_html: e.text_html,
        files,
        uri_list: e.uri_list
    };
};
