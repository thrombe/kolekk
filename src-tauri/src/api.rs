pub mod commands {
    use kolekk_types::api::tmdb::{ExternalIDs, ListResults, MultiSearchResult};
    use tauri::State;

    use crate::bad_error::Error;

    use super::tmdb::{Id, TmdbClient};

    #[tauri::command]
    pub async fn search_tmdb_multi(
        tmdb: State<'_, TmdbClient>,
        query: String,
        page: u32,
        include_adult: bool,
    ) -> Result<ListResults<MultiSearchResult>, Error> {
        tmdb.search_multi(query, page, include_adult).await
    }

    #[tauri::command]
    pub async fn tmdb_get_external_ids(
        tmdb: State<'_, TmdbClient>,
        id: Id,
    ) -> Result<ExternalIDs, Error> {
        tmdb.get_external_ids(id).await
    }
}

pub mod tmdb {
    #[allow(unused_imports)]
    use crate::{dbg, debug, error};

    use std::{fmt::Debug, sync::Arc};

    use kolekk_types::api::tmdb::{
        AltTitles, ExternalIDs, ExternalIdSearchResult, Images, ListResults, Movie,
        MovieListResult, MultiSearchResult, Tv, TvListResult,
    };
    use reqwest::Client;
    use serde::{de::DeserializeOwned, Deserialize, Serialize};

    use crate::bad_error::{BadError, Error, InferBadError, Inspectable};

    const BASE_URL: &str = "https://api.themoviedb.org/3/";
    const IMAGE_BASE_URL: &str = "https://image.tmdb.org/t/p/";

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct AllInfo<T> {
        // need to clone this type as flatten does not work good with TS macro
        #[serde(flatten)]
        pub t: T,
        pub alternative_titles: AltTitles,
        pub images: Images,
        pub external_ids: ExternalIDs,
    }

    impl<T> From<kolekk_types::api::tmdb::AllInfo<T>> for AllInfo<T> {
        fn from(value: kolekk_types::api::tmdb::AllInfo<T>) -> Self {
            Self {
                t: value.t,
                alternative_titles: value.alternative_titles,
                images: value.images,
                external_ids: value.external_ids,
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "media_type")]
    pub enum Id {
        #[serde(rename = "movie")]
        Movie { id: u32 },
        #[serde(rename = "tv")]
        Tv { id: u32 },
    }
    impl Id {
        pub fn id_type(&self) -> &'static str {
            match &self {
                Self::Movie { .. } => "movie",
                Self::Tv { .. } => "tv",
            }
        }

        pub fn id(&self) -> u32 {
            match &self {
                Self::Movie { id } => *id,
                Self::Tv { id } => *id,
            }
        }
    }
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum ExternalId {
        Tvdb(String),
        Imbd(String),
    }
    impl ExternalId {
        pub fn id_type(&self) -> &'static str {
            match &self {
                Self::Tvdb(_) => "tvdb_id",
                Self::Imbd(_) => "imdb_id",
            }
        }

