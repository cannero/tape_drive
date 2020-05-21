use tape_drive::downloader;
use tape_drive::nom_parser;
use tape_drive::AsyncError;

#[async_std::main]
async fn main() -> Result<(), AsyncError> {
    let file = downloader::download_file(
        "https://raw.githubusercontent.com/bnb/awesome-developer-streams/master/README.md",
    )
    .await?;

    let streamers = nom_parser::parse_file(&file)?;
    // TODO build hashmap by iterating over streamers, (login, streamer).collect()
    // for streamer in streamers {
    //     println!("{:?}", streamer);
    // }
    println!("{} streamers from file", streamers.len());

    let mut args = std::env::args();
    if args.any(|a| a == "-s" || a == "--single") {
        let _twitch_streamers =
            downloader::get_twitch_users(streamers.iter().map(|s| s.login_name()).collect())
                .await?;
    //for tw_s in twitch_streamers {
    //   println!("{:?}", tw_s);
    //}
    } else {
        let streamers = downloader::get_twitch_users_parallel(
            streamers.iter().map(|s| s.login_name()).collect(),
        )
        .await?;
        println!("{:?}", streamers);
    }
    Ok(())
}
