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
    users: Vec<TwitchUserData>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TwitchUserData {
    id: String,
    display_name: String,
    view_count: u32,
}

pub async fn get_twitch_user(login: String) -> Result<TwitchUserData, AsyncError> {
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

fn build_urls(login_names: Vec<String>, chunk_size: usize) -> Vec<String> {
    login_names
        .chunks(chunk_size)
        .map(|streamer_chunk| {
            streamer_chunk
                .iter()
                .enumerate()
                .map(|(i, login_name)| {
                    let fmt = if i == 0 {
                        "https://api.twitch.tv/helix/users?login="
                    } else {
                        "&login="
                    };
                    format!("{}{}", fmt, login_name)
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_urls() {
        let login_names = vec!["s_1".to_string(), "s_2".to_string(), "s_3".to_string()];

        let urls = build_urls(login_names, 2);

        assert_eq!(
            urls,
            vec![
                "https://api.twitch.tv/helix/users?login=s_1&login=s_2",
                "https://api.twitch.tv/helix/users?login=s_3"
            ]
        );
    }
}