        pub fn id(&self) -> &str {
            match &self {
                Self::Tvdb(id) => id,
                Self::Imbd(id) => id,
            }
        }
    }

    // https://developers.themoviedb.org/3/configuration/get-api-configuration
    #[derive(Debug, Clone, Deserialize)]
    pub struct Config {
        pub images: Option<ImageConfig>,
    }
    impl Config {
        pub fn get_secure_base_url(&self) -> &str {
            self.images
                .as_ref()
                .and_then(|e| e.secure_base_url.as_ref().map(|e| e.as_ref()))
                .unwrap_or(IMAGE_BASE_URL)
        }
    }
    impl Default for Config {
        fn default() -> Self {
            Config {
                images: Some(ImageConfig {
                    base_url: Some("http".to_string() + &IMAGE_BASE_URL[5..]),
                    secure_base_url: Some(IMAGE_BASE_URL.into()),
                }),
            }
        }
    }
    #[derive(Debug, Clone, Deserialize)]
    pub struct ImageConfig {
        pub base_url: Option<String>,
        pub secure_base_url: Option<String>,
    }
    #[derive(Debug, Clone)]
    pub struct TmdbClient {
        api_key: String,
        config: Config,
        client: Client,
    }

    impl TmdbClient {
        // reqwest::Client is clonable
        pub async fn new(api_key: impl Into<String>, client: Client) -> Result<Self, Error> {
            // https://api.themoviedb.org/3/configuration?api_key=<<api_key>>
            let api_key = api_key.into();
            let tmdb = Self {
                // config: client
                //     .get(format!("{}configuration?api_key={}", BASE_URL, &api_key))
                //     .send()
                //     .await
                //     .look(|e| dbg!(e))
                //     .infer_err()?
                //     .json()
                //     .await
                //     .look(|e| dbg!(e))
                //     .infer_err()?,
                config: Default::default(),
                api_key,
                client,
            };

            Ok(tmdb)
        }

        async fn get_parsed<T: DeserializeOwned + Debug>(
            &self,
            url: impl reqwest::IntoUrl,
        ) -> Result<T, Error> {
            let res = self
                .client
                .get(url)
                .send()
                .await
                .look(|e| dbg!(e))
                .infer_err()?
                // .json()
                .text()
                .await
                .look(|e| dbg!(e))
                .infer_err()?;
            let res = serde_json::from_str(&res).look(|e| dbg!(e)).infer_err()?;
            Ok(res)
        }

        pub async fn get_image_bytes(&self, path: impl AsRef<str>) -> Result<Vec<u8>, Error> {
            let img_path = path.as_ref();
            let bytes = self
                .client
                .get(format!(
                    "{}original{}",
                    self.config.get_secure_base_url(),
                    img_path
                ))
                .send()
                .await
                .look(|e| dbg!(e))
                .infer_err()?
                .bytes()
                .await
                .infer_err()?
                .into();
            Ok(bytes)
        }

        pub async fn get_movie_details(&self, id: u32) -> Result<Movie, Error> {
            // https://developers.themoviedb.org/3/movies/get-movie-details
            // https://api.themoviedb.org/3/movie/{movie_id}?api_key=<<api_key>>&language=en-US
            let res = self
                .get_parsed(format!(
                    "{}movie/{id}?api_key={}&language=en-US",
                    BASE_URL, &self.api_key,
                ))
                .await?;
            Ok(res)
        }
        pub async fn get_tv_details(&self, id: u32) -> Result<Tv, Error> {
            // https://developers.themoviedb.org/3/getting-started/append-to-response
            let res = self
                .get_parsed(format!(
                    "{}tv/{id}?api_key={}&language=en-US",
                    BASE_URL, &self.api_key,
                ))
                .await?;
            Ok(res)
        }

        pub async fn get_all_movie_details(&self, id: u32) -> Result<AllInfo<Movie>, Error> {
            // https://developers.themoviedb.org/3/getting-started/append-to-response
            let res = self
                .get_parsed(format!(
                    "{}movie/{id}?api_key={}&language=en-US&append_to_response=alternative_titles,images,external_ids",
                    BASE_URL, &self.api_key,
                ))
                .await?;
            Ok(res)
        }

        pub async fn get_all_tv_details(&self, id: u32) -> Result<AllInfo<Tv>, Error> {
            let res = self
                .get_parsed(format!(
                    "{}tv/{id}?api_key={}&language=en-US&append_to_response=alternative_titles,images,external_ids",
                    BASE_URL, &self.api_key,
                ))
                .await?;
            Ok(res)
        }

        pub async fn search_movies(
            &self,
            query: impl AsRef<str>,
            page: u32,
            include_adult: bool,
        ) -> Result<ListResults<MovieListResult>, Error> {
            // https://developers.themoviedb.org/3/search/search-movies
            // https://api.themoviedb.org/3/search/movie?api_key=<<api_key>>&language=en-US&page=<<number>>&include_adult=<<bool>>&query=<<query>>
            let res = self.get_parsed(format!(
                    "{}search/movie?api_key={}&language=en-US&page={page}&include_adult={include_adult}&query={}",
                    BASE_URL, &self.api_key, query.as_ref()
                )).await?;
            Ok(res)
        }
        pub async fn search_tv(
            &self,
            query: impl AsRef<str>,
            page: u32,
            include_adult: bool,
        ) -> Result<ListResults<TvListResult>, Error> {
            // https://developers.themoviedb.org/3/search/search-tv-shows
            // https://api.themoviedb.org/3/search/tv?api_key=<<api_key>>&language=en-US&page=<<number>>&include_adult=<<bool>>&query=<<query>>
            let res = self.get_parsed(format!(
                    "{}search/tv?api_key={}&language=en-US&page={page}&include_adult={include_adult}&query={}",
                    BASE_URL, &self.api_key, query.as_ref()
                )).await?;
            Ok(res)
        }

        // the total_results field includes "person" results too which are ignored in this function
        pub async fn search_multi(
            &self,
            query: impl AsRef<str>,
            page: u32,
            include_adult: bool,
        ) -> Result<ListResults<MultiSearchResult>, Error> {
            // https://developers.themoviedb.org/3/search/multi-search
            // https://api.themoviedb.org/3/search/multi?api_key=<<api_key>>&language=en-US&page=<<number>>&include_adult=<<bool>>&query=<<query>>
            let res: ListResults<serde_json::Value> = self.get_parsed(format!(
                    "{}search/multi?api_key={}&language=en-US&page={page}&include_adult={include_adult}&query={}",
                    BASE_URL, &self.api_key, query.as_ref()
                )).await?;
            let results = res
                .results
                .into_iter()
                .filter(|e| {
                    e.get("media_type").is_some() && e.get("media_type").unwrap() != "person"
                })
                .map(|e| serde_json::from_value(e).look(|e| dbg!(e)).infer_err())
                .collect::<Result<_, Error>>()?;
            Ok(ListResults {
                page: res.page,
                total_results: res.total_results,
                total_pages: res.total_pages,
                results,
            })
        }

        pub async fn get_alt_titles(&self, id: Id) -> Result<AltTitles, Error> {
            // https://developers.themoviedb.org/3/tv/get-movie-alternative-titles
            // https://api.themoviedb.org/3/tv/{tv_id}/alternative_titles?api_key=<<api_key>>&language=en-US
            // https://developers.themoviedb.org/3/movies/get-movie-alternative-titles
            // https://api.themoviedb.org/3/movie/{movie_id}/alternative_titles?api_key=<<api_key>>&language=en-US
            let res = self
                .get_parsed(format!(
                    "{}{}/{}/alternative_titles?api_key={}&language=en-US",
                    BASE_URL,
                    id.id_type(),
                    id.id(),
                    &self.api_key,
                ))
                .await?;
            Ok(res)
        }
        pub async fn get_external_ids(&self, id: Id) -> Result<ExternalIDs, Error> {
            // https://developers.themoviedb.org/3/tv/get-tv-external-ids
            // https://api.themoviedb.org/3/tv/{tv_id}/external_ids?api_key=<<api_key>>&language=en-US
            // https://developers.themoviedb.org/3/movies/get-movie-details
            // https://api.themoviedb.org/3/movie/{movie_id}/external_ids?api_key=<<api_key>>&language=en-US
            let res = self
                .get_parsed(format!(
                    "{}{}/{}/external_ids?api_key={}&language=en-US",
                    BASE_URL,
                    id.id_type(),
                    id.id(),
                    &self.api_key,
                ))
                .await?;
            Ok(res)
        }
        pub async fn get_images(&self, id: Id) -> Result<Images, Error> {
            // https://developers.themoviedb.org/3/movies/get-movie-images
            // https://api.themoviedb.org/3/movie/{movie_id}/images?api_key=<<api_key>>&language=en-US
            // https://developers.themoviedb.org/3/tv/get-tv-details
            // https://api.themoviedb.org/3/tv/{tv_id}/images?api_key=<<api_key>>&language=en-US
            let res = self
                .get_parsed(format!(
                    "{}{}/{}/images?api_key={}&language=en-US",
                    BASE_URL,
                    id.id_type(),
                    id.id(),
                    &self.api_key,
                ))
                .await?;
            Ok(res)
        }

        pub async fn find(&self, id: ExternalId) -> Result<ExternalIdSearchResult, Error> {
            // https://developers.themoviedb.org/3/find/find-by-id
            // https://api.themoviedb.org/3/find/{external_id}?api_key=<<api_key>>&language=en-US&external_source=<<id type>>
            let res = self
                .get_parsed(format!(
                    "{}find/{}?api_key={}&language=en-US&external_source={}",
                    BASE_URL,
                    id.id(),
                    &self.api_key,
                    id.id_type()
                ))
                .await?;
            Ok(res)
        }
    }
}
