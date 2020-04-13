use crate::AsyncError;

type DownloadResult = Result<String, AsyncError>;
pub async fn download_file(url: &str) -> DownloadResult {
    let mut res = surf::get(url).await?;
    Ok(res.body_string().await?)
}
