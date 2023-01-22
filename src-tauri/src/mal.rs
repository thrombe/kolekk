use lib_mal::MALClient;
use lib_mal::{ClientBuilder, MALError};
use std::path::PathBuf;
use tauri::State;

pub async fn mal_init() -> Result<lib_mal::MALClient, MALError> {
    let client_id = include_str!("../../cache/mal_client_id").to_owned();
    // the MALClient will attempt to refresh the cached access_token, if applicable
    let client = ClientBuilder::new()
        .secret(client_id)
        .caching(true)
        .cache_dir(PathBuf::from(include_str!("../../cache/cache_dir")))
        .build_with_refresh()
        .await?;

    // if client.need_auth {
    //     // this has to exactly match a URI that's been registered with the MAL api
    //     let redirect = "http://localhost:2525";
    //     let (auth_url, challenge, state) = client.get_auth_parts();
    //     // the user will have to have access to a browser in order to log in and
    //     // give your application permission
    //     println!("Go here to log in :) -> {}", auth_url);
    //     // once the user has the URL, be sure to call client.auth to listen for
    //     // the callback and complete the OAuth2 handshake
    //     client.auth(&redirect, &challenge, &state).await?;
    // }
    Ok(client)
}

#[tauri::command]
pub async fn get_seasonal_anime(
    mal: State<'_, MalClient>,
) -> Result<lib_mal::prelude::AnimeList, MALError> {
    let mal = mal.0.clone();
    let ranking_list = mal
        .get_anime_ranking(lib_mal::prelude::options::RankingType::ByPopularity, 50)
        .await?;
    // try to add "next page" functionality to the lib_mal crate
    dbg!(&ranking_list);
    Ok(ranking_list)
}

pub struct MalClient(pub std::sync::Arc<MALClient>);

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MalAuthData {
    auth_url: String,
    challenge: String,
    state: String,
}

#[tauri::command]
pub fn mal_auth_needed(mal: State<'_, MalClient>) -> Option<MalAuthData> {
    if mal.0.need_auth {
        let (auth_url, challenge, state) = mal.0.get_auth_parts();
        Some(MalAuthData {
            auth_url,
            challenge,
            state,
        })
    } else {
        None
    }
}

#[tauri::command]
pub async fn mal_auth(auth_data: MalAuthData, mal: State<'_, MalClient>) -> Result<(), MALError> {
    if std::sync::Arc::strong_count(&mal.0) + std::sync::Arc::weak_count(&mal.0) > 1 {
        return Err(MALError::new("refcount is not 1", "None", None));
    }
    let redirect = "http://localhost:2525";

    // this stuff is not safe, tho idk any other way to do this
    let a = std::sync::Arc::as_ptr(&mal.0);
    let a = unsafe { &mut *(a as *mut MALClient) };
    tauri::async_runtime::block_on(a.auth(
        redirect,
        &auth_data.challenge,
        &auth_data.state,
    ))?;
    Ok(())
}
