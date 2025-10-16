use futures_util::TryStreamExt;
use reqwest::Client;

use super::error::ImageFetchError;
use super::parse::ParsedUrl;

/// Fetched image bytes
#[derive(Debug)]
pub struct FetchedImage {
    pub bytes: Vec<u8>,
    pub url: String,
}

/// Stage 2: Fetch image via HTTP (SSRF protection handled by custom DNS resolver)
pub async fn fetch_http(
    parsed: ParsedUrl,
    client: &Client,
    max_size: usize,
) -> Result<FetchedImage, ImageFetchError> {
    tracing::info!("Fetching image from URL: {}", parsed.original);

    // Make HTTP request (custom DNS resolver validates IPs)
    let response = client
        .get(parsed.url.as_str())
        .send()
        .await
        .map_err(|e| ImageFetchError::Request(e.to_string()))?;

    // Check status
    if !response.status().is_success() {
        return Err(ImageFetchError::Request(format!(
            "HTTP {}",
            response.status()
        )));
    }

    // Check Content-Length if available to avoid downloading large files
    if let Some(content_length) = response.content_length() {
        if content_length as usize > max_size {
            tracing::warn!(
                "Image from {} exceeds max size (Content-Length): {} > {}",
                parsed.original,
                content_length,
                max_size
            );
            return Err(ImageFetchError::TooLarge);
        }
    }

    // Stream response body with size limit enforcement
    let bytes = response
        .bytes_stream()
        .map_err(|e| ImageFetchError::Request(format!("Failed to read response chunk: {}", e)))
        .try_fold(Vec::new(), |mut acc, chunk| {
            let url = parsed.original.clone();
            async move {
                // Check if adding this chunk would exceed max_size
                if acc.len() + chunk.len() > max_size {
                    tracing::warn!(
                        "Image from {} exceeds max size while streaming: {} + {} > {}",
                        url,
                        acc.len(),
                        chunk.len(),
                        max_size
                    );
                    return Err(ImageFetchError::TooLarge);
                }

                acc.extend_from_slice(&chunk);
                Ok(acc)
            }
        })
        .await?;

    Ok(FetchedImage {
        bytes,
        url: parsed.original,
    })
}
