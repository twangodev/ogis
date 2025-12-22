use thiserror::Error;

#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error("Template '{name}' not found. Available: {available}")]
    TemplateNotFound { name: String, available: String },

    #[error("Font properties not found for template '{0}'")]
    FontPropertiesNotFound(String),

    #[error("SVG parse error: {0}")]
    SvgParse(String),

    #[error("UTF-8 encoding error: {0}")]
    Utf8(String),

    #[error("PNG encoding failed: {0}")]
    PngEncode(String),

    #[error("Failed to create pixmap")]
    PixmapCreation,

    #[error("XML processing error: {0}")]
    Xml(String),

    #[error("Text measurement error: {0}")]
    TextMeasurement(String),
}
