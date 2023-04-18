pub mod commands {
    use std::time::Duration;

    use kolekk_types::api::{
        tachidesk::{
            Chapter, Extension, ExtensionAction, Manga, MangaListPage, MangaSource, SourceFilter,
        },
        tmdb::{ExternalIDs, ListResults, MultiSearchResult},
    };
    use reqwest::Client;
    use tauri::{Manager, State, WindowEvent};

    use crate::{
        bad_error::{BadError, Error, InferBadError},
        config::AppConfig,
    };

    use super::{
        tachidesk::TachideskClient,
        tmdb::{Id, TmdbClient},
    };

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

    #[tauri::command]
    pub async fn init_tachidesk_client(
        app_handle: tauri::State<'_, tauri::AppHandle>,
        client: tauri::State<'_, Client>,
        conf: tauri::State<'_, AppConfig>,
    ) -> Result<bool, Error> {
        if app_handle.try_state::<TachideskClient>().is_none() {
            let tachi = TachideskClient::download_if_needed(
                client.inner().clone(),
                conf.app_data_dir.join("tachidesk"),
            )
            .await?;

            let now = std::time::SystemTime::now();
            while tachi.get_server_info().await.ok().is_none() {
                tokio::time::sleep(Duration::from_secs_f32(0.5)).await;
                if now.elapsed().infer_err()?.as_secs_f64() > 10.0 {
                    tachi.child.lock().infer_err()?.start_kill().infer_err()?;
                    return None.bad_err("server timeout :(");
                }
            }

            // TODO: if nothing works, try spawnning some async task that just listens for async channel and kills tachidesk when it receives from it
            let handle = app_handle.app_handle();
            app_handle.manage(tachi);
            // TODO: ugly unwraps :(
            app_handle
                // .get_window("kolekk")
                .windows()
                .into_values()
                .next()
                .expect("no window?")
                .on_window_event(move |e| match e {
                    WindowEvent::Destroyed | WindowEvent::CloseRequested { .. } => {
                        handle
                            .state::<TachideskClient>()
                            .inner()
                            .child
                            .lock()
                            .infer_err()
                            .unwrap()
                            .start_kill()
                            .unwrap();
                    }
                    _ => {}
                });
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[tauri::command]
    pub async fn tachidesk_get_all_extensions(
        tachi: tauri::State<'_, TachideskClient>,
    ) -> Result<Vec<Extension>, Error> {
        tachi.get_all_extensions().await
    }

    #[tauri::command]
    pub async fn tachidesk_extension_action(
        tachi: tauri::State<'_, TachideskClient>,
        pkg_name: String,
        action: ExtensionAction,
    ) -> Result<(), Error> {
        tachi.extension_action(pkg_name, action).await
    }

    #[tauri::command]
    pub fn tachidesk_get_extension_icon_url(
        tachi: tauri::State<'_, TachideskClient>,
        icon_url: String,
    ) -> Result<String, Error> {
        let u = tachi.get_extension_icon_url(icon_url);
        Ok(u)
    }

    #[tauri::command]
    pub async fn tachidesk_get_manga_chapter_list(
        tachi: tauri::State<'_, TachideskClient>,
        manga_id: u64,
    ) -> Result<Vec<Chapter>, Error> {
        tachi.get_manga_chapter_list(manga_id).await
    }

    #[tauri::command]
    pub fn tachidesk_get_manga_page_url(
        tachi: tauri::State<'_, TachideskClient>,
        manga_id: u64,
        chapter_index: u64,
        page: u64,
    ) -> Result<String, Error> {
        let u = tachi.get_manga_page_url(manga_id, chapter_index, page);
        Ok(u)
    }

    #[tauri::command]
    pub fn tachidesk_get_manga_thumbnail_url(
        tachi: tauri::State<'_, TachideskClient>,
        manga_id: u64,
    ) -> Result<String, Error> {
        let u = tachi.get_manga_thumbnail_url(manga_id);
        Ok(u)
    }

    #[tauri::command]
    pub async fn tachidesk_get_manga(
        tachi: tauri::State<'_, TachideskClient>,
        manga_id: u64,
    ) -> Result<Manga, Error> {
        tachi.get_manga(manga_id).await
    }

    #[tauri::command]
    pub async fn tachidesk_get_chapter(
        tachi: tauri::State<'_, TachideskClient>,
        manga_id: u64,
        chapter_index: u64,
    ) -> Result<Chapter, Error> {
        tachi.get_chapter(manga_id, chapter_index).await
    }

    #[tauri::command]
    pub async fn tachidesk_get_source_list(
        tachi: tauri::State<'_, TachideskClient>,
    ) -> Result<Vec<MangaSource>, Error> {
        tachi.get_source_list().await
    }

    #[tauri::command]
    pub async fn tachidesk_get_source_filters(
        tachi: tauri::State<'_, TachideskClient>,
        source_id: String,
    ) -> Result<Vec<SourceFilter>, Error> {
        tachi.get_source_filters(source_id).await
    }

    #[tauri::command]
    pub async fn tachidesk_get_latest_manga_list(
        tachi: tauri::State<'_, TachideskClient>,
        source_id: String,
        page: u64,
    ) -> Result<MangaListPage, Error> {
        tachi.get_latest_manga_list(source_id, page).await
    }

    #[tauri::command]
    pub async fn tachidesk_get_popular_manga_list(
        tachi: tauri::State<'_, TachideskClient>,
        source_id: String,
        page: u64,
    ) -> Result<MangaListPage, Error> {
        tachi.get_popular_manga_list(source_id, page).await
    }

    #[tauri::command]
    pub async fn tachidesk_search_manga_in(
        tachi: tauri::State<'_, TachideskClient>,
        source_id: String,
        query: String,
        page: u64,
    ) -> Result<MangaListPage, Error> {
        tachi.search_manga_in(source_id, query, page).await
    }
}

mod common {
    #[allow(unused_imports)]
    use crate::{dbg, debug, error};

    use std::fmt::Debug;

    use reqwest::Client;
    use serde::de::DeserializeOwned;

    use crate::bad_error::{Error, InferBadError, Inspectable};

    pub async fn get_parsed<T: DeserializeOwned + Debug>(
        client: &Client,
        url: impl reqwest::IntoUrl,
    ) -> Result<T, Error> {
        let res = client
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
        let res = serde_json::from_str(&res).infer_err()?;
        Ok(res)
    }
}

pub mod tmdb {
    #[allow(unused_imports)]
    use crate::{dbg, debug, error};

    use std::fmt::Debug;

    use kolekk_types::api::tmdb::{
        AltTitles, ExternalIDs, ExternalIdSearchResult, Images, ListResults, Movie,
        MovieListResult, MultiSearchResult, Tv, TvListResult,
    };
    use reqwest::Client;
    use serde::{de::DeserializeOwned, Deserialize, Serialize};

    use crate::bad_error::{Error, InferBadError, Inspectable};

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
            let res = super::common::get_parsed(&self.client, url).await?;
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
                    e.get("media_type").is_some()
                        && (e.get("media_type").unwrap() == "tv"
                            || e.get("media_type").unwrap() == "movie")
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

pub mod tachidesk {
    #[allow(unused_imports)]
    use crate::{dbg, debug, error};

    use std::{
        fmt::Debug,
        fs::File,
        io::{BufReader, Cursor, Write},
        path::{Path, PathBuf},
        str::FromStr,
        sync::Mutex,
    };

    use flate2::bufread::MultiGzDecoder;
    use kolekk_types::api::tachidesk::{
        About, Chapter, Extension, ExtensionAction, Manga, MangaListPage, MangaSource, SourceFilter,
    };
    use reqwest::{Client, Url};
    use serde::{de::DeserializeOwned, Deserialize};
    use tar::Archive;
    use tokio::process::{Child, Command};

    use crate::bad_error::{BadError, Error, InferBadError, Inspectable};

    const BASE_URL: &str = "http://0.0.0.0:4567";

    // /api/v1/source/{source id}/preferences
    // /api/v1/source/{source id}/filters
    // /api/v1/source/{source id}/filters?reset=true
    // /api/v1/category ???
    // /api/v1/category/reorder ???
    // /api/v1/backup/import/file
    // /api/v1/backup/export/file
    // /api/v1/update/recentChapters/{page num or something}
    // api/v1/meta ??
    // /api/v1/manga/{mangaid}/library ? add to library or somethin
    // /api/v1/manga/{mangaid}/category ??
    #[derive(Debug)]
    pub struct TachideskClient {
        pub child: Mutex<Child>,
        client: Client,
        pub jre: PathBuf,
        pub tachidesk_path: PathBuf,
        pub root_dir: PathBuf,
    }

    #[derive(Debug, Clone, Deserialize)]
    struct GithubRelease {
        url: String,
        name: String,
        tag_name: String,
        created_at: String,
        published_at: String,
        draft: bool,
        prerelease: bool,
        assets: Vec<GithubReleaseAsset>,
    }
    #[derive(Debug, Clone, Deserialize)]
    struct GithubReleaseAsset {
        url: String,
        name: String,
        size: u64,
        browser_download_url: String,
    }

    impl TachideskClient {
        pub fn new(
            client: Client,
            jre: impl AsRef<Path>,
            tachidesk_jar: impl AsRef<Path>,
            tachidesk_root_dir: impl AsRef<Path>,
        ) -> Result<Self, Error> {
            let jre = jre.as_ref();
            let tachidesk_jar = tachidesk_jar.as_ref();
            let root_dir = tachidesk_root_dir.as_ref();
            let cache_dir = root_dir.join("cache");
            let thumbnails_dir = root_dir.join("thumbnails");

            let _ = std::fs::remove_dir_all(&thumbnails_dir).look(|e| dbg!(e));
            let _ = std::fs::remove_dir_all(&cache_dir).look(|e| dbg!(e));
            // let _ = std::fs::remove_file(&root_dir.join("database.mv.db")).look(|e| dbg!(e));

            jre.exists()
                .then_some(())
                .bad_err("jre path does not exist")?;
            tachidesk_jar
                .exists()
                .then_some(())
                .bad_err("tachidesk jar does not exist")?;

            let mut tachi = Command::new(jre.join("bin/java"));
            tachi
                .kill_on_drop(true)
                .arg(format!(
                    "-Dsuwayomi.tachidesk.config.server.rootDir={}",
                    root_dir.to_string_lossy()
                ))
                .arg("-Dsuwayomi.tachidesk.config.server.webUIEnabled=false")
                .arg("-Dsuwayomi.tachidesk.config.server.systemTrayEnabled=false")
                .arg("-Dsuwayomi.tachidesk.config.server.debugLogsEnabled=false")
                .arg(format!("-Djava.io.tmpdir={}", cache_dir.to_string_lossy()))
                .arg("-jar")
                .arg(tachidesk_jar);

            let client = Self {
                client,
                child: Mutex::new(tachi.spawn().infer_err()?),
                jre: jre.to_path_buf(),
                tachidesk_path: tachidesk_jar.to_path_buf(),
                root_dir: root_dir.to_path_buf(),
            };
            Ok(client)
        }

        pub async fn download_if_needed(
            client: Client,
            tachidesk_path: impl AsRef<Path>,
        ) -> Result<Self, Error> {
            let tachidesk_path = tachidesk_path.as_ref().to_path_buf();
            let assets = tachidesk_path.join("assets");
            // TODO: check if new version is released
            let asset_info = assets.join("asset_info");
            let tachidesk_root_dir = tachidesk_path.join("data_root");

            if !assets.exists() || !asset_info.exists() {
                // TODO:
                // - need to communicate how much of the file is downloaded (progress bar)
                let res = client
                    .get("https://api.github.com/repos/Suwayomi/Tachidesk-Server/releases/latest")
                    .header("User-Agent", "kolekk")
                    .send()
                    .await
                    .look(|e| dbg!(e))
                    .infer_err()?
                    .text()
                    .await
                    .look(|e| dbg!(e))
                    .infer_err()?;
                let releases = serde_json::from_str::<GithubRelease>(&res)
                    .look(|e| dbg!(e))
                    .infer_err()?;
                let asset = releases
                    .assets
                    .into_iter()
                    .find(|r| r.name.ends_with("linux-x64.tar.gz"))
                    .bad_err("could not find required asset")?;
                let bytes = client
                    .get(asset.browser_download_url)
                    .send()
                    .await
                    .infer_err()?
                    .bytes()
                    .await
                    .infer_err()?;
                let out_dir = assets.clone();
                let tar_contents = tachidesk_path.join(
                    asset
                        .name
                        .strip_suffix(".tar.gz")
                        .bad_err("name contains .tar.gz. this won't fail")?,
                );

                let _r: Result<_, Error> = tokio::task::spawn_blocking(move || {
                    std::fs::create_dir_all(&tachidesk_path).infer_err()?;
                    let zip_path = tachidesk_path.join(&asset.name);
                    let mut tar_gz = File::create(&zip_path).infer_err()?;
                    std::io::copy(&mut Cursor::new(bytes), &mut tar_gz).infer_err()?;
                    // let tar = GzDecoder::new(tar_gz);
                    let tar_gz = File::open(zip_path).infer_err()?;
                    let decoder = MultiGzDecoder::new(BufReader::new(tar_gz));
                    let mut archive = Archive::new(decoder);
                    archive.unpack(&tachidesk_path).infer_err()?;
                    std::fs::rename(tar_contents, out_dir).infer_err()?;

                    let mut asset_info = File::create(asset_info).infer_err()?;
                    write!(&mut asset_info, "{}", asset.name).infer_err()?;
                    Ok(())
                })
                .await
                .infer_err()?;
            }

            Self::new(
                client,
                assets.join("jre"),
                assets.join("Tachidesk-Server.jar"),
                tachidesk_root_dir,
            )
        }

        async fn get_parsed<T: DeserializeOwned + Debug>(
            &self,
            url: impl reqwest::IntoUrl,
        ) -> Result<T, Error> {
            let res = super::common::get_parsed(&self.client, url).await?;
            Ok(res)
        }

        pub async fn get_server_info(&self) -> Result<About, Error> {
            self.get_parsed(format!("{}/api/v1/settings/about", BASE_URL))
                .await
        }

        pub async fn get_all_extensions(&self) -> Result<Vec<Extension>, Error> {
            self.get_parsed(format!("{}/api/v1/extension/list", BASE_URL))
                .await
        }

        pub fn get_extension_icon_url(&self, icon_url: impl AsRef<str>) -> String {
            format!("{}{}", BASE_URL, icon_url.as_ref())
        }

        pub async fn extension_action(
            &self,
            pkg_name: impl AsRef<str>,
            action: ExtensionAction,
        ) -> Result<(), Error> {
            let _res = self
                .client
                .get(format!(
                    "{}/api/v1/extension/{}/{}",
                    BASE_URL,
                    action.as_ref(),
                    pkg_name.as_ref()
                ))
                .send()
                .await
                .look(|e| dbg!(e))
                .infer_err()?;
            Ok(())
        }

        pub async fn get_manga_chapter_list(&self, manga_id: u64) -> Result<Vec<Chapter>, Error> {
            self.get_parsed(format!("{}/api/v1/manga/{}/chapters", BASE_URL, manga_id))
                .await
        }

        pub async fn get_chapter(
            &self,
            manga_id: u64,
            chapter_index: u64,
        ) -> Result<Chapter, Error> {
            self.get_parsed(format!(
                "{}/api/v1/manga/{}/chapter/{}",
                BASE_URL, manga_id, chapter_index
            ))
            .await
        }

        pub fn get_manga_page_url(&self, manga_id: u64, chapter_index: u64, page: u64) -> String {
            format!(
                "{}/api/v1/manga/{}/chapter/{}/page/{}",
                BASE_URL, manga_id, chapter_index, page
            )
        }

        pub fn get_manga_thumbnail_url(&self, manga_id: u64) -> String {
            format!("{}/api/v1/manga/{}/thumbnail", BASE_URL, manga_id)
        }

        pub async fn get_manga(&self, manga_id: u64) -> Result<Manga, Error> {
            self.get_parsed(format!("{}/api/v1/manga/{}", BASE_URL, manga_id))
                .await
        }

        pub async fn get_source_list(&self) -> Result<Vec<MangaSource>, Error> {
            self.get_parsed(format!("{}/api/v1/source/list", BASE_URL))
                .await
        }

        pub async fn get_source_filters(
            &self,
            source_id: impl AsRef<str>,
        ) -> Result<Vec<SourceFilter>, Error> {
            self.get_parsed(format!(
                "{}/api/v1/source/{}/filters?reset=false",
                BASE_URL,
                source_id.as_ref()
            ))
            .await
        }

        pub async fn get_latest_manga_list(
            &self,
            source_id: impl AsRef<str>,
            page: u64,
        ) -> Result<MangaListPage, Error> {
            self.get_parsed(format!(
                "{}/api/v1/source/{}/latest/{}",
                BASE_URL,
                source_id.as_ref(),
                page
            ))
            .await
        }

        pub async fn get_popular_manga_list(
            &self,
            source_id: impl AsRef<str>,
            page: u64,
        ) -> Result<MangaListPage, Error> {
            self.get_parsed(format!(
                "{}/api/v1/source/{}/popular/{}",
                BASE_URL,
                source_id.as_ref(),
                page
            ))
            .await
        }

        pub async fn search_manga_in(
            &self,
            source_id: impl AsRef<str>,
            query: impl AsRef<str>,
            page: u64,
        ) -> Result<MangaListPage, Error> {
            self.get_parsed(
                Url::parse_with_params(
                    &format!("{}/api/v1/source/{}/search", BASE_URL, source_id.as_ref(),),
                    &[
                        ("searchTerm", query.as_ref()),
                        ("pageNum", &page.to_string()),
                    ],
                )
                .infer_err()?,
            )
            .await
        }
    }
}
