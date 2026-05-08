//! High-level entry point that applies the gradient cache where it helps.
//!
//! Gradient templates (those auto-composed from a layout + gradient YAML) are
//! split into a static gradient SVG (cached as a Pixmap) and a per-request
//! foreground SVG. Static templates fall through to the original single-pass
//! `render()` path unchanged.

use std::collections::HashMap;
use std::sync::Arc;

use super::error::GeneratorError;
use super::gradient_cache::{GradientCache, build_key};
use super::render::{RenderOptions, RenderOutput, composite, encode, render, render_to_pixmap};
use super::svg::{Images, TextContent, apply_color_overrides, generate_svg, generate_svg_from};
use crate::templates::TemplateMap;

/// Render a template, using the gradient cache when the template has a split
/// gradient/foreground definition. Falls back to the single-pass path for
/// static templates.
#[allow(clippy::too_many_arguments)]
pub fn render_with_gradient_cache(
    text: TextContent,
    images: Images,
    template_name: &str,
    templates: &TemplateMap,
    color_overrides: &HashMap<String, String>,
    fontdb: &Arc<usvg::fontdb::Database>,
    options: &RenderOptions,
    truncation: bool,
    gradient_cache: &GradientCache,
) -> Result<(RenderOutput, GradientCacheOutcome), GeneratorError> {
    let Some(split) = templates.gradient_splits.get(template_name) else {
        // Static template: existing single-pass path.
        let svg = generate_svg(
            text,
            images,
            template_name,
            templates,
            color_overrides,
            fontdb,
            truncation,
        )?;
        let output = render(&svg, fontdb, options)?;
        return Ok((output, GradientCacheOutcome::Bypassed));
    };

    // 1. Resolve the gradient SVG with any color overrides, look up or render.
    let template_colors = templates.colors.get(template_name);
    let gradient_svg_resolved = match template_colors {
        Some(tc) => apply_color_overrides(&split.gradient_svg, tc, color_overrides),
        None => split.gradient_svg.clone(),
    };
    let key = build_key(template_name, &split.gradient_color_keys, color_overrides);

    // Probe before insert so we can record hit/miss accurately.
    let already_present = gradient_cache_has(gradient_cache, &key);
    let bg_pm = gradient_cache.get_or_render(key, || {
        render_to_pixmap(&gradient_svg_resolved, fontdb, 1.0)
    })?;
    let outcome = if already_present {
        GradientCacheOutcome::Hit
    } else {
        GradientCacheOutcome::Miss
    };

    // 2. Foreground SVG: text + logo + per-request color overrides; render at
    // the requested scale onto a transparent pixmap.
    let fg_svg = generate_svg_from(
        text,
        images,
        &split.fg_svg,
        template_name,
        templates,
        color_overrides,
        fontdb,
        truncation,
    )?;
    let fg_pm = render_to_pixmap(&fg_svg, fontdb, options.scale)?;

    // 3. Composite (bilinear-resampling bg to the request scale) and encode.
    let composed = composite(&bg_pm, &fg_pm, options.scale)?;
    let output = encode(&composed, options)?;
    Ok((output, outcome))
}

/// Outcome of a gradient cache lookup, surfaced to callers for telemetry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradientCacheOutcome {
    /// Template had no gradient split; cache was not consulted.
    Bypassed,
    /// Cache lookup found an existing entry.
    Hit,
    /// Cache lookup missed and the gradient was rendered + inserted.
    Miss,
}

impl GradientCacheOutcome {
    pub fn as_label(&self) -> &'static str {
        match self {
            GradientCacheOutcome::Bypassed => "bypassed",
            GradientCacheOutcome::Hit => "hit",
            GradientCacheOutcome::Miss => "miss",
        }
    }
}

/// Helper: peek at the cache without taking ownership / inserting.
fn gradient_cache_has(cache: &GradientCache, key: &str) -> bool {
    // The cache exposes a get_or_render that uses get_with internally. We want
    // a no-side-effect probe; emulate it with a get-style check by attempting
    // a get_or_render that fails on miss. Since our wrapper doesn't expose a
    // raw `.get`, we keep the probe semantically simple: use weighted_size as
    // a sentinel is wrong, so instead expose a dedicated method here.
    cache.contains_key(key)
}
