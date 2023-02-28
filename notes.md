

- issues
  - [tauri no audioplayback](https://github.com/tauri-apps/tauri/issues/3478)
    - new Audio(svelte::convertFileSrc(localpath_or_songuri)).play().then(() => {})
    - [how to Audio + <input type=range>](https://medium.com/@tommarren/create-a-custom-audio-progress-bar-using-javascript-51b358811abd)
    - [playing sound works? glitch](https://github.com/tauri-apps/tauri/discussions/5687)
  - [page flicker on resize](https://github.com/tauri-apps/tauri/issues/1564)
  - [`assetScope` suddenly no longer working?](https://github.com/tauri-apps/tauri/issues/6158)


- [initial setup](https://tauri.app/v1/guides/getting-started/setup/sveltekit/):
  - npm create svelte@latest
  - npm install --save-dev @sveltejs/adapter-static@next
  - cargo tauri init
  - cargo tauri dev

- bindgen stuff
  - [ts-rs](https://crates.io/crates/ts-rs)
  - [typescript-definitions](https://crates.io/crates/typescript-definitions)
  - [tauri issue for code gen](https://github.com/tauri-apps/tauri/issues/1514)
  - [tauri bindgen](https://github.com/tauri-apps/tauri-bindgen)
  - [tauri-specta](https://github.com/oscartbeaumont/tauri-specta)

- links
  - https://svelte.dev/docs#run-time-client-side-component-api
  - https://svelte.dev/tutorial/await-blocks
  - https://kit.svelte.dev/docs/load
  - https://tauri.app/v1/guides/getting-started/setup/sveltekit
  - https://tauri.app/v1/guides/features/command
  - https://tauri.app/v1/references/architecture/inter-process-communication/

- projects
  - [pretty big tauri app](https://github.com/kkoomen/pointless)
  - [good svelte, big](https://github.com/iohzrd/identia)
  - [nice ffi](https://github.com/zhanglun/lettura)
  - [nice ffi. svelte](https://github.com/probablykasper/mr-tagger)
  - [nice ffi. svelte, tauri::State](https://github.com/probablykasper/remind-me-again)
  - [nice ffi. svelte, macos. tauri::State](https://github.com/probablykasper/time-machine-inspector)
  - [massive virtual fs app (partly tauri)](https://github.com/spacedriveapp/spacedrive)
  - [music player svelte tauri](https://github.com/basharovV/musicat)
    - [blog tut](https://vyacheslavbasharov.com/blog/building-music-player-tauri-svelte)
  - [chat gpt tauri app (big-ish)](https://github.com/lencx/ChatGPT)


- api stuff
  - imdb (does not provide a search api apparently)
    - [cinemagoer: popular, in-use, provides all info, python, (slow?)](https://github.com/cinemagoer/cinemagoer)
    - [seach into imdb provided databases](https://crates.io/crates/imdb-async)
    - [tvrank. ''](https://crates.io/crates/tvrank)
    - [fast offline index](https://docs.rs/imdb-index/latest/imdb_index/)
  - mal
    - [lib-mal](https://crates.io/crates/lib-mal)
      - [smol bug](https://github.com/AnActualEmerald/lib-mal/blob/8dac0b18535d776b861da4bee8d700e424ed1017/src/client.rs#L635)
    - [myanimelist-rs](https://crates.io/crates/myanimelist-rs)
    - [mal official api refs](https://myanimelist.net/clubs.php?cid=13727)
    - [mal official api v2 examples](https://myanimelist.net/apiconfig/references/api/v2)
    - [mal manage api keys settings page](https://myanimelist.net/apiconfig)
  - tachiyomi/tachidesk extensions
    - [tachiyomiorg](https://github.com/tachiyomiorg)
    - [tachiyomi-extensions](https://github.com/tachiyomiorg/tachiyomi-extensions)
    - [Tachidesk-Server](https://github.com/Suwayomi/Tachidesk-Server)
    - [Tachidesk-WebUI](https://github.com/Suwayomi/Tachidesk-WebUI)
    - [Tachidesk-Sorayomi](https://github.com/Suwayomi/Tachidesk-Sorayomi)
  - selenium
    - [thirtyfour](https://github.com/stevepryde/thirtyfour)
    - [selenium-rs](https://github.com/saresend/selenium-rs)

- tauri stuff
  - [tauri js api](https://tauri.app/v1/api/js/)
  - [gh discussions](https://github.com/tauri-apps/tauri/discussions)
  - [no serialise ipc data transfer](https://github.com/JonasKruckenberg/pisano/blob/main/src-tauri/src/main.rs)
  - [tauri + wasm gui](https://github.com/tauri-apps/tauri/discussions/5231)
  - drag and drop
    - [files into](https://github.com/tauri-apps/tauri/issues/2768#issuecomment-997169108)
    - [files out](https://github.com/tauri-apps/tauri/issues/2593)
  - clipboard
    - [arboard](https://docs.rs/arboard)
    - xclip -selection clipboard -t image/png <path>
    - [tauri-plugin-clipboard](https://crates.io/crates/tauri-plugin-clipboard)
  - splashscreen
    - [tauri splashscreen example](https://github.com/tauri-apps/tauri/blob/dev/examples/splashscreen/main.rs)
    - [Splashscreen](https://tauri.app/v1/guides/features/splashscreen/)

- backend stuff
  - [database normalisation wiki](https://en.wikipedia.org/wiki/Database_normalization)
  - [seaorm entity structure docs](https://www.sea-ql.org/SeaORM/docs/generate-entity/entity-structure/)
  - db
    - [lib.rs database-implementations](https://lib.rs/database-implementations)
    - [lib.rs database](https://lib.rs/database)
    - [sled](https://sled.rs/introduction.html)
    - rdbms
      - [sqlx compile time checking](https://lib.rs/crates/sqlx)
      - [postgres](https://rust-lang-nursery.github.io/rust-cookbook/database/postgres.html)
      - [rusqlite sqlite wrapper](https://lib.rs/crates/rusqlite)
    - orm (object-relational mapping)
      - [sea-orm sqlx+orm](https://github.com/SeaQL/sea-orm)
        - [docs tuts cookbook](https://www.sea-ql.org/SeaORM/docs/index/)
      - [rbatis](https://lib.rs/crates/rbatis)
      - [diesel](https://lib.rs/crates/diesel)
      - [ormx sqlx+lightweight orm](https://lib.rs/crates/ormx)
  - search engine
    - [tantivy](https://lib.rs/crates/tantivy)
    - [meilisearch](https://lib.rs/gh/meilisearch/meilisearch/meilisearch-types)
      - [docs](https://docs.meilisearch.com/learn/getting_started/quick_start.html#securing-meilisearch)
  - [fuzzy string stuff](https://lib.rs/search?q=fuzzy)
    - [skim](https://lib.rs/crates/skim)
    - [fzyr](https://crates.io/crates/fzyr)
    - [C fzy](https://github.com/jhawthorn/fzy)
    - [scout](https://crates.io/crates/scout)
    - [Go fzf](https://github.com/junegunn/fzf)
    - [fzy's algorithm.md](https://github.com/jhawthorn/fzy/blob/master/ALGORITHM.md)

- svelte/js stuff
  - [generate typescript types from rust](https://dev.to/alexeagleson/how-to-build-a-rust-cli-tool-to-generate-typescript-types-from-rust-20cm)
  - [synchronizing ts and rs types](https://imfeld.dev/writing/generating_typescript_types_from_rust)
    - in build.rs?
      - can move all the required types in a seperate crate and then use that in build.rs with some crate to convert the type to ts
        - ts_rs has the required functions in the trait TS
      /- maybe modify some of this in [ts_rs](https://github.com/Aleph-Alpha/ts-rs/blob/b4ba8b81fd8833296e99285eae7608864c52e51e/macros/src/lib.rs#L41)
        /- remove the #[test] stuff and use the generated function in build.rs to output the types to another directory
  - drag and drop
    - [mdn html drag and drop api](https://developer.mozilla.org/en-US/docs/Web/API/HTML_Drag_and_Drop_API)
  - lazy loading
    - [modern browsers + simple svelte](https://dev.to/collardeau/lazy-loading-images-in-svelte-1mk6)
    - [online repl + simple + svelte](https://svelte.dev/repl/adb8dc564044415f8ffbbd240a39d68d?version=3.44.2)
    - [modern + old browsers svelte](https://css-tricks.com/lazy-loading-images-in-svelte/)
  - CORS
    - [mdn CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS)
    - CORS proxy
      - [how it be](https://httptoolkit.com/blog/cors-proxies/)
      - [gh gist of proxies](https://gist.github.com/jimmywarting/ac1be6ea0297c16c477e17f8fbe51347)
      - [cloudflare cors proxy demo gh](https://github.com/Zibri/cloudflare-cors-anywhere)
      - [cors-anywhere herokuapp](https://cors-anywhere.herokuapp.com/)
      - [cloudflare cors headers something](https://developers.cloudflare.com/workers/examples/cors-header-proxy)

- ui stuff
  - [color pallet gen](https://coolors.co/bac1b8-58a4b0-0c7c59-2b303a-d64933)

- custom uri scheme
  - create on for this app
    - [Create a custom URL Protocol Handler](https://unix.stackexchange.com/questions/497146/create-a-custom-url-protocol-handler)
    - [issue: Custom URI scheme support tauri](https://github.com/tauri-apps/tauri/issues/323)
    - [.desktop file template](https://github.com/tauri-apps/tauri/issues/5176)
    - [tauri deep link plugin](https://github.com/FabianLars/tauri-plugin-deep-link)
  - open other apps
    - [Anchor tag does not open link in default browser](https://github.com/tauri-apps/tauri/issues/4756)
      - no worky ```<a href="stremio:///detail/series/kitsu:43806/" target="_blank" rel="noreferrer" >stremio</a>```
      - [temp solution](https://github.com/tauri-apps/tauri/issues/4756#issuecomment-1200745849)

- search engine stuff:
  - tantivity
    - use in memory databases and retrieve all info from there and do not bother db
      - if in-memory stuff is not enough, can easily switch to temp dirs or something
    - use db only when storing/indexing
    - use tauri events or whatever to listen for commits when searching.
      - will be useful when indexing filesystem or sometihing
        - [maybe index fs like this??](https://github.com/quickwit-oss/tantivy/blob/6761237ec71b4e25ee4b5661e794b4755c6c5e56/examples/faceted_search.rs)
    - [warmer?](https://github.com/quickwit-oss/tantivy/blob/6761237ec71b4e25ee4b5661e794b4755c6c5e56/examples/warmer.rs)
    - fuzzy search
      - [It is reasonably simple however to split a query into several tokens, map these tokens into
        FuzzyTermQuery and combine those into a BooleanQuery.](https://github.com/quickwit-oss/tantivy/issues/947#issuecomment-734054824)
    - [pagination](https://docs.rs/tantivy/latest/tantivy/collector/struct.TopDocs.html#method.and_offset)
    - indexing multiple kinds of things using
      - [facet](https://docs.rs/tantivy/latest/tantivy/schema/struct.Facet.html)
      - [facet collector](https://docs.rs/tantivy/latest/tantivy/collector/struct.FacetCollector.html)

- ideas
  - do something like the ddg images. when a card is selected, display it in a new row and display
    a lof of the info that is available for it.
  - open stremio from app
    - [stremio src/deep_links/mod.rs](https://github.com/Stremio/stremio-core/blob/development/src/deep_links/mod.rs)
    - stremio:///detail/series/kitsu:43806
      - that number is the kitsu id
    - stremio:///detail/movie/tt1630029 stremio:///detail/series/tt13616990
      - that tt<number> is imdb id
    - [run stremio on browser](https://app.strem.io/shell-v4.4)
    - [run stremio on browser.dev](https://stremio-pwa.pages.dev/#/)
  - add lazy fetching to image loading from db. maybe have some item that is just an id+type that can be fetched and stored in
    the frontend all at once, and can be queried as needed lazily
    - tantivy already has DocAddress stuff. so should be pretty ez
  - maybe yeet sea_orm and just use sqlx
  - an ez way to handle passimg stuff to js could be to just store objects as Json or something after serialising (or as serde
    Value if supported by tantivy) and not even bothering to convert them back to rust types when sending data.
    and stuff can be indexed with whatever stuff required seperately.
