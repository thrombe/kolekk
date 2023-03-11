import { writable, type Writable } from 'svelte/store';
import type { ListResults, MultiSearchResult } from 'types';

export let search_results: Writable<ListResults<MultiSearchResult>> = writable({
    results: new Array(),
    page: null,
    total_results: null,
    total_pages: null
});
export let search_query = writable('');
export let include_adult = writable(false);
