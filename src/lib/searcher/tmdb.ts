import { invoke } from "@tauri-apps/api";
import type { ListResults, MultiSearchResult } from "types";
import { Paged, ResetSearch, SavedSearch, UniqueSearch } from "./mixins";
import type { RObject } from "./searcher";




export class Tmdb extends Paged<MultiSearchResult> {
    query: string;
    include_adult: boolean;
    constructor() {
        super();
        this.query = "";
        this.include_adult = false;
        this.next_page_num = 1;
    }

    static new() {
        const RS = ResetSearch(Tmdb);
        const US = UniqueSearch<MultiSearchResult, typeof RS>(RS);
        const SS = SavedSearch<MultiSearchResult, typeof US>(US);
        return new SS();
    }

    override reset_offset() {
        this.next_page_num = 1;
    }

    static obj_type() {
        return null as unknown as MultiSearchResult;
    }


    async search(page: number) {
        let r: ListResults<MultiSearchResult> =  await invoke('search_tmdb_multi', {
            query: this.query,
            page: page,
            includeAdult: this.include_adult
        });
        this.has_next_page = page < (r.total_pages ? r.total_pages : 0);
        return r.results;
    }

    get_key(t: RObject<MultiSearchResult>) {
        if (!t.id) {
            console.warn("item does not have an id :/", t);
        }
        return t.id;
    }
}
