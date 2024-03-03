import { invoke } from "@tauri-apps/api";
import type { Chapter, Extension, ExtensionAction, Indexed, Manga, MangaListPage, MangaSource, SourceFilter, TypeFacet } from "types";
import { Db, new_factory } from "./database";
import { Paged, SavedSearch, SlowSearch, UniqueSearch } from "./mixins";
import type { ForceDb, Keyed, RObject, RSearcher } from "./searcher";




class TachiClient {
    // extension_searcher: RSearcher<Extension>;
    // source_searcher: RSearcher<MangaSource>;

    // set on_exts_update(callback: () => Promise<void>) {
    //     this.extension_searcher.on_update = callback;
    // }

    constructor() {
        // this.extension_searcher = new_searcher(extension_facet);
        // this.source_searcher = new_searcher(source_facet);
    }

    async init() {
        await invoke('init_tachidesk_client');
    }

    static async get_all_extensions() {
        let exts: Extension[] = await invoke('tachidesk_get_all_extensions');
        return exts;
    }

    static async get_sources() {
        let sources: MangaSource[] = await invoke('tachidesk_get_source_list');
        return sources;
    }

    // async get_extension_searcher() {
    //     return this.extension_searcher;
    // }

    // async get_source_searcher() {
    //     return this.source_searcher;
    // }

    async tachidesk_action(pkgName: string, action: ExtensionAction) {
        // this await waits till the action is complete in the backend
        // https://github.com/Suwayomi/Tachidesk-WebUI/blob/d51150b7848cf7a6596bbba7c015328a578dfd16/src/components/ExtensionCard.tsx#L91
        await invoke('tachidesk_extension_action', { pkgName, action });
    };
}

const extension_facet = { Temp: '/temp/tachi/extension' };
export class TachiExtensions extends Db<Extension> {
    constructor(q: string) {
        super(extension_facet, q);
    }

    static new(q: string) {
        const SS = SavedSearch<Extension, typeof TachiExtensions>(TachiExtensions);
        return new SS(q);
    }

    static factory() {
        return new_factory<Extension>(extension_facet);
    }
    
    // TODO: reload on search if no objects. maybe use a mixin
    async reload() {
        let exts = await TachiClient.get_all_extensions();
        await invoke('delete_facet_objects', { facet: extension_facet });
        await super.add_items(
            ...exts.map((e) => {
                let searchable: Indexed[] = [{ data: e.name, field: 'Text' }];
                return { data: e, searchable };
            })
        );
    }

    static obj_type() {
        return null as unknown as RObject<ForceDb<Extension>>;
    }
}

const source_facet = { Temp: '/temp/tachi/source' };
export class TachiSources extends Db<MangaSource> {
    constructor(q: string) {
        super(source_facet, q);
    }

    static new(q: string) {
        const SS = SavedSearch<MangaSource, typeof TachiSources>(TachiSources);
        return new SS(q);
    }

    static factory() {
        return new_factory<MangaSource>(source_facet);
    }
    
    async reload() {
        let exts = await TachiClient.get_sources();
        await invoke('delete_facet_objects', { facet: source_facet });
        await super.add_items(
            ...exts.map((e) => {
                let searchable: Indexed[] = [{ data: e.name, field: 'Text' }];
                return { data: e, searchable };
            })
        );
    }

    static obj_type() {
        return null as unknown as RObject<ForceDb<MangaSource>>;
    }
}

export class TachiMangaSearch extends Paged<Manga> {
    // TODO: source: MangaSource;
    source: string;

    constructor(source: string, q: string) {
        super(q);
        this.source = source;
        this.next_page_num = 1;
    }

    static new(source: string, q: string) {
        const US = UniqueSearch<Manga, typeof TachiMangaSearch>(TachiMangaSearch);
        const SS = SavedSearch<Manga, typeof US>(US);
        return new SS(source, q);
    }

