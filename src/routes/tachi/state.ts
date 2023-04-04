import { writable, type Writable } from 'svelte/store';
import type { Extension } from 'types';

export let extensions: Writable<Extension[]> = writable(new Array());
