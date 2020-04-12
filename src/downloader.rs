type DownloadResult = Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>;
pub async fn download_file(url: &str) -> DownloadResult {
    let mut res = surf::get(url).await?;
    dbg!(res.body_string().await?);
    Ok(())
}
