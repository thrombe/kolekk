import { invoke } from "@tauri-apps/api";
import type { ListResults, MultiSearchResult } from "types";
import { Paged, SavedSearch, SlowSearch, UniqueSearch, type ISlow } from "./mixins";
import type { RObject, RSearcher } from "./searcher";




export class Tmdb extends Paged<MultiSearchResult> {
    include_adult: boolean;
    constructor(q: string) {
        super(q);
        this.include_adult = false;
        this.next_page_num = 1;
    }

    static new(q: string) {
        const US = UniqueSearch<MultiSearchResult, typeof Tmdb>(Tmdb);
        const SS = SavedSearch<MultiSearchResult, typeof US>(US);
        return new SS(q);
    }

    static factory() {
        type R = RSearcher<MultiSearchResult>;
        class Fac {
            async with_query(q: string) {
                let t = Tmdb.new(q);
                return t as R | null;
            }
        }
        const SS = SlowSearch<R, typeof Fac>(Fac);
        return new SS();
    }

    static obj_type() {
        return null as unknown as MultiSearchResult;
    }


    async search(page: number) {
        if (this.query.length == 0) {
            this.has_next_page = false;
            return [];
        }
        
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
