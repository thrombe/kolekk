use serde::{Deserialize, Serialize};
use std::fmt::Debug;
pub use ts_rs::TS;

pub mod api {
    pub mod tmdb {
        use serde::{Deserialize, Serialize};
        use ts_rs::TS;

        // https://developers.themoviedb.org/3/tv/get-tv-images
        // https://developers.themoviedb.org/3/movies/get-movie-images
        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        pub struct Images {
            pub backdrops: Vec<ImageInfo>,
            pub posters: Vec<ImageInfo>,
        }
        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        pub struct ImageInfo {
            pub aspect_ratio: Option<f32>, // sort by this
            pub vote_average: Option<f32>,
            pub height: Option<u32>,
            pub width: Option<u32>,
            pub file_path: Option<String>,
        }

        // https://developers.themoviedb.org/3/tv/get-tv-details
        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        pub struct Tv {
            pub backdrop_path: Option<String>,
            pub poster_path: Option<String>,
            pub status: Option<String>, // ongoing/ended/whatever
            pub genres: Vec<Genre>,
            pub id: Option<u32>,
            pub name: Option<String>,
            pub seasons: Vec<Season>,
            pub original_name: Option<String>,
            pub overview: Option<String>, // description of show

            pub number_of_seasons: Option<u32>,
            pub number_of_episodes: Option<u32>,

            pub popularity: Option<f32>,
            pub vote_average: Option<f32>,
            pub vote_count: Option<u32>,
        }
        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        pub struct Genre {
            pub id: Option<u32>,
            pub name: Option<String>,
        }
        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        pub struct Season {
            pub episode_count: Option<u32>,
            pub id: Option<u32>,
            pub name: Option<String>,
            pub poster_path: Option<String>,
            pub season_number: Option<u32>,
        }

        // https://developers.themoviedb.org/3/movies/get-movie-details
        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        pub struct Movie {
            pub adult: Option<bool>,
            pub backdrop_path: Option<String>,
            pub poster_path: Option<String>,
            pub status: Option<String>, // ongoing/ended/whatever
            pub genres: Vec<Genre>,
            pub id: Option<u32>,
            pub imdb_id: Option<String>,
            pub title: Option<String>,
            pub original_title: Option<String>,
            pub overview: Option<String>,
            pub popularity: Option<f32>,
            pub vote_average: Option<f32>,
            pub vote_count: Option<u32>,
            pub runtime: Option<u32>,
        }

        // https://developers.themoviedb.org/3/tv/get-tv-alternative-titles
        // https://developers.themoviedb.org/3/movies/get-movie-alternative-titles
        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        pub struct AltTitles {
            pub id: Option<u32>,
            pub results: Vec<Title>,
        }
        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        pub struct Title {
            pub title: Option<String>,
            pub iso_3166_1: Option<String>,
        }

        // https://developers.themoviedb.org/3/tv/get-tv-external-ids
        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        pub struct ExternalIDs {
            pub id: Option<u32>,
            pub imdb_id: Option<String>,
            pub tvdb_id: Option<u32>,
            pub wikidata_id: Option<String>,
        }

        // https://developers.themoviedb.org/3/find/find-by-id
        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        pub struct ExternalIdSearchResult {
            pub movie_results: Vec<MovieListResult>,
            pub tv_results: Vec<TvListResult>,
        }

        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        pub struct ListResults<T> {
            pub page: Option<u32>,
            pub total_results: Option<u32>,
            pub total_pages: Option<u32>,
            pub results: Vec<T>,
        }

        // https://developers.themoviedb.org/3/tv/get-popular-tv-shows
        // https://developers.themoviedb.org/3/tv/get-top-rated-tv
        // https://developers.themoviedb.org/3/search/search-tv-shows
        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        pub struct TvListResult {
            pub backdrop_path: Option<String>,
            pub poster_path: Option<String>,
            pub genre_ids: Vec<u32>,
            pub id: Option<u32>,
            pub name: Option<String>,
            pub original_name: Option<String>,
            pub overview: Option<String>,
            pub popularity: Option<f32>,
            pub vote_average: Option<f32>,
            pub vote_count: Option<u32>,
        }

