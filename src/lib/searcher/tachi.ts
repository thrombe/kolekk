import { invoke } from "@tauri-apps/api";
import type { Chapter, Extension, ExtensionAction, Indexed, Manga, MangaListPage, MangaSource, SourceFilter, TypeFacet } from "types";
import { Db } from "./database";
import { Paged, QuerySet, ResetSearch, SavedSearch, SlowSearch, UniqueSearch } from "./mixins";
import type { ForceDb, RObject } from "./searcher";




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
    constructor() {
        super(extension_facet);
    }

    static new() {
        const RS = ResetSearch(TachiExtensions);
        const QS = QuerySet<Extension, typeof RS>(RS);
        const SS = SavedSearch<Extension, typeof QS>(QS);
        return new SS();
    }
    
    // TODO: reload on search if no objects. maybe use a mixin
    async reload() {
        let exts = await TachiClient.get_all_extensions();
        await invoke('delete_facet_objects', { facet: extension_facet });
        await super.add_item(
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
    constructor() {
        super(extension_facet);
    }

    static new() {
        const RS = ResetSearch(TachiSources);
        const QS = QuerySet<MangaSource, typeof RS>(RS);
        const SS = SavedSearch<MangaSource, typeof QS>(QS);
        return new SS();
    }

    async reload() {
        let exts = await TachiClient.get_sources();
        await invoke('delete_facet_objects', { facet: source_facet });
        await super.add_item(
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
    source: MangaSource;
    query: string;

    constructor(source: MangaSource) {
        super();
        this.source = source;
        this.query = '';
    }

    static new(source: MangaSource) {
        const RS = ResetSearch(TachiMangaSearch);
        const QS = QuerySet<Manga, typeof RS>(RS);
        const US = UniqueSearch<Manga, typeof QS>(QS);
        const SL = SlowSearch<Manga, typeof US>(US);
        const SS = SavedSearch<Manga, typeof SL>(SL);
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
        return r.mangaList;
    }

    override reset_offset() {
        this.next_page_num = 1;
    }

    get_key(t: RObject<Manga>) {
        return t.id;
    }

    static obj_type() {
        return null as unknown as Manga;
    }


    async get_filters() {
        let filters: SourceFilter[] = await invoke('tachidesk_get_source_filters', { sourceId: this.source.id });
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
            sourceId: this.source.id,
            page
        });
        return search_results;
    }

    async search_manga(page: number, query: string) {
        let search_results: MangaListPage = await invoke('tachidesk_search_manga_in', {
            sourceId: this.source.id,
            query,
            page
        });
        return search_results;
    }
}

export class TachiChapters extends Db<Chapter> {
    manga: Manga;
    query: string;
    facet: TypeFacet;

    constructor(manga: Manga) {
        let facet = { Temp: "/temp/tachi/chapters/" + manga.id }
        super(facet);
        this.manga = manga;
        this.query = '';
        this.facet = facet;
    }

    static new(manga: Manga) {
        const RS = ResetSearch(TachiChapters);
        const QS = QuerySet<Chapter, typeof RS>(RS);
        const SS = SavedSearch<Chapter, typeof QS>(QS);
        return new SS(manga);
    }

    async reload() {
        let r = await this.get_chapters();
        await invoke('delete_facet_objects', { facet: this.facet });
        await super.add_item(
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
