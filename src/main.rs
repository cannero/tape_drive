mod downloader;
mod nom_parser;

type AsyncError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[async_std::main]
async fn main() -> Result<(), AsyncError> {
    let twitch_streamer = downloader::get_twitch_user("ksivamuthu".to_string()).await?;
    println!("{:?}", twitch_streamer);
    return Ok(());

    let file = downloader::download_file(
        "https://raw.githubusercontent.com/bnb/awesome-developer-streams/master/README.md",
    )
    .await?;

    let streamers = nom_parser::parse_file(&file)?;
    for streamer in streamers {
        println!("{:?}", streamer);
    }

    Ok(())
}
