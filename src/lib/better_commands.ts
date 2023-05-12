
import { invoke } from '@tauri-apps/api';
import type { Tag, Meta, Taggable, TypeFacet, SearchableEntry, Path, Extension, Indexed, ExtensionAction, MangaSource, SourceFilter, MangaListPage, Manga, Chapter, MultiSearchResult, ListResults } from 'types';



type Constructor<T> = new (...args: any[]) => T;

interface ISaved<T> {
    invalidate_search_results(): void;
    next_page(): Promise<RObject<T>[]>;
    set_query(q: string): Promise<RObject<T>[]>;

    search_results: RObject<T>[];
    results_valid: boolean;
    on_update(): Promise<void>;
}
function SavedSearch<T, S extends Constructor<{
    next_page(): Promise<RObject<T>[]>;
    query: string;
    reset_search(): void;
}>>(s: S) {
    return class SavedSearch extends s  implements ISaved<T> {
        search_results: Array<RObject<T>>;
        results_valid: boolean;
        async on_update(): Promise<void> {}

        constructor(...args: any[]) {
            super(...args);
            this.search_results = new Array();
            this.results_valid = false;
        }

        invalidate_search_results() {
            this.results_valid = false;
        }

        override next_page = async () => {
            if (this.results_valid) {
                let r= await super.next_page();
                this.search_results.push(...r);
                await this.on_update();
                return this.search_results;
            } else {
                this.results_valid = true;
                this.search_results =  await super.next_page();
                await this.on_update();
                return this.search_results;
            }
        }

        async set_query(q: string) {
            this.reset_search();
            this.query = q;
            this.invalidate_search_results();
            return await this.next_page();
        }

        override reset_search() {
            super.reset_search();
            this.search_results = new Array();
            this.results_valid = false;
        }
    } as S & Constructor<ISaved<T>>
}
interface IUnique<T> {
    next_page(): Promise<RObject<T>[]>;
    reset_search(): void;
}
function UniqueSearch<T, S extends Constructor<{
    next_page(): Promise<RObject<T>[]>;
    get_key(t: RObject<T>): any;
    reset_search(): void;
}>>(s: S) {
    return class UniqueSearch extends s implements IUnique<T> {
        uniq: Set<T>;
        constructor(...args: any[]) {
            super(...args);
            this.uniq = new Set();
        }

        // overridden - tho i cannot annotate it
        async next_page() {
            let r = await super.next_page();
            let items = r.filter((item) => {
                let k = this.get_key(item);
                if (this.uniq.has(k)) {
                    // collisions.push(item);
                    return false;
                } else {
                    this.uniq.add(k);
                    return true;
                }
            });
            return items;
        }

        override reset_search() {
            super.reset_search();
            this.uniq = new Set();
        }
    } as S & Constructor<IUnique<T>>
}
interface IReset {
    reset_search(): void;
}
function ResetSearch<S extends Constructor<{
    query: string;
    has_next_page: boolean;
    reset_offset(): void;
}>>(s: S) {
    return class ResetSearch extends s implements IReset {
        constructor(...args: any[]) {
            super(...args);
        }

        reset_search() {
            this.query = "";
            this.has_next_page = true;
            this.reset_offset();
        }
    } as S & Constructor<IReset>;
}
abstract class Paged<T> {
    next_page_num: number = 0;
    has_next_page: boolean = true;

    // implementor must set has_next_page
    abstract search(page: number): Promise<RObject<T>[]>;

    async next_page() {
        if (!this.has_next_page) {
            return new Array<RObject<T>>();
        }
        let r = await this.search(this.next_page_num);
        this.next_page_num += 1;
        return r;
    }

    reset_offset() {
        this.next_page_num = 0;
    }
}
abstract class Offset<T> {
    curr_offset: number = 0;
    has_next_page: boolean = true;

    // implementor must set has_next_page
    abstract search(page: number): Promise<RObject<T>[]>;

    async next_page() {
        if (!this.has_next_page) {
            return new Array<RObject<T>>();
        }
        let r = await this.search(this.curr_offset);
        this.curr_offset += r.length;
        return r;
    }

    reset_offset() {
        this.curr_offset = 0;
    }
}

class Tmdb extends Paged<MultiSearchResult> {
    query: string;
    include_adult: boolean;
    constructor() {
        super();
        this.query = "";
        this.include_adult = false;
        this.next_page_num = 1;
    }

    override reset_offset() {
        this.next_page_num = 1;
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
        return t.id;
    }
}
class Db<T> extends Offset<T> {
    facet: TypeFacet;
    query: string;
    limit: number;
    constructor(facet: TypeFacet) {
        super();
        this.query = "";
        this.limit = 50;
        this.facet = facet;
    }

