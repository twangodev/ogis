use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::ToSchema;

use crate::auth::HmacError;
use crate::generator::GeneratorError;
use crate::image::ImageFetchError;

/// Machine-readable error codes for programmatic handling
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Validation errors (400)
    ValidationFieldTooLong,
    ValidationInvalidUrl,
    ValidationInvalidHexColor,

    // Auth errors (401)
    AuthMissingSignature,

    // Forbidden errors (403)
    AuthInvalidSignature,
    AuthInvalidSignatureFormat,
    SsrfPrivateIpBlocked,

    // Not found errors (404)
    TemplateNotFound,

    // Unprocessable entity errors (422)
    ImageTooLarge,
    ImageInvalidContentType,
    ImageHttpNotAllowed,

    // Internal errors (500)
    RenderFailed,
    SvgParseFailed,
    PngEncodeFailed,
    FontError,
    InternalError,

    // Upstream errors (502)
    UpstreamFetchFailed,

    // Overload errors (503)
    ServiceOverloaded,

    // Timeout errors (504)
    UpstreamTimeout,
}

/// JSON error response body
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

/// Error detail within the response
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorDetail {
    /// Machine-readable error code
    pub code: ErrorCode,
    /// Human-readable error message
    pub message: String,
    /// Additional details about the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// Field that caused the error (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

/// Main API error type
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: ErrorCode,
    message: String,
    details: Option<String>,
    field: Option<String>,
}

impl ApiError {
    pub fn new(status: StatusCode, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            details: None,
            field: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        self.status
    }

    // ========== Validation Errors (400) ==========

