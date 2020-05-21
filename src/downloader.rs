use crate::AsyncError;
use async_std::{
    prelude::*,
    sync::{Arc, Mutex},
    task,
};
use futures::{channel::mpsc, future::join_all, sink::SinkExt};
use serde::Deserialize;
use std::collections::hash_map::{Entry, HashMap};

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

type DownloadResult = Result<String, AsyncError>;
pub async fn download_file(url: &str) -> DownloadResult {
    let mut res = surf::get(url).await?;
    Ok(res.body_string().await?)
}

const CLIENT_ID_KEY: &str = "TWITCH_CLIENT_ID";
const BEARER_TOKEN_KEY: &str = "BEARER_TOKEN_KEY";
const LOGIN_CHUNK_SIZE: usize = 100;

#[derive(Deserialize)]
struct UsersData {
    #[serde(rename = "data")]
    users: Vec<TwitchUserData>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TwitchUserData {
    id: String,
    display_name: String,
    view_count: u32,
    description: String,
    #[serde(skip)]
    follower_count: u32,
}

impl TwitchUserData {
    fn set_follower_count(&mut self, count: u32) {
        self.follower_count = count;
    }
}

#[derive(Deserialize, Debug)]
pub struct TwitchFollowers {
    total: u32,
    #[serde(skip)]
    id: String,
}

pub async fn get_twitch_users(
    login_names: Vec<&String>,
) -> Result<Vec<TwitchUserData>, AsyncError> {
    let urls = build_users_urls(&login_names, LOGIN_CHUNK_SIZE);
    let mut res = vec![];
    for url in urls {
        res.extend(get_data_for_twitch_users(&url).await?);
    }
    Ok(res)
}

pub async fn get_twitch_users_parallel(
    login_names: Vec<&String>,
) -> Result<HashMap<String, TwitchUserData>, AsyncError> {
    let streamers_mutex = Arc::new(Mutex::new(HashMap::new()));
    let (streamer_sender, streamer_receiver) = mpsc::unbounded();
    let receiver_handle = task::spawn(streamer_received_loop(
        streamer_receiver,
        streamers_mutex.clone(),
    ));
    let urls = build_users_urls(&login_names, LOGIN_CHUNK_SIZE / 2);
    //println!("urls: {}, login_names: {}", urls.len(), login_names.len());
    for chunk in urls.chunks(4) {
        let mut workers = vec![];
        for url in chunk {
            workers.push(get_and_send_users_data(&url, streamer_sender.clone()));
        }
        let _res = join_all(workers).await;
        //println!("{:?}", res);
    }
    drop(streamer_sender);
    receiver_handle.await?;

    add_followers(streamers_mutex.clone()).await?;

    //println!("entries: {:?}", streamers_mutex.lock().await);

    //let streamers = &*streamers_mutex.lock().await;
    Ok(Arc::try_unwrap(streamers_mutex).unwrap().into_inner())
}

async fn streamer_received_loop(
    mut events: Receiver<Vec<TwitchUserData>>,
    streamers: Arc<Mutex<HashMap<String, TwitchUserData>>>,
) -> Result<(), AsyncError> {
    let mut streamers = streamers.lock().await;
    while let Some(event) = events.next().await {
        for streamer in event {
            match streamers.entry(streamer.id.clone()) {
                Entry::Occupied(_) => panic!("streamer {:?} already in map", streamer.id),
                Entry::Vacant(entry) => entry.insert(streamer),
            };
        }
    }
    print!("all events received");
    Ok(())
}

async fn get_and_send_users_data(
    url: &String,
    mut broker: Sender<Vec<TwitchUserData>>,
) -> Result<(), AsyncError> {
    let data = get_data_for_twitch_users(url).await?;
    broker.send(data).await?;
    Ok(())
}

async fn get_data_for_twitch_users(url: &String) -> Result<Vec<TwitchUserData>, AsyncError> {
    let client_id = match std::env::var(CLIENT_ID_KEY) {
        Ok(val) => val,
        Err(e) => panic!("twich client id not found, key {}, {}", CLIENT_ID_KEY, e),
    };
    let bearer_token = match std::env::var(BEARER_TOKEN_KEY) {
        Ok(val) => format!("Bearer {}", val),
        Err(e) => panic!(
            "twich bearer token not found, key {}, {}",
            BEARER_TOKEN_KEY, e
        ),
    };
    //println!("{}", url);
    let UsersData { users } = surf::get(url)
        .set_header("Authorization".parse().unwrap(), bearer_token)
        .set_header("Client-ID".parse().unwrap(), client_id)
        .recv_json()
        .await?;

    Ok(users)
}

async fn add_followers(
    streamers_mutex: Arc<Mutex<HashMap<String, TwitchUserData>>>,
) -> Result<(), AsyncError> {
    println!("adding followers");
    let streamers = streamers_mutex.lock().await;
    println!("{} streamers", streamers.len());
    let follower_urls: Vec<(String, String)> = streamers
        .iter()
        .take(20)
        .map(|(k, _)| (build_followers_url(k), k.clone()))
        .collect();
    drop(streamers);
    println!("{} followers waiting", follower_urls.len());

    let (followers_sender, followers_receiver) = mpsc::unbounded();
    let receiver_handle = task::spawn(followers_received_loop(
        followers_receiver,
        streamers_mutex.clone(),
    ));

    for chunk in follower_urls.chunks(4) {
        let mut workers = vec![];
        for (url, id) in chunk {
            workers.push(get_and_send_followers(&url, &id, followers_sender.clone()));
        }
        let _res = join_all(workers).await;
        //println!("{:?}", res);
    }

    drop(followers_sender);
    receiver_handle.await?;
    Ok(())
}

async fn followers_received_loop(
    mut events: Receiver<TwitchFollowers>,
    streamers: Arc<Mutex<HashMap<String, TwitchUserData>>>,
) -> Result<(), AsyncError> {
    println!("starting receiver");
    let mut streamers = streamers.lock().await;
    while let Some(followers) = events.next().await {
        match streamers.entry(followers.id.clone()) {
            Entry::Occupied(mut entry) => {
                let tu = &mut *entry.get_mut();
                tu.set_follower_count(followers.total);
            }
            Entry::Vacant(_) => panic!("streamer {:?} not found", followers.id),
        };
    }
    Ok(())
}

async fn get_and_send_followers(
    url: &String,
    id: &String,
    mut broker: Sender<TwitchFollowers>,
) -> Result<(), AsyncError> {
    let mut follower = get_total_followers(url).await?;
    follower.id = id.clone();
    broker.send(follower).await?;
    Ok(())
}

async fn get_total_followers(url: &String) -> Result<TwitchFollowers, AsyncError> {
    let client_id = match std::env::var(CLIENT_ID_KEY) {
        Ok(val) => val,
        Err(e) => panic!("twich client id not found, key {}, {}", CLIENT_ID_KEY, e),
    };
    let bearer_token = match std::env::var(BEARER_TOKEN_KEY) {
        Ok(val) => format!("Bearer {}", val),
        Err(e) => panic!(
            "twich bearer token not found, key {}, {}",
            BEARER_TOKEN_KEY, e
        ),
    };
    //println!("{}", url);
    let follower: TwitchFollowers = surf::get(url)
        .set_header("Authorization".parse().unwrap(), bearer_token)
        .set_header("Client-ID".parse().unwrap(), client_id)
        .recv_json()
        .await?;

    Ok(follower)
}

fn build_users_urls(login_names: &Vec<&String>, chunk_size: usize) -> Vec<String> {
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

fn build_followers_url(user_id: &String) -> String {
    format!(
        "https://api.twitch.tv/helix/users/follows?to_id={}&first=1",
        user_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_urls() {
        let login_names = vec!["s_1".to_string(), "s_2".to_string(), "s_3".to_string()];

        let urls = build_users_urls(&login_names.iter().collect(), 2);

        assert_eq!(
            urls,
            vec![
                "https://api.twitch.tv/helix/users?login=s_1&login=s_2",
                "https://api.twitch.tv/helix/users?login=s_3"
            ]
        );
    }
}
