use derivative::Derivative;
use sea_orm::entity::prelude::*;
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

        use serde::{Serialize, Deserialize};
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
            let v = (value>0).then_some(value as _);
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
            Type1 {
                title: String,
                value: String,
            },
            Type2(String),
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

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[serde(tag = "object_type")]
// #[serde(rename = "image")]
pub struct Image {
    pub id: u32,
    pub title: String,
    pub src_path: String,
    pub db_path: String,
    pub chksum: Vec<u8>,
    pub size: usize,
    pub urls: Vec<String>,
    pub tags: Vec<u32>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[serde(tag = "object_type")]
pub struct Bookmark {
    pub id: u32,
    pub title: Option<String>,
    pub url: String,
    pub tags: Vec<u32>,
    pub description: Option<String>,
    pub related: Vec<u32>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[serde(tag = "object_type")]
pub struct Group {
    pub id: u32,
    pub main: Option<u32>,
    pub items: Vec<u32>,
    pub tags: Vec<u32>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum Object {
    Image(Image),
    Bookmark(Bookmark),
    Group(Group),
    Tag(Tag),
    Content(Content),
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[serde(tag = "content_type")]
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
    Main {
        id: u32,
        name: String,
    },
    #[serde(rename = "alias_tag")]
    Alias {
        id: u32,
        name: String,
        alias_to: u32,
    },
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
pub struct SearchableEntry {
    pub obj: Json,
    pub search_context: Vec<String>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[serde(tag = "content_type")]
pub struct JsonObject<T> {
    pub id: u32,
    pub obj: T,
    pub tags: Vec<u32>,
}

#[serde(rename_all = "camelCase")]
pub enum ThumbnailSize {
    Original,
    W50,
    W100,
    W150,
    W200,
    W350,
    W500,
    W750,
    W1000,
    W1920,
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

#[derive(Serialize, Deserialize, TS, Debug, PartialEq, Eq, Clone)]
pub struct FileMetadata {
    pub chksum: [u8; 16],
    pub size: u64,
}

// impl Image {
//     pub async fn all_from_db(db: &DatabaseConnection) -> Vec<Self> {
//         let img = images::Entity::find().all(db).await.unwrap();
//         let mut images = vec![];
//         for e in img.into_iter() {
//             let e = Self {
//                 id: e.id,
//                 title: e.title,
//                 src_path: e.src_path,
//                 db_path: e.db_path,
//                 chksum: e.chksum,
//                 size: e.size as _,
//                 urls: urls::Entity::find_by_id(e.id)
//                     .all(db)
//                     .await
//                     .unwrap()
//                     .into_iter()
//                     .map(|e| e.url)
//                     .collect(),
//                 tags: tags::Entity::find()
//                     .filter(tags::Column::Id.eq(e.id))
//                     .all(db)
//                     .await
//                     .unwrap()
//                     .into_iter()
//                     .map(|e| e.tag)
//                     .collect(),
//             };
//             images.push(e);
//         }
//         images
//     }

//     pub async fn add_tag(&mut self, db: &DatabaseConnection, tag: String) {
//         if tags::Entity::find()
//             .filter(tags::Column::Id.eq(self.id))
//             .filter(tags::Column::Tag.eq(tag.clone()))
//             .one(db)
//             .await
//             .unwrap()
//             .is_some()
//         {
//             return;
//         }

//         let tag = tags::ActiveModel {
//             id: sea_orm::Set(self.id),
//             tag: sea_orm::Set(tag),
//         };
//         let tag = tag.insert(db).await.unwrap();
//         self.tags.push(tag.tag);
//     }

//     pub async fn remove_tag(&mut self, db: &DatabaseConnection, tag: String) {
//         self.tags
//             .remove(self.tags.iter().position(|e| *e == tag).unwrap());
//         let tag = tags::ActiveModel {
//             id: sea_orm::Set(self.id),
//             tag: sea_orm::Set(tag),
//         };
//         let _ = tags::Entity::delete(tag).exec(db).await.unwrap();
//     }
// }

// pub mod bookmarks {
//     use super::*;

//     #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
//     #[sea_orm(table_name = "bookmarks")]
//     pub struct Model {
//         #[sea_orm(primary_key)]
//         pub id: u32,
//         pub title: String,
//         pub url: String,
//     }

//     #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
//     pub enum Relation {}

//     impl ActiveModelBehavior for ActiveModel {}
// }

// pub mod images {
//     use super::*;

//     #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
//     #[sea_orm(table_name = "images")]
//     pub struct Model {
//         #[sea_orm(primary_key)]
//         pub id: u32,
//         // #[sea_orm(primary_key)]
//         // #[sea_orm(unique)]
//         pub chksum: Vec<u8>,
//         pub size: u32,
//         pub title: String,
//         pub src_path: String,
//         pub db_path: String,
//     }

//     #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
//     pub enum Relation {}

//     impl ActiveModelBehavior for ActiveModel {}
// }

// pub mod tags {
//     use super::*;

//     #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
//     #[sea_orm(table_name = "tags")]
//     pub struct Model {
//         #[sea_orm(primary_key)]
//         pub id: u32,
//         #[sea_orm(primary_key)]
//         pub tag: String,
//     }

//     #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
//     pub enum Relation {}

//     impl ActiveModelBehavior for ActiveModel {}
// }

// pub mod urls {
//     use super::*;

//     #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
//     #[sea_orm(table_name = "urls")]
//     pub struct Model {
//         #[sea_orm(primary_key)]
//         pub id: u32,
//         #[sea_orm(unique)]
//         pub url: String,
//     }

//     #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
//     pub enum Relation {}

//     impl ActiveModelBehavior for ActiveModel {}
// }

// pub mod metadata {
//     use super::*;

//     #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
//     #[sea_orm(table_name = "metadata")]
//     pub struct Model {
//         #[sea_orm(primary_key)]
//         pub id: u32,
//         // pub added_ts: datetime?,
//         // pub last_edit: ts?,
//     }

//     #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
//     pub enum Relation {}

//     impl ActiveModelBehavior for ActiveModel {}
// }
