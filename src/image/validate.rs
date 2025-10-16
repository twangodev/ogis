use super::error::ImageFetchError;
use super::fetch::FetchedImage;

/// Stage 3: Validate content type using magic numbers
pub fn validate_content_type(fetched: FetchedImage) -> Result<Vec<u8>, ImageFetchError> {
    // Validate content-type using magic numbers (not headers!)
    let kind = infer::get(&fetched.bytes).ok_or_else(|| {
        tracing::warn!(
            "Could not determine file type for image from {}",
            fetched.url
        );
        ImageFetchError::InvalidContentType
    })?;

    // Allow only safe image types
    let mime_type = kind.mime_type();
    if !matches!(
        mime_type,
        "image/png" | "image/jpeg" | "image/gif" | "image/webp" | "image/svg+xml"
    ) {
        tracing::warn!("Invalid image type from {}: {}", fetched.url, mime_type);
        return Err(ImageFetchError::InvalidContentType);
    }

    tracing::info!(
        "Successfully validated {} image ({} bytes) from {}",
        mime_type,
        fetched.bytes.len(),
        fetched.url
    );

    Ok(fetched.bytes)
}