        // https://developers.themoviedb.org/3/search/search-movies
        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        pub struct MovieListResult {
            pub id: Option<u32>,
            pub adult: Option<bool>,
            pub backdrop_path: Option<String>,
            pub poster_path: Option<String>,
            pub overview: Option<String>,
            pub popularity: Option<f32>,
            pub vote_average: Option<f32>,
            pub vote_count: Option<u32>,
            pub genre_ids: Vec<u32>,
            pub title: Option<String>,
            pub original_title: Option<String>,
        }

        // https://developers.themoviedb.org/3/search/multi-search
        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        #[serde(tag = "media_type")]
        pub enum MultiSearchResult {
            #[serde(rename = "movie")]
            Movie {
                #[serde(flatten)]
                result: MovieListResult,
            },
            #[serde(rename = "tv")]
            Tv {
                #[serde(flatten)]
                result: TvListResult,
            },
        }

        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        pub struct AllInfo<T> {
            // #[serde(flatten)]
            pub t: T,
            pub alternative_titles: AltTitles,
            pub images: Images,
            pub external_ids: ExternalIDs,
        }
    }

    pub mod tachidesk {
        use std::collections::HashMap;

        use serde::{Deserialize, Serialize};
        use ts_rs::TS;

        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        pub enum ExtensionAction {
            #[serde(rename = "install")]
            Install,
            #[serde(rename = "update")]
            Update,
            #[serde(rename = "uninstall")]
            Uninstall,
        }
        impl AsRef<str> for ExtensionAction {
            fn as_ref(&self) -> &str {
                match self {
                    Self::Install => "install",
                    Self::Update => "update",
                    Self::Uninstall => "uninstall",
                }
            }
        }

        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        #[serde(rename_all = "camelCase")]
        pub struct About {
            build_time: u64,
            build_type: String,
            github: String,
            name: String,
            version: String,
        }

        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        #[serde(rename_all = "camelCase")]
        pub struct Extension {
            pub name: String,
            pub pkg_name: String,
            pub version_name: String,
            pub version_code: u64,
            pub lang: String,
            pub is_nsfw: bool,
            pub apk_name: String,
            pub icon_url: String,
            pub installed: bool,
            pub has_update: bool,
            pub obsolete: bool,
        }

        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        #[serde(rename_all = "camelCase")]
        pub struct MangaSource {
            pub id: String,
            pub name: String,
            pub lang: String,
            pub icon_url: String,
            pub supports_latest: bool,
            pub is_configurable: bool,
            pub is_nsfw: bool,
            pub display_name: String,
        }

        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        #[serde(rename_all = "camelCase")]
        pub struct Chapter {
            pub id: u64,
            pub url: String,
            pub name: String,
            pub upload_date: u64,
            pub chapter_number: f64,
            pub scanlator: Option<String>,
            pub manga_id: u64,
            pub read: bool,
            pub bookmarked: bool,
            pub last_page_read: u64,
            pub last_read_at: u64,
            pub index: u64,
            pub fetched_at: u64,
            pub chapter_count: u64,
            #[serde(deserialize_with = "deser_page_count")]
            pub page_count: Option<u64>,
            pub downloaded: bool,
            pub meta: HashMap<String, MetaValue>,
        }
        fn deser_page_count<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let value = i64::deserialize(deserializer)?;
            let v = (value > 0).then_some(value as _);
            Ok(v)
        }

        // tachiyomi => domain/src/main/java/tachiyomi/domain/manga/model/Manga.kt
        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        #[serde(rename_all = "camelCase")]
        pub struct Manga {
            pub id: u64,
            pub source_id: String,

