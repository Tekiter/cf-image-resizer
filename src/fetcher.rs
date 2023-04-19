use futures::stream::StreamExt;
use worker::{Fetch, Url};

pub enum ImageFetchError {
    InvalidUrl,
    FailedToFetch,
}

pub async fn fetch_image_from_url(url: &str) -> Result<Vec<u8>, ImageFetchError> {
    let url = Url::parse(url).map_err(|_| ImageFetchError::InvalidUrl)?;

    let fetcher = Fetch::Url(url);
    let mut res = fetcher
        .send()
        .await
        .map_err(|_| ImageFetchError::FailedToFetch)?;

    let buffer = res.stream().map_err(|_| ImageFetchError::FailedToFetch)?;

    let bytes = buffer.map(|entry| entry.unwrap_or(vec![])).concat().await;

    Ok(bytes)
}
