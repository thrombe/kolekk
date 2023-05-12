
import type { Tag, Meta, Taggable, TypeFacet, SearchableEntry, MultiSearchResult, Extension, MangaSource, Manga, Chapter } from 'types';
import { TagSearch, Db, new_db, db_obj_type } from './database';
import type { TachiExtensions, TachiChapters, TachiSources, TachiMangaSearch, TachiChapterExplorer } from './tachi';
import { Tmdb } from './tmdb';

type ReturnType<Type> = Type extends (...args: never[]) => infer R ? R : never;


export { TagSearch, Tmdb, Db, TachiExtensions, TachiSources, TachiMangaSearch, TachiChapterExplorer, new_db };


// this should onlybe used for the type parameter in RObject and RSearcher
export interface ForceDb<_> {
    force: null;
}

export type RObject<T> =
    // a hacky way to force match this
    T extends ForceDb<infer E>
    ? ReturnType<typeof db_obj_type<E>>

    : T extends Tag
    ? ReturnType<typeof TagSearch.obj_type>

    : T extends MultiSearchResult
    ? ReturnType<typeof Tmdb.obj_type>

    : T extends Extension
    ? ReturnType<typeof TachiExtensions.obj_type>

    : T extends MangaSource
    ? ReturnType<typeof TachiSources.obj_type>

    : T extends Manga
    ? ReturnType<typeof TachiMangaSearch.obj_type>

    : ReturnType<typeof db_obj_type<T>>;


export type RSearcher<T> =
    // a hacky way to force match this
    T extends ForceDb<infer E>
    ? ReturnType<typeof new_db<E>>

    : T extends Tag
    ? ReturnType<typeof TagSearch.new>

    : T extends MultiSearchResult
    ? ReturnType<typeof Tmdb.new>

    : T extends Extension
    ? ReturnType<typeof TachiExtensions.new>

    : T extends MangaSource
    ? ReturnType<typeof TachiSources.new>

    : T extends Manga
    ? ReturnType<typeof TachiMangaSearch.new>

    : T extends Chapter
    ? ReturnType<typeof TachiChapters.new>

    : ReturnType<typeof new_db<T>>


export type RDbEntry<T> = T extends Tag ? Tag : SearchableEntry<T>;
