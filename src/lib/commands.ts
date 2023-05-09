
import { invoke } from '@tauri-apps/api';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import type { Tag, Meta, Taggable, TypeFacet, SearchableEntry, Path } from 'types';

// TypeFacet should match T
export function new_searcher<T>(facet: TypeFacet, page_size: number): RSearcher<T> {
    if (facet == "Tag") {
        return new TagSearcher(page_size) as RSearcher<T>;
    } else {
        return new Searcher<T>(facet, page_size) as RSearcher<T>;
    }
}

export type RObject<T> = T extends Tag ? Meta<T, TypeFacet> : RObjectNotTag<T>;

export type RSearcher<T> = T extends Tag ? TagSearcher : Searcher<T>;

export type RSearchable<T> = T extends Tag ? Tag : SearchableEntry<T>;

export type RObjectNotTag<T> = Meta<Taggable<T>, TypeFacet>;

class Searcher<T> {
    _limit: number;
    _query: string;
    facet: TypeFacet;

    _results_valid: boolean;
    _has_next_page: boolean;
    search_results: Array<RObject<T>>;
    on_update: (s: RSearcher<T>) => Promise<void>;

    constructor(facet: TypeFacet, page_size: number) {
        this.facet = facet;
        this._query = '';
        this._limit = page_size;
        this._results_valid = false;
        this._has_next_page = false;
        this.search_results = [];
        this.on_update = async (_e: any) => { };
    }

    get query() {
        return this._query;
    }

    async set_query(q: string) {
        this._query = q;
        this._results_valid = false;
        return await this.next_page();
    }

    invalidate_search_results() {
        this._results_valid = false;
    }

    async reload_reader() {
        await invoke("reload_reader");
    }

    // TODO: edit this command to enter 1 item at a time
    // TODO: remove TagSearcher::add_tag
    async add_item(...items: RSearchable<T>[]) {
        await invoke('enter_searchable', {
            facet: this.facet,
            data: items,
        });
        this.invalidate_search_results();
        return await this.next_page();
    }

    async next_page() {
        await this.reload_reader();
        if (this._results_valid) {
            if (!this._has_next_page) {
                return this.search_results;
            }
            let search_results = await this.search(this.search_results.length);
            if (search_results.length != this._limit) {
                this._has_next_page = false;
            }
            this.search_results.push(...search_results);
        } else {
            let search_results = await this.search(0);

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

    async search(offset: number) {
        return await invoke('search_jsml_object', {
            query: this._query,
            facet: this.facet,
            limit: this._limit,
            offset,
        }) as Array<RObject<T>>;
    }
}

class TagSearcher extends Searcher<Tag> {
    constructor(page_size: number) {
        super("Tag", page_size);
    }

    async search(offset: number) {
        let searched_tags = await invoke('search_tags', {
            query: this._query,
            limit: this._limit,
            offset: offset
        });
        return searched_tags as RObject<Tag>[];
    }

    async add_item(...items: RSearchable<Tag>[]) {
        for (let tag of items) {
            let id = await this.add_tag(tag);
        }
        this.invalidate_search_results();
        return await this.next_page();
    }



    async add_tag(tag: Tag) {
        let id: number = await invoke('save_new_tag', { tag });
        return id;
    }

    async add_tag_to_object(id: number, tag_id: number) {
        await invoke('add_tag_to_object', { id, tagId: tag_id });
    }

    async get_tags_from_ids(...ids: number[]) {
        return await invoke<[RObject<Tag>]>('get_tags_from_ids', { ids });
    }

    async remove_tag_from_object(id: number, tag_id: number) {
        await invoke('remove_tag_from_object', { id, tagId: tag_id });
    }
}



export async function get_path(path: Path) {
    let p: string = await invoke('get_path', { path });
    // let p1 = convertFileSrc(p);
    return p;
}

