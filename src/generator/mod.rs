pub mod error;
mod events;
pub mod gradient_cache;
mod render;
mod render_cached;
pub mod strategies;
mod svg;
mod text_measurement;
mod utils;

pub use error::GeneratorError;
pub use gradient_cache::GradientCache;
pub use render::{OutputFormat, RenderOptions, RenderOutput, render_to_pixmap};
pub use render_cached::{GradientCacheOutcome, render_with_gradient_cache};
pub use svg::{Images, TextContent};
pub use text_measurement::FontProperties;