    pub fn validation_field_too_long(field: &str, max_len: usize) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::ValidationFieldTooLong,
            format!("{} exceeds maximum length of {}", field, max_len),
        )
        .with_field(field.to_lowercase())
    }

    pub fn validation_invalid_url(field: &str, reason: &str) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::ValidationInvalidUrl,
            "Invalid URL format",
        )
        .with_field(field.to_lowercase())
        .with_details(reason)
    }

    pub fn validation_invalid_hex_color(field: &str, value: &str) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::ValidationInvalidHexColor,
            format!("Invalid hex color '{}' for parameter '{}'", value, field),
        )
        .with_field(field.to_string())
        .with_details("Expected 6 hex characters (e.g., 'FF0000')")
    }

    // ========== Auth Errors (401/403) ==========

    pub fn auth_missing_signature() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            ErrorCode::AuthMissingSignature,
            "Authentication required: missing signature parameter",
        )
        .with_field("signature")
    }

    pub fn auth_invalid_signature() -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            ErrorCode::AuthInvalidSignature,
            "Authentication failed: invalid signature",
        )
    }

    pub fn auth_invalid_signature_format() -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            ErrorCode::AuthInvalidSignatureFormat,
            "Invalid signature format: must be hex-encoded",
        )
        .with_field("signature")
    }

    // ========== Forbidden Errors (403) ==========

    pub fn ssrf_blocked(details: &str) -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            ErrorCode::SsrfPrivateIpBlocked,
            "SSRF protection: request to private IP blocked",
        )
        .with_details(details)
    }

    // ========== Not Found Errors (404) ==========

    pub fn template_not_found(name: &str, available: &str) -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            ErrorCode::TemplateNotFound,
            format!("Template '{}' not found", name),
        )
        .with_details(format!("Available templates: {}", available))
    }

    // ========== Unprocessable Entity Errors (422) ==========

    pub fn image_too_large() -> Self {
        Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            ErrorCode::ImageTooLarge,
            "Image exceeds maximum allowed size",
        )
    }

    pub fn image_invalid_content_type() -> Self {
        Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            ErrorCode::ImageInvalidContentType,
            "File is not a supported image type",
        )
        .with_details("Supported types: PNG, JPEG, GIF, WebP, SVG")
    }

    pub fn image_http_not_allowed() -> Self {
        Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            ErrorCode::ImageHttpNotAllowed,
            "HTTP URLs not allowed",
        )
        .with_details("Use HTTPS or enable --allow-http flag")
    }

    // ========== Internal Errors (500) ==========

    pub fn render_failed(details: &str) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::RenderFailed,
            "Failed to render image",
        )
        .with_details(details)
    }

    pub fn svg_parse_failed(details: &str) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::SvgParseFailed,
            "Failed to parse SVG template",
        )
        .with_details(details)
    }

    pub fn png_encode_failed(details: &str) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::PngEncodeFailed,
            "Failed to encode PNG",
        )
        .with_details(details)
    }

    pub fn font_error(details: &str) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::FontError,
            "Template font configuration error",
        )
        .with_details(details)
    }

    pub fn internal(message: &str) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InternalError,
            message,
        )
    }

    // ========== Upstream Errors (502) ==========

    pub fn upstream_fetch_failed(details: &str) -> Self {
        Self::new(
            StatusCode::BAD_GATEWAY,
            ErrorCode::UpstreamFetchFailed,
            "Failed to fetch remote image",
        )
        .with_details(details)
    }

    // ========== Overload Errors (503) ==========

    pub fn service_overloaded() -> Self {
        Self::new(
            StatusCode::SERVICE_UNAVAILABLE,
            ErrorCode::ServiceOverloaded,
            "Server overloaded, try again later",
        )
    }

    // ========== Timeout Errors (504) ==========

    pub fn upstream_timeout() -> Self {
        Self::new(
            StatusCode::GATEWAY_TIMEOUT,
            ErrorCode::UpstreamTimeout,
            "Image fetch timed out",
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorBody {
            error: ErrorDetail {
                code: self.code,
                message: self.message,
                details: self.details,
                field: self.field,
            },
        };

        (self.status, Json(body)).into_response()
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ApiError {}

// ========== From Implementations ==========

impl From<ImageFetchError> for ApiError {
    fn from(err: ImageFetchError) -> Self {
        match err {
            ImageFetchError::Request(msg) => {
                // Check if it's a timeout
                let msg_lower = msg.to_lowercase();
                if msg_lower.contains("timed out") || msg_lower.contains("timeout") {
                    ApiError::upstream_timeout().with_details(msg)
                } else {
                    ApiError::upstream_fetch_failed(&msg)
                }
            }
            ImageFetchError::TooLarge => ApiError::image_too_large(),
            ImageFetchError::InvalidContentType => ApiError::image_invalid_content_type(),
            ImageFetchError::PrivateIpBlocked(msg) => ApiError::ssrf_blocked(&msg),
            ImageFetchError::InvalidUrl(msg) => {
                if msg.contains("HTTP URLs not allowed") || msg.contains("http://") {
                    ApiError::image_http_not_allowed()
                } else {
                    ApiError::validation_invalid_url("url", &msg)
                }
            }
        }
    }
}

impl From<HmacError> for ApiError {
    fn from(err: HmacError) -> Self {
        match err {
            HmacError::MissingSignature => ApiError::auth_missing_signature(),
            HmacError::InvalidSignature(_) => ApiError::auth_invalid_signature(),
            HmacError::InvalidHexFormat(_) => ApiError::auth_invalid_signature_format(),
        }
    }
}

impl From<GeneratorError> for ApiError {
    fn from(err: GeneratorError) -> Self {
        match err {
            GeneratorError::TemplateNotFound { name, available } => {
                ApiError::template_not_found(&name, &available)
            }
            GeneratorError::FontPropertiesNotFound(template) => ApiError::font_error(&format!(
                "Font properties not found for template '{}'",
                template
            )),
            GeneratorError::SvgParse(msg) => ApiError::svg_parse_failed(&msg),
            GeneratorError::Utf8(msg) => {
                ApiError::internal("UTF-8 encoding error").with_details(msg)
            }
            GeneratorError::PngEncode(msg) => ApiError::png_encode_failed(&msg),
            GeneratorError::PixmapCreation => ApiError::render_failed("Failed to create pixmap"),
            GeneratorError::Xml(msg) => ApiError::render_failed(&msg),
            GeneratorError::TextMeasurement(msg) => ApiError::render_failed(&msg),
        }
    }
}
