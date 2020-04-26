mod downloader;
mod nom_parser;

type AsyncError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[async_std::main]
async fn main() -> Result<(), AsyncError> {
    let file = downloader::download_file(
        "https://raw.githubusercontent.com/bnb/awesome-developer-streams/master/README.md",
    )
    .await?;

    let (_, color) = nom_parser::hex_color("Some text: #2F12A4")?;
    println!("{:?}", color);

    Ok(())
}
