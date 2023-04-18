import { writable, type Writable } from 'svelte/store';
import type { Extension, JsonObject } from 'types';

interface Item<E, K> {
  data: E,
  id: K,
}
export let extensions: Writable<Item<JsonObject<Extension>, any>[]> = writable(new Array());
export let search_query = writable('');
