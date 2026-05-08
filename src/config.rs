use bytesize::ByteSize;
use clap::{Args, Parser, ValueEnum};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ImageFallbackBehavior {
    /// Skip image element if fetch fails
    Skip,
    /// Return error if fetch fails
    Error,
}

/// Default values for OG image fields
#[derive(Clone, Debug, Args)]
pub struct Defaults {
    /// Default title when not provided
    #[arg(long, default_value = "Open Graph Images", env = "OGIS_DEFAULT_TITLE")]
    pub title: String,

    /// Default description when not provided
    #[arg(
        long,
        default_value = "A fast, free, and beautiful platform for open graph image generation.",
        env = "OGIS_DEFAULT_DESCRIPTION"
    )]
    pub description: String,

    /// Default subtitle when not provided
    #[arg(
        long,
        default_value = "Open Graph Images for Everyone",
        env = "OGIS_DEFAULT_SUBTITLE"
    )]
    pub subtitle: String,

    /// Default logo URL when not provided
    #[arg(
        long,
        default_value = "https://ogis.dev/logo-light.png",
        env = "OGIS_DEFAULT_LOGO"
    )]
    pub logo: String,
}

/// Image fetching and caching settings
#[derive(Clone, Debug, Args)]
pub struct ImageSettings {
    /// Logo fetch connect timeout in seconds
    #[arg(long, default_value = "5", env = "OGIS_LOGO_CONNECT_TIMEOUT")]
    pub connect_timeout_secs: u64,

    /// Logo fetch total timeout in seconds
    #[arg(long, default_value = "10", env = "OGIS_LOGO_TOTAL_TIMEOUT")]
    pub total_timeout_secs: u64,

    /// Maximum logo image size in bytes (default: 5MB)
    #[arg(long, default_value = "5242880", env = "OGIS_LOGO_MAX_SIZE")]
    pub max_size_bytes: usize,

    /// Cache maximum size (default: 2GB, supports formats like "512MB", "1.5GiB")
    #[arg(long, default_value = "2GB", env = "OGIS_CACHE_MAX_BYTES")]
    pub cache_max_bytes: ByteSize,

    /// Logo cache TTL in seconds (default: 1 hour)
    #[arg(long, default_value = "3600", env = "OGIS_LOGO_CACHE_TTL")]
    pub cache_ttl_secs: u64,

    /// Maximum redirects to follow for logo URLs
    #[arg(long, default_value = "3", env = "OGIS_LOGO_MAX_REDIRECTS")]
    pub max_redirects: usize,

    /// Allow HTTP (insecure) URLs for logo fetching (HTTPS only by default)
    #[arg(long, default_value = "false", env = "OGIS_ALLOW_HTTP")]
    pub allow_http: bool,

    /// Behavior when image URL fetch fails
    #[arg(long, default_value = "skip", env = "OGIS_IMAGE_FALLBACK")]
    pub fallback: ImageFallbackBehavior,
}

impl ImageSettings {
    /// Get effective cache size with safety bounds [256 MB, 8 GB]
    pub fn effective_cache_size(&self) -> u64 {
        const MIN_CACHE_BYTES: u64 = 256 * 1024 * 1024; // 256 MB
        const MAX_CACHE_BYTES: u64 = 8 * 1024 * 1024 * 1024; // 8 GB

        // Apply safety bounds
        self.cache_max_bytes
            .as_u64()
            .clamp(MIN_CACHE_BYTES, MAX_CACHE_BYTES)
    }
}

/// Gradient template prerender cache settings
#[derive(Clone, Debug, Args)]
pub struct GradientCacheSettings {
    /// Gradient cache maximum size (default: 512MB; clamped to [64MB, 2GB])
    #[arg(long, default_value = "512MB", env = "OGIS_GRADIENT_CACHE_MAX_BYTES")]
    pub max_bytes: ByteSize,

    /// Comma-separated list of gradient template names to prerender at startup
    /// (lazy fallback handles anything not listed). Example:
    /// "gradient-aurora-centered,gradient-noir-banner"
    #[arg(
        long,
        env = "OGIS_GRADIENT_WARMUP_TEMPLATES",
        value_delimiter = ',',
        default_value = ""
    )]
    pub warmup_templates: Vec<String>,
}

impl GradientCacheSettings {
    /// Effective cache size, clamped to [64MB, 2GB].
    pub fn effective_max_bytes(&self) -> u64 {
        const MIN_BYTES: u64 = 64 * 1024 * 1024;
        const MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;
        self.max_bytes.as_u64().clamp(MIN_BYTES, MAX_BYTES)
    }
}

/// HMAC authentication settings
#[derive(Clone, Debug, Args)]
pub struct HmacSettings {
    /// HMAC secret key for request signing (if set, auth is required)
    /// Generate with: openssl rand -hex 32
    #[arg(long, env = "OGIS_HMAC_SECRET")]
    pub secret: Option<String>,
}

impl HmacSettings {
    /// Check if HMAC authentication is enabled (non-empty secret configured)
    pub fn is_enabled(&self) -> bool {
        self.secret_bytes().is_some()
    }

    /// Get secret as bytes (returns None if not enabled or empty)
    pub fn secret_bytes(&self) -> Option<Vec<u8>> {
        self.secret
            .as_ref()
            .filter(|s| !s.is_empty())
            .map(|s| s.as_bytes().to_vec())
    }
}

/// Documentation/API settings
#[derive(Clone, Debug, Args)]
pub struct DocsSettings {
    /// Allowed CORS origins for /docs endpoints (comma-separated)
    /// Example: https://ogis.dev,https://www.ogis.dev
    #[arg(
        long,
        default_value = "https://ogis.dev",
        env = "OGIS_DOCS_CORS_ORIGINS"
    )]
    pub cors_origins: Option<String>,
}

impl DocsSettings {
    /// Get list of allowed CORS origins
    pub fn allowed_origins(&self) -> Option<Vec<String>> {
        self.cors_origins.as_ref().map(|origins| {
            origins
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
    }
}

#[derive(Parser)]
#[command(name = "ogis")]
#[command(version)]
#[command(about = "Open Graph Image Service - Generate OG images dynamically")]
pub struct Config {
    /// Export OpenAPI spec as JSON and exit
    #[arg(long)]
    pub export_openapi: bool,

    /// Address to bind the server to
    #[arg(short, long, default_value = "0.0.0.0:3000", env = "OGIS_ADDR")]
    pub addr: String,

    /// Maximum length for any input field
    #[arg(long, default_value = "1000", env = "OGIS_MAX_INPUT_LENGTH")]
    pub max_input_length: usize,

    #[command(flatten)]
    pub defaults: Defaults,

    #[command(flatten)]
    pub image: ImageSettings,

    #[command(flatten)]
    pub gradient_cache: GradientCacheSettings,

    #[command(flatten)]
    pub hmac: HmacSettings,

    #[command(flatten)]
    pub docs: DocsSettings,

    #[command(flatten)]
    pub otel: crate::telemetry::OtelSettings,
}

impl Config {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
