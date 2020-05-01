use crate::AsyncError;
use serde::{Deserialize, Serialize};

type DownloadResult = Result<String, AsyncError>;
pub async fn download_file(url: &str) -> DownloadResult {
    let mut res = surf::get(url).await?;
    Ok(res.body_string().await?)
}

const CLIENT_ID_KEY: &str = "TWITCH_CLIENT_ID";

#[derive(Deserialize, Serialize)]
struct UsersData {
    #[serde(rename = "data")]
    users: Vec<TwitchUser>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TwitchUser {
    id: String,
    display_name: String,
    view_count: u32,
}

pub async fn get_twitch_user(login: String) -> Result<TwitchUser, AsyncError> {
    let client_id = match std::env::var(CLIENT_ID_KEY) {
        Ok(val) => val,
        Err(e) => panic!("twich client id not found, key {}, {}", CLIENT_ID_KEY, e),
    };
    let url = format!("https://api.twitch.tv/helix/users?login={}", login);
    let UsersData { users } = surf::get(url)
        .set_header("Client-ID".parse().unwrap(), client_id)
        .recv_json()
        .await?;

    Ok(users[0].clone())
}
