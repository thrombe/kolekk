import { invoke } from "@tauri-apps/api";
import type { Meta, Tag, Taggable, TypeFacet } from "types";
import { Offset, SavedSearch } from "./mixins";
import type { RObject, RDbEntry } from "./searcher";



// facet and T should match
export function new_db<T>(facet: TypeFacet, q: string) {
    const SS = SavedSearch<T, typeof Db<T>>(Db<T>);
    return new SS(facet, q);
}
export function new_factory<T>(facet: TypeFacet) {
    class Fac {
        facet: TypeFacet;
        constructor(facet: TypeFacet) {
            this.facet = facet;
        }

        async with_query(q: string) {
            let t = new_db<T>(this.facet, q);
            return t;
        }
    }
    return new Fac(facet);
}

export function db_obj_type<T>() {
    return null as unknown as Meta<Taggable<T>, TypeFacet>;
}

export class Db<T> extends Offset<T> {
    facet: TypeFacet;
    limit: number;
    constructor(facet: TypeFacet, q: string) {
        super(q);
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
    async add_item(...items: RDbEntry<T>[]) {
        await invoke('enter_searchable', {
            facet: this.facet,
            data: items,
        });
        await invoke("reload_reader");
    }
}


export class TagSearch extends Offset<Tag> {
    limit: number;
    constructor(q: string) {
        super(q);
        this.limit = 50;
    }

    static new(q: string) {
        const SS = SavedSearch<Tag, typeof TagSearch>(TagSearch);
        return new SS(q);
    }

    static factory() {
        class Fac {
            async with_query(q: string) {
                return TagSearch.new(q);
            }
        }
        return new Fac();
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

    static obj_type() {
        return null as unknown as Meta<Tag, TypeFacet>;
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

    async add_item(...items: RDbEntry<Tag>[]) {
        for (let tag of items) {
            let id = await this.add_tag(tag);
        }
        await invoke("reload_reader");
    }
}

