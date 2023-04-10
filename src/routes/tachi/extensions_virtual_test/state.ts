import { writable, type Writable } from 'svelte/store';
import type { Extension } from 'types';

interface Item<E, K> {
  data: E,
  id: K,
}
export let extensions: Writable<Item<Extension, string>[]> = writable(new Array());
