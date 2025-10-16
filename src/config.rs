use clap::{Parser, ValueEnum};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ImageFallbackBehavior {
    /// Skip image element if fetch fails
    Skip,
    /// Return error if fetch fails
    Error,
}

#[derive(Parser)]
#[command(name = "ogis")]
#[command(version)]
#[command(about = "Open Graph Image Service - Generate OG images dynamically")]
pub struct Config {
    /// Address to bind the server to
    #[arg(short, long, default_value = "0.0.0.0:3000", env = "OGIS_ADDR")]
    pub addr: String,

    /// Maximum length for any input field
    #[arg(long, default_value = "1000", env = "OGIS_MAX_INPUT_LENGTH")]
    pub max_input_length: usize,

    /// Logo fetch connect timeout in seconds
    #[arg(long, default_value = "5", env = "OGIS_LOGO_CONNECT_TIMEOUT")]
    pub logo_connect_timeout_secs: u64,

    /// Logo fetch total timeout in seconds
    #[arg(long, default_value = "10", env = "OGIS_LOGO_TOTAL_TIMEOUT")]
    pub logo_total_timeout_secs: u64,

    /// Maximum logo image size in bytes (default: 5MB)
    #[arg(long, default_value = "5242880", env = "OGIS_LOGO_MAX_SIZE")]
    pub logo_max_size_bytes: usize,

    /// Logo cache maximum entries
    #[arg(long, default_value = "100", env = "OGIS_LOGO_CACHE_SIZE")]
    pub logo_cache_size: u64,

    /// Logo cache TTL in seconds (default: 1 hour)
    #[arg(long, default_value = "3600", env = "OGIS_LOGO_CACHE_TTL")]
    pub logo_cache_ttl_secs: u64,

    /// Maximum redirects to follow for logo URLs
    #[arg(long, default_value = "3", env = "OGIS_LOGO_MAX_REDIRECTS")]
    pub logo_max_redirects: usize,

    /// Allow HTTP (insecure) URLs for logo fetching (HTTPS only by default)
    #[arg(long, default_value = "false", env = "OGIS_ALLOW_HTTP")]
    pub allow_http: bool,

    /// Behavior when image URL fetch fails
    #[arg(long, default_value = "skip", env = "OGIS_IMAGE_FALLBACK")]
    pub image_fallback: ImageFallbackBehavior,

    /// Default title when not provided
    #[arg(long, default_value = "Open Graph Images", env = "OGIS_DEFAULT_TITLE")]
    pub default_title: String,

    /// Default description when not provided
    #[arg(long, default_value = "A fast, free, and beautiful platform for open graph image generation", env = "OGIS_DEFAULT_DESCRIPTION")]
    pub default_description: String,

    /// Default subtitle when not provided
    #[arg(long, default_value = "img.ogis.dev", env = "OGIS_DEFAULT_SUBTITLE")]
    pub default_subtitle: String,
}

impl Config {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