            pub url: String,
            pub title: String,
            pub thumbnail_url: String,

            pub artist: Option<String>,
            pub author: Option<String>,
            pub description: Option<String>,
            pub genre: Vec<String>,
            pub status: String,

            pub in_library: bool,
            pub source: Option<MangaSource>,

            pub meta: HashMap<String, MetaValue>,

            pub real_url: Option<String>,
            pub fresh_data: bool,
            pub unread_count: Option<u64>,
            pub download_count: Option<u64>,

            pub age: u64,
            pub chapters_age: u64,
            pub chapter_count: Option<u64>,
            pub chapters_last_fetched_at: u64,
            pub in_library_at: u64,
            pub initialized: bool,
            pub last_chapter_read: Option<Chapter>,
            pub last_fetched_at: u64,
            pub last_read_at: Option<u64>,
            pub thumbnail_url_last_fetched: u64,
            pub update_strategy: String,
        }

        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        #[serde(rename_all = "camelCase")]
        pub struct MangaListPage {
            pub has_next_page: bool,
            pub manga_list: Vec<Manga>,
        }

        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        #[serde(untagged)]
        pub enum MetaValue {
            String(String),
            Bool(bool),
            U64(u64),
            None,
        }

        // tachiyomi => source-api/src/commonMain/kotlin/eu/kanade/tachiyomi/source/model/Filter.kt
        // https://serde.rs/enum-representations.html#adjacently-tagged
        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        #[serde(tag = "type", content = "filter")]
        pub enum SourceFilter {
            CheckBox {
                name: String,
                state: bool,
                // iso_code: Option<String>, // language has this one too in mangadex
                // value: Option<String>, // content rating has this one too in mangadex
            },
            Group {
                name: String,
                state: Vec<Self>,
            },
            Sort {
                name: String,
                state: SortFilter,
                #[serde(rename = "values")]
                sort_categories: Vec<String>,
            },
            Select {
                name: String,
                #[serde(rename = "state")]
                selected_index: usize,
                #[serde(rename = "displayValues")]
                display_values: Vec<String>,
                values: Vec<SelectableItem>,
            },
            TriState {
                // id: Option<String>, // is in mangadex
                name: String,
                state: usize,
                excluded: bool,
                ignored: bool,
                included: bool,
            },
            Text {
                name: String,
                #[serde(rename = "state")]
                text_value: String,
            },
            Header {
                name: String,
                state: usize,
            },
            Separator {
                name: String,
                state: usize,
            },
        }

        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        pub struct SortFilter {
            #[serde(rename = "index")]
            selected_index: usize,
            ascending: bool,
        }

        #[derive(Serialize, Deserialize, TS, Debug, Clone)]
        #[serde(untagged)]
        pub enum SelectableItem {
            Type1 { title: String, value: String },
            Type2(String),
            Type3(HashMap<String, String>),
        }
    }
}

pub mod objects {
    use std::{ops::Deref, borrow::Cow};

    use serde::{Deserialize, Serialize};
    use ts_rs::TS;

    use crate::{
        api,
        utility::{Path, Source},
    };

    pub type Id = u32;

    // #[derive(Serialize, Deserialize, TS, Debug, Clone)]
    // #[serde(untagged)]
    // #[allow(clippy::large_enum_variant)]
    // pub enum Object<T = ()> {
    //     Image(Image),
    //     Bookmark(Bookmark),
    //     Group(Group),
    //     Tag(Tag),
    //     Content(Content),
    //     Unknown(T),
    // }

    #[derive(Serialize, Deserialize, TS, Debug, Clone)]
    #[serde(tag = "object_type")]
    pub struct Image {
        pub title: Option<String>,
        pub src: Option<Source>,
        pub path: Path,
        pub chksum: Vec<u8>,
        pub size: usize,
    }

    #[derive(Serialize, Deserialize, TS, Debug, Clone)]
    #[serde(tag = "object_type")]
    pub struct Bookmark {
        pub title: Option<String>,
        pub url: String,
        pub description: Option<String>,
    }

