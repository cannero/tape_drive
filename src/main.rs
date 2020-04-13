mod downloader;

type AsyncError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[async_std::main]
async fn main() -> Result<(), AsyncError> {
    let file = downloader::download_file(
        "https://raw.githubusercontent.com/bnb/awesome-developer-streams/master/README.md",
    )
    .await?;

    dbg!(file);
    Ok(())
}
