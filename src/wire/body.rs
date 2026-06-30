use crate::generator::OutputFormat;
use crate::params::OgParams;
use std::collections::HashMap;

use super::varint::{read_bytes, read_varint, write_varint};
use super::{MAX_EXTRA, WireError};

const B_TITLE: u16 = 1 << 0;
const B_DESC: u16 = 1 << 1;
const B_SUB: u16 = 1 << 2;
const B_LOGO: u16 = 1 << 3;
const B_IMAGE: u16 = 1 << 4;
const B_TEMPLATE: u16 = 1 << 5;
const B_FORMAT: u16 = 1 << 6;
const B_SCALE: u16 = 1 << 7;
const B_QUALITY: u16 = 1 << 8;
// Bit 9 was the colors block; retired in the literal-name format. Color overrides
// now live in the extra block (B_EXTRA) as name=value entries.
const B_EXTRA: u16 = 1 << 10;
// Reserved: bit 9 (retired colors block) + bits 11..=15 (incl. the continuation flag).
const RESERVED_MASK: u16 = 0b1111_1010_0000_0000;

#[allow(dead_code)] // encoder reference; the server only decodes today
fn write_str(out: &mut Vec<u8>, s: &str) {
    write_varint(out, s.len() as u64);
    out.extend_from_slice(s.as_bytes());
}

#[allow(dead_code)] // encoder reference; the server only decodes today
fn write_url(out: &mut Vec<u8>, s: &str) {
    if let Some(rest) = s.strip_prefix("https://") {
        out.push(1);
        write_str(out, rest);
    } else if let Some(rest) = s.strip_prefix("http://") {
        out.push(2);
        write_str(out, rest);
    } else {
        out.push(0);
        write_str(out, s);
    }
}

#[allow(dead_code)] // encoder reference; the server only decodes today
pub fn pack_body(p: &OgParams) -> Result<Vec<u8>, WireError> {
    // Color overrides are literal name=value entries, wire-indistinguishable from
    // text overrides; the color-vs-text split happens at render time (extract_colors).
    let mut extras: Vec<(&str, &str)> = p
        .extra
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    if extras.len() > MAX_EXTRA {
        return Err(WireError::TooManyEntries);
    }
    extras.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

    let format_code: Option<u8> = match p.format.as_deref().and_then(OutputFormat::from_str) {
        Some(OutputFormat::Jpeg) => Some(1),
        Some(OutputFormat::WebP) => Some(2),
        _ => None, // png / unset → omit
    };
    let scale_mu: Option<u16> = match p.scale {
        Some(s) if (s - 1.0).abs() > f32::EPSILON => Some((s * 1000.0).round() as u16),
        _ => None,
    };
    let quality: Option<u8> = match p.quality {
        Some(90) | None => None,
        q => q,
    };

    let mut presence: u16 = 0;
    presence |= (p.title.is_some() as u16) * B_TITLE;
    presence |= (p.description.is_some() as u16) * B_DESC;
    presence |= (p.subtitle.is_some() as u16) * B_SUB;
    presence |= (p.logo.is_some() as u16) * B_LOGO;
    presence |= (p.image.is_some() as u16) * B_IMAGE;
    presence |= (p.template.is_some() as u16) * B_TEMPLATE;
    presence |= (format_code.is_some() as u16) * B_FORMAT;
    presence |= (scale_mu.is_some() as u16) * B_SCALE;
    presence |= (quality.is_some() as u16) * B_QUALITY;
    presence |= ((!extras.is_empty()) as u16) * B_EXTRA;

    let mut out = Vec::new();
    out.extend_from_slice(&presence.to_le_bytes());
    if let Some(name) = &p.template {
        write_str(&mut out, name);
    }
    if let Some(code) = format_code {
        out.push(code);
    }
    if let Some(mu) = scale_mu {
        out.extend_from_slice(&mu.to_le_bytes());
    }
    if let Some(q) = quality {
        out.push(q);
    }
    if let Some(t) = &p.title {
        write_str(&mut out, t);
    }
    if let Some(d) = &p.description {
        write_str(&mut out, d);
    }
    if let Some(s) = &p.subtitle {
        write_str(&mut out, s);
    }
    if let Some(l) = &p.logo {
        write_url(&mut out, l);
    }
    if let Some(i) = &p.image {
        write_url(&mut out, i);
    }
    if !extras.is_empty() {
        out.push(extras.len() as u8);
        for (k, v) in &extras {
            write_str(&mut out, k);
            write_str(&mut out, v);
        }
    }
    Ok(out)
}