    async search(offset: number) {
        let r = await invoke('search_jsml_object', {
            query: this.query,
            facet: this.facet,
            limit: this.limit,
            offset,
        }) as Array<RObject<T>>;
        if (r.length < this.limit) {
            this.has_next_page = false;
        }
        return r;
    }

    // TODO: edit this command to enter 1 item at a time
    // TODO: remove TagSearcher::add_tag
    async add_item(...items: RSearchable<T>[]) {
        await invoke('enter_searchable', {
            facet: this.facet,
            data: items,
        });
        await invoke("reload_reader");
    }
}
class TagSearch extends Offset<Tag> {
    query: string;
    limit: number;
    constructor() {
        super();
        this.query = "";
        this.limit = 50;
    }

    async search(offset: number) {
        let r = await invoke('search_tags', {
            query: this.query,
            limit: this.limit,
            offset: offset
        }) as Array<RObject<Tag>>;
        if (r.length < this.limit) {
            this.has_next_page = false;
        }
        return r;
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

    async add_item(...items: RSearchable<Tag>[]) {
        for (let tag of items) {
            let id = await this.add_tag(tag);
        }
        await invoke("reload_reader");
    }
}

export const extension_facet = { Temp: '/temp/tachi/extension' };
export const source_facet = { Temp: '/temp/tachi/source' };

class TachiClient {
    extension_searcher: RDbHandle<Extension>;
    source_searcher: RDbHandle<MangaSource>;

    set on_exts_update(callback: () => Promise<void>) {
        this.extension_searcher.on_update = callback;
    }

    constructor() {
        this.extension_searcher = new_searcher(extension_facet);
        this.source_searcher = new_searcher(source_facet);
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

class TachiExtensions extends Db<Extension> {
    constructor() {
        super(extension_facet);
    }
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
}

class TachiSources extends Db<MangaSource> {
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
}


export type DbSearcherType<T> =  Db<T> & IReset & ISaved<T>;
export function DbSearcher<T>() {
    const RS = ResetSearch(Db<T>);
    const SS = SavedSearch<T, typeof RS>(RS);
    return SS;
}

export type TagSearcherType = TagSearch & IReset & ISaved<Tag>;
export function TagSearcher() {
    const RS = ResetSearch(TagSearch);
    const SS = SavedSearch<Tag, typeof RS>(RS);
    return SS;
}

export type TmdbSearcherType = TagSearch & IReset & IUnique<Tag> & ISaved<MultiSearchResult>;
export function TmdbSearcher() {
    const RS = ResetSearch(Tmdb);
    const US = UniqueSearch<MultiSearchResult, typeof RS>(RS);
    const SS = SavedSearch<MultiSearchResult, typeof US>(US);
    return SS;
}

export type TachiExtensionSearcherType = TachiExtensions & IReset & ISaved<Extension>;
export function TachiExtensionSearcher() {
    const RS = ResetSearch(TachiExtensions);
    const SS = SavedSearch<Extension, typeof RS>(RS);
    return SS;
}

export type TachiSourceSearcherType = TachiSources & IReset & ISaved<MangaSource>;
export function TachiSourceSearcher() {
    const RS = ResetSearch(TachiSources);
    const SS = SavedSearch<MangaSource, typeof RS>(RS);
    return SS;
}

type GetReturnType<Type> = Type extends (...args: never[]) => infer Return
  ? Return
  : never;

let i = new (TmdbSearcher())();
let j = new (DbSearcher<Manga>())("Image") as RDbHandle<Manga>;
let k = new (TachiExtensionSearcher())();
k.reload();




// TypeFacet should match T
export function new_searcher<T>(facet: TypeFacet): RDbHandle<T> {
    if (facet == "Tag") {
        let s = new (TagSearcher())();
        return s as unknown as RDbHandle<T>;
    } else {
        let s = new (DbSearcher<T>())(facet);
        return s as unknown as RDbHandle<T>;
    }
}

export type RObject<T> = T extends Tag ? Meta<T, TypeFacet> : T extends MultiSearchResult ? MultiSearchResult : RObjectNotTag<T>;

export type RDbHandle<T> = T extends Tag ? TagSearcherType : DbSearcherType<T>;

export type RSearchable<T> = T extends Tag ? Tag : SearchableEntry<T>;

export type RObjectNotTag<T> = Meta<Taggable<T>, TypeFacet>;

export async function get_path(path: Path) {
    let p: string = await invoke('get_path', { path });
    // let p1 = convertFileSrc(p);
    return p;
}


class TachiSourceExplorer {
    source: MangaSource;

    constructor(source: MangaSource) {
        this.source = source;
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

class TachiMangaExplorer {
    manga: Manga;

    constructor(manga: Manga) {
        this.manga = manga;
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

class TachiChapterExplorer {
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

