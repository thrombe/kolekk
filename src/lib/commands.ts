
import { invoke } from '@tauri-apps/api';
import type { Path } from 'types';


export async function get_path(path: Path) {
    let p: string = await invoke('get_path', { path });
    // let p1 = convertFileSrc(p);
    return p;
}
