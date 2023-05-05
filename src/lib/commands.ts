
import { invoke } from '@tauri-apps/api';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import type { Tag, Meta, Taggable, TypeFacet, SearchableEntry, Path } from 'types';

export class Searcher<T> {
    _limit: number;
    _query: string;
    facet: TypeFacet;

    _results_valid: boolean;
    _has_next_page: boolean;
    search_results: Array<RObject<T>>;
    on_update: any;

    // TypeFacet should match T
    constructor(facet: TypeFacet, page_size: number, on_update = (_e: any) => {}) {
        this.facet = facet;
        this._query = '';
        this._limit = page_size;
        this._results_valid = false;
        this._has_next_page = false;
        this.search_results = [];
        this.on_update = on_update;
    }

    get query() {
        return this._query;
    }

    async set_query(q: string) {
        this._query = q;
        this._results_valid = false;
        return await this.next_page();
    }

    // get facet() {
    //     return this._facet;
    // }

    // set facet(f: TypeFacet) {
    //     this._facet = f;
    //     this._results_valid = false;
    //     this.next_page();
    // }

    invalidate_search_results() {
        this._results_valid = false;
    }

    async reload_reader() {
        await invoke("reload_reader");
    }

    async add_item(...items: SearchableEntry<T>[]) {
        await enter_searchable<T>(this.facet, items);
        this.invalidate_search_results();
        return await this.next_page();
    }

    async next_page() {
        await this.reload_reader();
        if (this._results_valid) {
            if (!this._has_next_page) {
                return this.search_results;
            }
            let search_results = await search<T>(
                this._query,
                this.facet,
                this._limit,
                this.search_results.length,
            );
            if (search_results.length != this._limit) {
                this._has_next_page = false;
            }
            this.search_results.push(...search_results);
        } else {
            let search_results = await search<T>(
                this._query,
                this.facet,
                this._limit,
                0,
            );

            this._has_next_page = true;
            if (search_results.length != this._limit) {
                this._has_next_page = false;
            }
            this._results_valid = true;
            this.search_results = search_results;
        }
        await this.on_update(this);
        return this.search_results;
    }
}

// TODO: searching for Tag should call 'search_tags' command
export type RObject<T> = T extends Tag ? Meta<T, TypeFacet> : Meta<Taggable<T>, TypeFacet>;

export async function search<T>(
    query: string,
    facet: TypeFacet,
    limit: number,
    offset: number,
) {
    return await invoke('search_jsml_object', {
        query,
        facet,
        limit,
        offset,
    }) as Array<RObject<T>>;
}

export async function enter_searchable<T>(facet: TypeFacet, data: SearchableEntry<T>[]) {
    await invoke('enter_searchable', {
        facet,
        data,
    })
}

export async function get_path(path: Path) {
    let p: string =  await invoke('get_path', { path });
    // let p1 = convertFileSrc(p);
    return p;
}