fn read_u8(input: &mut &[u8]) -> Result<u8, WireError> {
    Ok(read_bytes(input, 1)?[0])
}
fn read_u16(input: &mut &[u8]) -> Result<u16, WireError> {
    let b = read_bytes(input, 2)?;
    Ok(u16::from_le_bytes([b[0], b[1]]))
}
fn read_string(input: &mut &[u8], max: usize) -> Result<String, WireError> {
    let len = read_varint(input)? as usize;
    if len > max {
        return Err(WireError::FieldTooLong);
    }
    let bytes = read_bytes(input, len)?;
    std::str::from_utf8(bytes)
        .map(str::to_string)
        .map_err(|_| WireError::BadUtf8)
}
fn read_url(input: &mut &[u8], max: usize) -> Result<String, WireError> {
    let tag = read_u8(input)?;
    let rest = read_string(input, max)?;
    Ok(match tag {
        0 => rest,
        1 => format!("https://{rest}"),
        2 => format!("http://{rest}"),
        _ => return Err(WireError::BadSchemeTag),
    })
}

pub fn unpack_body(bytes: &[u8], max_field_len: usize) -> Result<OgParams, WireError> {
    let mut input = bytes;
    let presence = read_u16(&mut input)?;
    if presence & RESERVED_MASK != 0 {
        return Err(WireError::ReservedBit);
    }

    let mut p = OgParams {
        title: None,
        description: None,
        subtitle: None,
        logo: None,
        image: None,
        template: None,
        signature: None,
        format: None,
        scale: None,
        quality: None,
        extra: HashMap::new(),
    };

    if presence & B_TEMPLATE != 0 {
        p.template = Some(read_string(&mut input, max_field_len)?);
    }
    if presence & B_FORMAT != 0 {
        p.format = Some(
            match read_u8(&mut input)? {
                1 => "jpeg",
                2 => "webp",
                _ => return Err(WireError::BadFormat),
            }
            .to_string(),
        );
    }
    if presence & B_SCALE != 0 {
        p.scale = Some(read_u16(&mut input)? as f32 / 1000.0);
    }
    if presence & B_QUALITY != 0 {
        p.quality = Some(read_u8(&mut input)?);
    }
    if presence & B_TITLE != 0 {
        p.title = Some(read_string(&mut input, max_field_len)?);
    }
    if presence & B_DESC != 0 {
        p.description = Some(read_string(&mut input, max_field_len)?);
    }
    if presence & B_SUB != 0 {
        p.subtitle = Some(read_string(&mut input, max_field_len)?);
    }
    if presence & B_LOGO != 0 {
        p.logo = Some(read_url(&mut input, max_field_len)?);
    }
    if presence & B_IMAGE != 0 {
        p.image = Some(read_url(&mut input, max_field_len)?);
    }

    if presence & B_EXTRA != 0 {
        let n = read_u8(&mut input)? as usize;
        if n > MAX_EXTRA {
            return Err(WireError::TooManyEntries);
        }
        for _ in 0..n {
            let k = read_string(&mut input, max_field_len)?;
            let v = read_string(&mut input, max_field_len)?;
            p.extra.insert(k, v);
        }
    }
    if !input.is_empty() {
        return Err(WireError::TrailingBytes);
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::OgParams;
    use std::collections::HashMap;

    fn blank() -> OgParams {
        OgParams {
            title: None,
            description: None,
            subtitle: None,
            logo: None,
            image: None,
            template: None,
            signature: None,
            format: None,
            scale: None,
            quality: None,
            extra: HashMap::new(),
        }
    }

    fn roundtrip(p: &OgParams) -> OgParams {
        let bytes = pack_body(p).unwrap();
        unpack_body(&bytes, 1000).unwrap()
    }

    #[test]
    fn empty_request_roundtrips_to_all_none() {
        let out = roundtrip(&blank());
        assert!(out.title.is_none() && out.logo.is_none() && out.extra.is_empty());
    }

    #[test]
    fn text_and_enumerables_roundtrip() {
        let mut p = blank();
        p.title = Some("Understanding Rust".into());
        p.description = Some("borrowing & lifetimes".into());
        p.subtitle = Some("".into()); // Some("") is distinct from None
        p.template = Some("twilight".into());
        p.format = Some("webp".into());
        p.scale = Some(0.75);
        p.quality = Some(85);
        let out = roundtrip(&p);
        assert_eq!(out.title.as_deref(), Some("Understanding Rust"));
        assert_eq!(out.subtitle.as_deref(), Some("")); // preserved, not None
        assert_eq!(out.template.as_deref(), Some("twilight"));
        assert_eq!(out.format.as_deref(), Some("webp"));
        assert_eq!(out.scale, Some(0.75));
        assert_eq!(out.quality, Some(85));
    }

    #[test]
    fn empty_string_distinct_from_absent() {
        let mut p = blank();
        p.title = Some("".into());
        assert_eq!(roundtrip(&p).title.as_deref(), Some(""));
        let q = blank();
        assert!(roundtrip(&q).title.is_none());
    }

    #[test]
    fn non_ascii_roundtrips() {
        let mut p = blank();
        p.title = Some("日本語 🚀".into());
        assert_eq!(roundtrip(&p).title.as_deref(), Some("日本語 🚀"));
    }

    #[test]
    fn unregistered_template_name_roundtrips_as_literal() {
        // Literal-name encoding: ANY template name round-trips, even one that is
        // in no registry (e.g. a user/tenant template). There is no ID to look up
        // and no UnknownTemplate rejection at encode time.
        let mut p = blank();
        p.template = Some("acme-promo-not-in-any-registry".into());
        assert_eq!(
            roundtrip(&p).template.as_deref(),
            Some("acme-promo-not-in-any-registry"),
        );
    }

    #[test]
    fn url_schemes_roundtrip_verbatim() {
        for url in [
            "https://cdn.x/a.png",
            "http://x/y",
            "data:image/png;base64,AAA",
            "//cdn/x",
            "",
        ] {
            let mut p = blank();
            p.logo = Some(url.into());
            assert_eq!(roundtrip(&p).logo.as_deref(), Some(url));
        }
    }

    #[test]
    fn reserved_bit_rejected() {
        // presence with bit 15 set, nothing else
        let bytes = 0x8000u16.to_le_bytes();
        assert!(matches!(
            unpack_body(&bytes, 1000),
            Err(WireError::ReservedBit)
        ));
    }

    #[test]
    fn retired_colors_bit_rejected() {
        // bit 9 was the colors block; it is retired in the literal-name format
        // and must now be decode-rejected as reserved.
        let bytes = 0x0200u16.to_le_bytes();
        assert!(matches!(
            unpack_body(&bytes, 1000),
            Err(WireError::ReservedBit)
        ));
    }

    #[test]
    fn many_extra_overrides_roundtrip() {
        // Colors fold into the extra block, so MAX_EXTRA (64) must accommodate the
        // old MAX_COLORS+MAX_EXTRA total. 40 entries (> the old 32 cap) round-trips.
        let mut p = blank();
        for i in 0..40 {
            p.extra.insert(format!("k{i:02}"), format!("v{i}"));
        }
        let out = roundtrip(&p);
        assert_eq!(out.extra.len(), 40);
        assert_eq!(out.extra.get("k07").map(String::as_str), Some("v7"));
    }

    #[test]
    fn color_and_text_overrides_roundtrip_verbatim() {
        // Color overrides and text overrides are both plain extra entries now;
        // values (including case) are preserved verbatim. The color-vs-text split
        // is a render-time concern (extract_colors), not a wire concern.
        let mut p = blank();
        p.template = Some("twilight".into());
        p.extra.insert("accent".into(), "1a2b3c".into()); // a color-looking override
        p.extra.insert("subreddit".into(), "rust".into()); // a text override
        p.extra.insert("accent_x".into(), "ABCDEF".into()); // case preserved verbatim

        let out = roundtrip(&p);
        assert_eq!(out.extra.get("accent").map(String::as_str), Some("1a2b3c"));
        assert_eq!(out.extra.get("subreddit").map(String::as_str), Some("rust"));
        assert_eq!(out.extra.get("accent_x").map(String::as_str), Some("ABCDEF"));
    }
}
