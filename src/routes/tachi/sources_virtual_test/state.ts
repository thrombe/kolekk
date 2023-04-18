import { writable, type Writable } from 'svelte/store';
import type { MangaSource } from 'types';

export let sources: Writable<MangaSource[]> = writable(new Array());
