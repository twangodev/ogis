pub mod error;
mod events;
mod render;
pub mod strategies;
mod svg;
mod text_measurement;
mod utils;

pub use error::GeneratorError;
pub use render::{OutputFormat, RenderOptions, RenderOutput, render};
pub use svg::{Images, TextContent, generate_svg};
pub use text_measurement::FontProperties;
