import { invoke } from "@tauri-apps/api";
import type { AlbumListResult, SearchResultsOk } from "types";
import { Paged, SavedSearch, SlowSearch, UniqueSearch } from "./mixins.ts";
import type { Keyed, RObject, RSearcher } from "./searcher.ts";


export class LastFm extends Paged<AlbumListResult> {
    constructor(q: string) {
        super(q);
        this.next_page_num = 1;
    }

    static new(q: string) {
        const US = UniqueSearch<AlbumListResult, typeof LastFm>(LastFm);
        const SS = SavedSearch<AlbumListResult, typeof US>(US);
        return new SS(q);
    }

    static factory() {
        type R = RSearcher<AlbumListResult>;
        class Fac {
            include_adult: boolean = false;
            async with_query(q: string) {
                let t = LastFm.new(q);
                return t as R | null;
            }
        }
        const SS = SlowSearch<R, typeof Fac>(Fac);
        return new SS();
    }

    static obj_type() {
        return null as unknown as AlbumListResult & Keyed;
    }


    async search(page: number) {
        if (this.query.length == 0) {
            this.has_next_page = false;
            return [];
        }
        
        let r: SearchResultsOk<AlbumListResult[]> =  await invoke('lfm_search_album', {
            page: page,
            album: this.query
        });
        let k = r.matches.map(e => {
            let p = e as AlbumListResult & Keyed;
            p.get_key = function() {
                return this.url;
            };
            return p;
        })
        this.has_next_page = r.matches.length >= r.items_per_page;
        return k;
    }

    get_key(t: RObject<AlbumListResult>) {
        if (!t.url) {
            console.warn("item does not have an id :/", t);
        }
        return t.url;
    }
}
