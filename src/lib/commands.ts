
import { invoke } from '@tauri-apps/api';
import type { Tag, Meta, Taggable, TypeFacet, SearchableEntry, Path, Extension, Indexed, ExtensionAction, MangaSource, SourceFilter, MangaListPage, Manga, Chapter } from 'types';

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
        await this.on_update(this as unknown as RSearcher<T>);
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


const extensions_facet = { Temp: '/temp/tachi/extension' };

class TachiClient {
    is_init: boolean;
    ext_searcher: RSearcher<Extension>;

    set on_exts_update(callback: (e: RSearcher<Extension>) => Promise<void>) {
        this.ext_searcher.on_update = callback;
    }

    constructor() {
        this.is_init = false;
        this.ext_searcher = new_searcher(extensions_facet, 50);
    }

    async init() {
        await invoke('init_tachidesk_client');
        this.is_init = true;
    }

    async get_all_extensions() {
        let exts: Extension[] = await invoke('tachidesk_get_all_extensions');
        return exts;
    }

    async get_extension_searcher() {
        let exts = await this.get_all_extensions();
        await invoke('delete_facet_objects', { facet: extensions_facet });
        await this.ext_searcher.add_item(
            ...exts.map((e) => {
                let searchable: Indexed[] = [{ data: e.name, field: 'Text' }];
                return { data: e, searchable };
            })
        );
        return this.ext_searcher;
    }

    async tachidesk_action(pkgName: string, action: ExtensionAction) {
        // this await waits till the action is complete in the backend
        // https://github.com/Suwayomi/Tachidesk-WebUI/blob/d51150b7848cf7a6596bbba7c015328a578dfd16/src/components/ExtensionCard.tsx#L91
        await invoke('tachidesk_extension_action', { pkgName, action });
    };

    async get_sources() {
        let sources: MangaSource[] = await invoke('tachidesk_get_source_list');
        return sources;
    }

    async get_source_filters(sourceId: string) {
        let filters: SourceFilter[] = await invoke('tachidesk_get_source_filters', { sourceId })
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

    async get_popular_manga(sourceId: string, page: number) {
        let search_results: MangaListPage = await invoke('tachidesk_get_popular_manga_list', {
            sourceId,
            page
        });
        return search_results;
    }

    async search_manga(sourceId: string, page: number, query: string) {
        let search_results: MangaListPage = await invoke('tachidesk_search_manga_in', {
            sourceId,
            query,
            page
        });
        return search_results;
    }

    async get_manga(mangaId: number) {
        let manga: Manga = await invoke('tachidesk_get_manga', { mangaId });
        return manga;
    }

    async get_chapters(mangaId: number) {
        let chapters: Chapter[] = await invoke('tachidesk_get_manga_chapter_list', { mangaId });
        return chapters;
    }

    async get_chapter(mangaId: number, chapterIndex: number) {
        let chapter: Chapter = await invoke('tachidesk_get_chapter', {
            mangaId,
            chapterIndex
        });
        return chapter;
    }

    async get_manga_page_url(chapter: Chapter, page: number) {
        let uri: string = await invoke('tachidesk_get_manga_page_url', {
            mangaId: chapter.mangaId,
            chapterIndex: chapter.index,
            page
        });
        return uri;
    }
}