    #[derive(Serialize, Deserialize, TS, Debug, Clone)]
    #[serde(tag = "object_type")]
    pub struct Group {
        pub main: Option<Id>,
        pub items: Vec<Id>,
    }

    #[derive(Serialize, Deserialize, TS, Debug, Clone)]
    #[serde(tag = "content_type", content = "content")]
    #[allow(clippy::large_enum_variant)]
    pub enum Content {
        TmdbTv(api::tmdb::AllInfo<api::tmdb::Movie>),
        TmdbMovie(api::tmdb::AllInfo<api::tmdb::Tv>),
        TachiManga(api::tachidesk::Manga),
    }

    #[derive(Serialize, Deserialize, TS, Debug, Clone)]
    #[serde(tag = "object_type")]
    pub enum Tag {
        #[serde(rename = "main_tag")]
        Main { name: String },
        #[serde(rename = "alias_tag")]
        Alias { name: String, alias_to: Id },
    }

    // add a notes object and link it to other objects to give some more context to them
    // the linking can be done using the Group ojject
    // like: this show was recommended by this person
    //   this image is related to this meme
    //   i read this link on this date
    #[derive(Serialize, Deserialize, TS, Debug, Clone)]
    #[serde(tag = "object_type")]
    pub struct Notes {
        pub data: String,
    }

    // to insert a random json object in database to make it searchable
    #[derive(Serialize, Deserialize, TS, Debug, Clone)]
    pub struct SearchableEntry<T> {
        pub data: T,
        pub searchable: Vec<Indexed>,
    }

    #[derive(Serialize, Deserialize, TS, Debug, Clone)]
    pub struct Indexed {
        pub field: Fields,
        // - [TS in ts_rs](https://docs.rs/ts-rs/latest/ts_rs/trait.TS.html#container-attributes)
        #[ts(type = "any")]
        pub data: serde_json::Value,
    }

    #[derive(Serialize, Deserialize, TS, Debug, Clone)]
    pub struct Taggable<T> {
        pub data: T,
        pub tags: Vec<Id>,
    }

    #[derive(Serialize, Deserialize, TS, Debug, Clone)]
    pub struct Meta<T> {
        pub id: Id,
        pub data: T,
        pub ctime: u64,
        pub last_update: u64,
        pub last_interaction: u64,
    }

    #[derive(
        Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash,
    )]
    pub enum Fields {
        Id,    // unique id
        Type,  // facet
        Text,  // any indexed text
        Ctime, // sort by this if same score
        Mtime, // sort by this if same score
        LastInteraction,
        Chksum, // to check if file or some data is already in db or no
        Tag,
        Json,
    }
    impl Deref for Fields {
        type Target = str;

        fn deref(&self) -> &Self::Target {
            match self {
                Self::Id => "id",
                Self::Type => "type",
                Self::Text => "text",
                Self::Ctime => "ctime",
                Self::Mtime => "mtime",
                Self::LastInteraction => "last_interaction",
                Self::Chksum => "chksum",
                Self::Tag => "tag",
                Self::Json => "json",
            }
        }
    }

    impl AsRef<str> for Fields {
        fn as_ref(&self) -> &str {
            self.deref()
        }
    }

    #[derive(Serialize, Deserialize, TS, Debug, Clone)]
    pub enum TypeFacet {
        Image,
        Bookmark,
        Tag,
        Group,
        Content,
        Notes,
        Temp(#[ts(type = "string")] Cow<'static, str>),
    }

    impl AsRef<str> for TypeFacet {
        fn as_ref(&self) -> &str {
            match self {
                Self::Image => "/image",
                Self::Bookmark => "/bookmark",
                Self::Tag => "/tag",
                Self::Group => "/group",
                Self::Content => "/content",
                Self::Notes => "/notes",
                Self::Temp(s) => s,
            }
        }
    }
}

