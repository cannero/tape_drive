use async_std::task;
mod downloader;
fn main() {
    match task::block_on(async {
        downloader::download_file(
            "https://raw.githubusercontent.com/bnb/awesome-developer-streams/master/README.md",
        )
        .await
    }) {
        Err(e) => eprintln!("{}", e),
        _ => (),
    };
}