    static factory(source: string) {
        type R = RSearcher<Manga>;
        class Fac {
            source: string;
            constructor(source: string) {
                this.source = source;
            }
            
            async with_query(q: string) {
                let t = TachiMangaSearch.new(this.source, q);
                return t as R | null;
            }
        }
        const SS = SlowSearch<R, typeof Fac>(Fac);
        return new SS(source);
    }

    async search(page: number) {
        let r: MangaListPage;
        if (this.query.length == 0) {
            r = await this.get_popular_manga(page);
        } else {
            r = await this.search_manga(page, this.query);
        }
        this.has_next_page = r.hasNextPage && r.mangaList.length > 0;
        let k =  r.mangaList.map(e => {
            let p = e as Manga & Keyed;
            p.get_key = function() {
                return this.id;
            };
            return p;
        });
        return k;
    }

    get_key(t: RObject<Manga>) {
        return t.id;
    }

    static obj_type() {
        return null as unknown as Manga & Keyed;
    }

    async get_filters() {
        let filters: SourceFilter[] = await invoke('tachidesk_get_source_filters', { sourceId: this.source });
        // https://github.com/Suwayomi/Tachidesk-Server/blob/cde5dc5bfa4ce6cce6d565b41589672a754460c0/server/src/main/kotlin/suwayomi/tachidesk/manga/impl/Search.kt#L137
        // let r = await fetch(`http://localhost:4567/api/v1/source/${$page.params.src_id}/filters`, {
        //     method: 'POST',
        //     body: JSON.stringify({
        //         position: 2,
        //         state: JSON.stringify({
        //             position: 3,
        //             state: true
        //         })
        //     }),
        //     headers: {
        //         'Content-type': 'application/json'
        //     }
        // });
        // console.log(r);
        return filters;
    }

    async get_popular_manga(page: number) {
        let search_results: MangaListPage = await invoke('tachidesk_get_popular_manga_list', {
            sourceId: this.source,
            page
        });
        return search_results;
    }

    async search_manga(page: number, query: string) {
        let search_results: MangaListPage = await invoke('tachidesk_search_manga_in', {
            sourceId: this.source,
            query,
            page
        });
        return search_results;
    }
}

export class TachiChapters extends Db<Chapter> {
    manga: Manga;
    facet: TypeFacet;

    constructor(manga: Manga, q: string) {
        let facet = { Temp: "/temp/tachi/chapters/" + manga.id }
        super(facet, q);
        this.manga = manga;
        this.facet = facet;
    }

    static new(manga: Manga, q: string) {
        const SS = SavedSearch<Chapter, typeof TachiChapters>(TachiChapters);
        return new SS(manga, q);
    }

    static factory() {
        return new_factory<Manga>(source_facet);
    }

    static obj_type() {
        return null as unknown as RObject<ForceDb<Chapter>>;
    }
    
    async reload() {
        let r = await this.get_chapters();
        await invoke('delete_facet_objects', { facet: this.facet });
        await super.add_items(
            ...r.map((e) => {
                let searchable: Indexed[] = [{ data: e.name, field: 'Text' }];
                return { data: e, searchable };
            })
        );
    }


    static async get(mangaId: number) {
        let manga: Manga = await invoke('tachidesk_get_manga', { mangaId });
        return manga;
    }

    async get_chapters() {
        let chapters: Chapter[] = await invoke('tachidesk_get_manga_chapter_list', { mangaId: this.manga.id });
        return chapters;
    }

    async get_chapter(chapterIndex: number) {
        let chapter: Chapter = await invoke('tachidesk_get_chapter', {
            mangaId: this.manga.id,
            chapterIndex
        });
        return chapter;
    }
}

export class TachiChapterExplorer {
    chapter: Chapter;

    constructor(chapter: Chapter) {
        this.chapter = chapter;
    }

    async get_page_url(page: number) {
        let uri: string = await invoke('tachidesk_get_manga_page_url', {
            mangaId: this.chapter.mangaId,
            chapterIndex: this.chapter.index,
            page
        });
        return uri;
    }
}
