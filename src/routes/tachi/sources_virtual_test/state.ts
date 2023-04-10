import { writable, type Writable } from 'svelte/store';
import type { Source } from 'types';

export let sources: Writable<Source[]> = writable(new Array());
