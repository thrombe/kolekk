
import { invoke } from '@tauri-apps/api';
import type { Path } from 'types';


export async function get_path(path: Path) {
    let p: string = await invoke('get_path', { path });
    // let p1 = convertFileSrc(p);
    return p;
}

export async function logg(...objects: Array<string | Object | null | undefined>) {
    for (let o of objects) {
        if (typeof o === 'string') {
            await logg_string(o);
        } else if (o) {
            await logg_jsml(o);
        }
    }
}
export async function logg_string(...str: string[]) {
    for (let s of str) {
        await invoke('logg_string', { string: s });
    }
}
export async function logg_jsml(...objects: Object[]) {
    for (let jsml of objects) {
        await invoke('logg_jsml', { jsml });
    }
}
