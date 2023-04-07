import { writable } from 'svelte/store';
import type { MangaListPage } from 'types';

export let search_results = writable<MangaListPage>({ hasNextPage: false, mangaList: new Array() });
export let page_num_fetched = writable<number>(1);
export let search_query = writable('');
export let include_adult = writable(false);