pub mod utility {
    use derivative::Derivative;
    use serde::{Deserialize, Serialize};
    use ts_rs::TS;

    use std::{fmt::Debug, path::PathBuf};

    #[derive(Serialize, Deserialize, TS, Debug, Clone)]
    pub struct Path {
        pub base: BasePath,
        #[ts(type = "string")]
        pub path: PathBuf,
    }
    impl Path {
        pub fn join(&self, p: impl AsRef<std::path::Path>) -> Self {
            Self {
                base: self.base,
                path: self.path.join(p.as_ref()),
            }
        }
    }

    #[derive(Serialize, Deserialize, TS, Debug, Clone, Copy)]
    pub enum BasePath {
        AppDataDir,
        AppCacheDir,
        AbsolutePath,
    }

    #[derive(Serialize, Deserialize, TS, Debug, Clone)]
    #[serde(tag = "type", content = "src")]
    pub enum Source {
        Path(Path),
        Url(String),
    }

    #[derive(
        Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash,
    )]
    #[serde(rename_all = "camelCase")]
    pub enum ThumbnailSize {
        Original = 9,
        W50 = 0,
        W100 = 1,
        W150 = 2,
        W200 = 3,
        W350 = 4,
        W500 = 5,
        W750 = 6,
        W1000 = 7,
        W1920 = 8,
    }
    impl AsRef<str> for ThumbnailSize {
        fn as_ref(&self) -> &str {
            match self {
                Self::Original => "original",
                Self::W50 => "w50",
                Self::W100 => "w100",
                Self::W150 => "w150",
                Self::W200 => "w200",
                Self::W350 => "w350",
                Self::W500 => "w500",
                Self::W750 => "w750",
                Self::W1000 => "w1000",
                Self::W1920 => "w1920",
            }
        }
    }
    impl ThumbnailSize {
        pub fn value(&self) -> Option<u32> {
            match self {
                Self::Original => None,
                Self::W50 => Some(50),
                Self::W100 => Some(100),
                Self::W150 => Some(150),
                Self::W200 => Some(200),
                Self::W350 => Some(350),
                Self::W500 => Some(500),
                Self::W750 => Some(750),
                Self::W1000 => Some(1000),
                Self::W1920 => Some(1920),
            }
        }

        pub fn get_appropriate_size(u: u32) -> Self {
            match u {
                0..=75 => Self::W50,
                76..=125 => Self::W100,
                126..=250 => Self::W200,
                251..=400 => Self::W350,
                401..=600 => Self::W500,
                601..=800 => Self::W750,
                801..=1400 => Self::W1000,
                1401..=2100 => Self::W1920,
                2101.. => Self::Original,
            }
        }
    }

    #[derive(Serialize, Deserialize, TS, Derivative, Clone)]
    #[derivative(Debug)]
    pub struct ByteArrayFile {
        pub name: String,
        #[derivative(Debug = "ignore")]
        pub data: Vec<u8>,
    }

    #[derive(Serialize, Deserialize, TS, Debug, Clone)]
    pub struct DragDropPaste<F: Debug> {
        // priority in the same order
        pub file_uris: Option<Vec<String>>, // "http://" "ftp://" "smb://" "/home/"
        pub text: Option<String>,           // anything. links, just text, whatever
        pub text_html: Option<String>,      // <img href=""> <span>
        pub files: Option<Vec<F>>,          // File data

        pub uri_list: Option<String>, // link drops. (link is also available in self.text)
    }

    #[derive(Serialize, Deserialize, TS, Debug, PartialEq, Eq, Clone)]
    pub struct FileMetadata {
        pub chksum: [u8; 16],
        pub size: u64,
    }
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
pub enum FilderKind {
    File,
    Folder,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
pub struct Filder {
    pub name: String,
    pub files: Option<Vec<Filder>>,
    pub kind: FilderKind,
}
