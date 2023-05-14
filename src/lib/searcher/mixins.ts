import type { RObject } from "./searcher";




type Constructor<T> = new (...args: any[]) => T;

const sleep = (ms: number) => {
    return new Promise(
        (r) => setTimeout(r, ms)
    )
};



export interface ISaved<T> {
    invalidate_search_results(): void;
    next_page(): Promise<RObject<T>[]>;
    set_query(q: string): Promise<RObject<T>[]>;

    search_results: RObject<T>[];
    results_valid: boolean;
    on_update(): Promise<void>;
}
export function SavedSearch<T, S extends Constructor<{
    next_page(): Promise<RObject<T>[]>;
    set_query(q: string): Promise<RObject<T>[]>;
    reset_search(): void;
    query: string;
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
            this.invalidate_search_results();
            return await super.set_query(q);
        }

        override reset_search() {
            super.reset_search();
            this.search_results = new Array();
            this.results_valid = false;
        }
    } as S & Constructor<ISaved<T>>
}


export interface ISlow<T> {
    last_search: number;
    search_generation: number;
    set_query(q: string): Promise<RObject<T>[]>;
}
export function SlowSearch<T, S extends Constructor<{
    query: string;
    set_query(q: string): Promise<RObject<T>[]>;
}>>(s: S) {
    return class SlowSearch extends s implements ISlow<T> {
        last_search: number;
        search_generation: number;
        constructor(...args: any[]) {
            super(...args);
            this.last_search = 0;
            this.search_generation = 0;
        }

        async set_query(q: string) {
            this.search_generation += 1;
            let current_generation = this.search_generation;
            let del = 500;
            let now = Date.now();
            if (now - this.last_search < del) {
                await sleep(del);
            }

            // some other (concurrent) call to this method may change current_generation
            if (this.search_generation == current_generation) {
                this.last_search = Date.now();
                let r = await super.set_query(q);

                // to make sure that latest searches are not overwritten by searches that started earlier
                if (this.search_generation == current_generation) {
                    return r;
                }
            }
            return new Array<RObject<T>>();
        }
    } as S & Constructor<ISlow<T>>
}


export interface IUnique<T> {
    next_page(): Promise<RObject<T>[]>;
    reset_search(): void;
}
export function UniqueSearch<T, S extends Constructor<{
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

export interface IQuerySet<T> {
    set_query(q: string): Promise<RObject<T>[]>;
}
export function QuerySet<T, S extends Constructor<{
    next_page(): Promise<RObject<T>[]>;
    query: string;
}>>(s: S) {
    return class QuerySet extends s implements IQuerySet<T> {
        async set_query(q: string) {
            this.query = q;
            return await this.next_page();
        }
    } as S & Constructor<IQuerySet<T>>
}

export interface IReset {
    reset_search(): void;
}
export function ResetSearch<S extends Constructor<{
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


export abstract class Paged<T> {
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


export abstract class Offset<T> {
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
