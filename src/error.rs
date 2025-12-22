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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_field_too_long_status() {
        let err = ApiError::validation_field_too_long("Title", 1000);
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
        assert!(matches!(err.code, ErrorCode::ValidationFieldTooLong));
        assert_eq!(err.field, Some("title".to_string()));
    }

    #[test]
    fn test_validation_invalid_url_status() {
        let err = ApiError::validation_invalid_url("logo", "parse error");
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
        assert!(matches!(err.code, ErrorCode::ValidationInvalidUrl));
        assert_eq!(err.field, Some("logo".to_string()));
        assert_eq!(err.details, Some("parse error".to_string()));
    }

    #[test]
    fn test_validation_invalid_hex_color_status() {
        let err = ApiError::validation_invalid_hex_color("background", "XYZ123");
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
        assert!(matches!(err.code, ErrorCode::ValidationInvalidHexColor));
        assert_eq!(err.field, Some("background".to_string()));
    }

    #[test]
    fn test_auth_missing_signature_status() {
        let err = ApiError::auth_missing_signature();
        assert_eq!(err.status_code(), StatusCode::UNAUTHORIZED);
        assert!(matches!(err.code, ErrorCode::AuthMissingSignature));
        assert_eq!(err.field, Some("signature".to_string()));
    }

    #[test]
    fn test_auth_invalid_signature_status() {
        let err = ApiError::auth_invalid_signature();
        assert_eq!(err.status_code(), StatusCode::FORBIDDEN);
        assert!(matches!(err.code, ErrorCode::AuthInvalidSignature));
    }

    #[test]
    fn test_auth_invalid_signature_format_status() {
        let err = ApiError::auth_invalid_signature_format();
        assert_eq!(err.status_code(), StatusCode::FORBIDDEN);
        assert!(matches!(err.code, ErrorCode::AuthInvalidSignatureFormat));
        assert_eq!(err.field, Some("signature".to_string()));
    }

    #[test]
    fn test_ssrf_blocked_status() {
        let err = ApiError::ssrf_blocked("192.168.1.1");
        assert_eq!(err.status_code(), StatusCode::FORBIDDEN);
        assert!(matches!(err.code, ErrorCode::SsrfPrivateIpBlocked));
        assert_eq!(err.details, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_template_not_found_status() {
        let err = ApiError::template_not_found("missing", "a, b, c");
        assert_eq!(err.status_code(), StatusCode::NOT_FOUND);
        assert!(matches!(err.code, ErrorCode::TemplateNotFound));
        assert!(err.message.contains("missing"));
        assert!(err.details.as_ref().unwrap().contains("a, b, c"));
    }

    #[test]
    fn test_image_too_large_status() {
        let err = ApiError::image_too_large();
        assert_eq!(err.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(matches!(err.code, ErrorCode::ImageTooLarge));
    }

    #[test]
    fn test_image_invalid_content_type_status() {
        let err = ApiError::image_invalid_content_type();
        assert_eq!(err.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(matches!(err.code, ErrorCode::ImageInvalidContentType));
        assert!(err.details.is_some());
    }

    #[test]
    fn test_image_http_not_allowed_status() {
        let err = ApiError::image_http_not_allowed();
        assert_eq!(err.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(matches!(err.code, ErrorCode::ImageHttpNotAllowed));
    }

    #[test]
    fn test_render_failed_status() {
        let err = ApiError::render_failed("pixmap error");
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(matches!(err.code, ErrorCode::RenderFailed));
        assert_eq!(err.details, Some("pixmap error".to_string()));
    }

    #[test]
    fn test_svg_parse_failed_status() {
        let err = ApiError::svg_parse_failed("invalid xml");
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(matches!(err.code, ErrorCode::SvgParseFailed));
    }

    #[test]
    fn test_png_encode_failed_status() {
        let err = ApiError::png_encode_failed("encoding error");
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(matches!(err.code, ErrorCode::PngEncodeFailed));
    }

    #[test]
    fn test_font_error_status() {
        let err = ApiError::font_error("font not found");
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(matches!(err.code, ErrorCode::FontError));
    }

    #[test]
    fn test_internal_status() {
        let err = ApiError::internal("unexpected error");
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(matches!(err.code, ErrorCode::InternalError));
        assert_eq!(err.message, "unexpected error");
    }

    #[test]
    fn test_upstream_fetch_failed_status() {
        let err = ApiError::upstream_fetch_failed("connection refused");
        assert_eq!(err.status_code(), StatusCode::BAD_GATEWAY);
        assert!(matches!(err.code, ErrorCode::UpstreamFetchFailed));
    }

    #[test]
    fn test_service_overloaded_status() {
        let err = ApiError::service_overloaded();
        assert_eq!(err.status_code(), StatusCode::SERVICE_UNAVAILABLE);
        assert!(matches!(err.code, ErrorCode::ServiceOverloaded));
    }

    #[test]
    fn test_upstream_timeout_status() {
        let err = ApiError::upstream_timeout();
        assert_eq!(err.status_code(), StatusCode::GATEWAY_TIMEOUT);
        assert!(matches!(err.code, ErrorCode::UpstreamTimeout));
    }

    // ========== ErrorCode Serialization Tests ==========

    #[test]
    fn test_error_code_serialization() {
        assert_eq!(
            serde_json::to_string(&ErrorCode::ValidationFieldTooLong).unwrap(),
            "\"VALIDATION_FIELD_TOO_LONG\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::ValidationInvalidUrl).unwrap(),
            "\"VALIDATION_INVALID_URL\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::AuthMissingSignature).unwrap(),
            "\"AUTH_MISSING_SIGNATURE\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::SsrfPrivateIpBlocked).unwrap(),
            "\"SSRF_PRIVATE_IP_BLOCKED\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::TemplateNotFound).unwrap(),
            "\"TEMPLATE_NOT_FOUND\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::ImageTooLarge).unwrap(),
            "\"IMAGE_TOO_LARGE\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::UpstreamFetchFailed).unwrap(),
            "\"UPSTREAM_FETCH_FAILED\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::ServiceOverloaded).unwrap(),
            "\"SERVICE_OVERLOADED\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::UpstreamTimeout).unwrap(),
            "\"UPSTREAM_TIMEOUT\""
        );
    }

    // ========== From<ImageFetchError> Tests ==========

    #[test]
    fn test_from_image_fetch_error_request() {
        let err: ApiError = ImageFetchError::Request("connection refused".to_string()).into();
        assert_eq!(err.status_code(), StatusCode::BAD_GATEWAY);
        assert!(matches!(err.code, ErrorCode::UpstreamFetchFailed));
    }

    #[test]
    fn test_from_image_fetch_error_timeout() {
        let err: ApiError = ImageFetchError::Request("request timed out".to_string()).into();
        assert_eq!(err.status_code(), StatusCode::GATEWAY_TIMEOUT);
        assert!(matches!(err.code, ErrorCode::UpstreamTimeout));
    }

    #[test]
    fn test_from_image_fetch_error_timeout_uppercase() {
        let err: ApiError = ImageFetchError::Request("TIMEOUT error".to_string()).into();
        assert_eq!(err.status_code(), StatusCode::GATEWAY_TIMEOUT);
        assert!(matches!(err.code, ErrorCode::UpstreamTimeout));
    }

    #[test]
    fn test_from_image_fetch_error_too_large() {
        let err: ApiError = ImageFetchError::TooLarge.into();
        assert_eq!(err.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(matches!(err.code, ErrorCode::ImageTooLarge));
    }

    #[test]
    fn test_from_image_fetch_error_invalid_content_type() {
        let err: ApiError = ImageFetchError::InvalidContentType.into();
        assert_eq!(err.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(matches!(err.code, ErrorCode::ImageInvalidContentType));
    }

    #[test]
    fn test_from_image_fetch_error_private_ip_blocked() {
        let err: ApiError = ImageFetchError::PrivateIpBlocked("10.0.0.1".to_string()).into();
        assert_eq!(err.status_code(), StatusCode::FORBIDDEN);
        assert!(matches!(err.code, ErrorCode::SsrfPrivateIpBlocked));
    }

    #[test]
    fn test_from_image_fetch_error_http_not_allowed() {
        let err: ApiError = ImageFetchError::InvalidUrl("HTTP URLs not allowed".to_string()).into();
        assert_eq!(err.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(matches!(err.code, ErrorCode::ImageHttpNotAllowed));
    }

    #[test]
    fn test_from_image_fetch_error_invalid_url() {
        let err: ApiError = ImageFetchError::InvalidUrl("parse error".to_string()).into();
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
        assert!(matches!(err.code, ErrorCode::ValidationInvalidUrl));
    }

    // ========== From<HmacError> Tests ==========

    #[test]
    fn test_from_hmac_error_missing_signature() {
        let err: ApiError = HmacError::MissingSignature.into();
        assert_eq!(err.status_code(), StatusCode::UNAUTHORIZED);
        assert!(matches!(err.code, ErrorCode::AuthMissingSignature));
    }

    #[test]
    fn test_from_hmac_error_invalid_hex_format() {
        let hex_err = hex::FromHexError::InvalidHexCharacter { c: 'g', index: 0 };
        let err: ApiError = HmacError::InvalidHexFormat(hex_err).into();
        assert_eq!(err.status_code(), StatusCode::FORBIDDEN);
        assert!(matches!(err.code, ErrorCode::AuthInvalidSignatureFormat));
    }

    // ========== From<GeneratorError> Tests ==========

    #[test]
    fn test_from_generator_error_template_not_found() {
        let err: ApiError = GeneratorError::TemplateNotFound {
            name: "missing".to_string(),
            available: "a, b".to_string(),
        }
        .into();
        assert_eq!(err.status_code(), StatusCode::NOT_FOUND);
        assert!(matches!(err.code, ErrorCode::TemplateNotFound));
    }

    #[test]
    fn test_from_generator_error_font_properties_not_found() {
        let err: ApiError = GeneratorError::FontPropertiesNotFound("template".to_string()).into();
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(matches!(err.code, ErrorCode::FontError));
    }

    #[test]
    fn test_from_generator_error_svg_parse() {
        let err: ApiError = GeneratorError::SvgParse("invalid xml".to_string()).into();
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(matches!(err.code, ErrorCode::SvgParseFailed));
    }

    #[test]
    fn test_from_generator_error_utf8() {
        let err: ApiError = GeneratorError::Utf8("invalid utf8".to_string()).into();
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(matches!(err.code, ErrorCode::InternalError));
    }

    #[test]
    fn test_from_generator_error_png_encode() {
        let err: ApiError = GeneratorError::PngEncode("encode error".to_string()).into();
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(matches!(err.code, ErrorCode::PngEncodeFailed));
    }

    #[test]
    fn test_from_generator_error_pixmap_creation() {
        let err: ApiError = GeneratorError::PixmapCreation.into();
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(matches!(err.code, ErrorCode::RenderFailed));
    }

    #[test]
    fn test_from_generator_error_xml() {
        let err: ApiError = GeneratorError::Xml("write error".to_string()).into();
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(matches!(err.code, ErrorCode::RenderFailed));
    }

    #[test]
    fn test_from_generator_error_text_measurement() {
        let err: ApiError = GeneratorError::TextMeasurement("measurement error".to_string()).into();
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(matches!(err.code, ErrorCode::RenderFailed));
    }

    // ========== Builder Pattern Tests ==========

    #[test]
    fn test_with_details() {
        let err = ApiError::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::ValidationInvalidUrl,
            "msg",
        )
        .with_details("extra info");
        assert_eq!(err.details, Some("extra info".to_string()));
    }

    #[test]
    fn test_with_field() {
        let err = ApiError::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::ValidationInvalidUrl,
            "msg",
        )
        .with_field("logo");
        assert_eq!(err.field, Some("logo".to_string()));
    }

    #[test]
    fn test_builder_chain() {
        let err = ApiError::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::ValidationInvalidUrl,
            "msg",
        )
        .with_field("logo")
        .with_details("reason");
        assert_eq!(err.field, Some("logo".to_string()));
        assert_eq!(err.details, Some("reason".to_string()));
    }

    // ========== Display Tests ==========

    #[test]
    fn test_display() {
        let err = ApiError::validation_field_too_long("Title", 1000);
        assert_eq!(format!("{}", err), "Title exceeds maximum length of 1000");
    }

    #[test]
    fn test_display_simple_message() {
        let err = ApiError::service_overloaded();
        assert_eq!(format!("{}", err), "Server overloaded, try again later");
    }

    // ========== JSON Structure Tests ==========

    #[test]
    fn test_error_body_json_structure() {
        let body = ErrorBody {
            error: ErrorDetail {
                code: ErrorCode::TemplateNotFound,
                message: "Template 'foo' not found".to_string(),
                details: Some("Available templates: bar".to_string()),
                field: None,
            },
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("\"code\":\"TEMPLATE_NOT_FOUND\""));
        assert!(json.contains("\"message\":\"Template 'foo' not found\""));
        assert!(json.contains("\"details\":\"Available templates: bar\""));
        // field should be omitted when None
        assert!(!json.contains("\"field\""));
    }

    #[test]
    fn test_optional_fields_omitted() {
        let body = ErrorBody {
            error: ErrorDetail {
                code: ErrorCode::ServiceOverloaded,
                message: "Server overloaded".to_string(),
                details: None,
                field: None,
            },
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(!json.contains("details"));
        assert!(!json.contains("field"));
    }

    #[test]
    fn test_optional_fields_present() {
        let body = ErrorBody {
            error: ErrorDetail {
                code: ErrorCode::ValidationFieldTooLong,
                message: "Title too long".to_string(),
                details: Some("Max 1000".to_string()),
                field: Some("title".to_string()),
            },
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("\"details\":\"Max 1000\""));
        assert!(json.contains("\"field\":\"title\""));
    }
}
