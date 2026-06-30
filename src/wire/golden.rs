use crate::params::OgParams;
use crate::wire::{decode, encode};
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

fn cases() -> Vec<(&'static str, OgParams)> {
    let mut typical = blank();
    typical.title = Some("Understanding Rust Ownership".into());
    typical.description = Some("A practical guide to borrowing and lifetimes".into());
    typical.template = Some("twilight".into());

    let mut cjk = blank();
    cjk.title = Some("日本語のタイトルです".into());

    vec![("empty", blank()), ("typical", typical), ("cjk", cjk)]
}

#[test]
fn full_pipeline_roundtrips_unsigned() {
    for (name, p) in cases() {
        let (blob, sig) = encode(&p, None).unwrap();
        assert!(sig.is_none());
        let out = decode(&blob, None, None, 1000, 100_000).unwrap();
        assert_eq!(out.title, p.title, "case {name}");
        assert_eq!(out.description, p.description, "case {name}");
        assert_eq!(out.template, p.template, "case {name}");
    }
}

#[test]
fn full_pipeline_roundtrips_signed() {
    let secret = b"test-secret";
    let (_, p) = cases().into_iter().nth(1).unwrap();
    let (blob, sig) = encode(&p, Some(secret)).unwrap();
    let sig = sig.unwrap();
    assert!(decode(&blob, Some(&sig), Some(secret), 1000, 100_000).is_ok());
    // missing sig when secret set → unauthorized
    assert!(decode(&blob, None, Some(secret), 1000, 100_000).is_err());
    // tampered sig → unauthorized
    assert!(decode(&blob, Some("AAAAAAAA"), Some(secret), 1000, 100_000).is_err());
}

/// Frozen (blob → params) vectors. If this fails after a code change, the wire
/// format drifted and previously-published URLs would break. Regenerate ONLY by
/// a deliberate version bump.
#[test]
fn golden_vectors_decode_stably() {
    // (blob, expected title, expected template) pinned from a known-good build.
    let vectors: &[(&str, &str, Option<&str>)] = &[(
        "ERNUACCM1GOtsTDr04lKEGxiExpTFDb3iSBwyIHD54EneQkFO2BnOLgtxwjgXuonIIj-t8-yCXvYGinvmT6DrYITKQc",
        "Understanding Rust Ownership",
        Some("twilight"),
    )];
    for (blob, title, template) in vectors {
        let out = decode(blob, None, None, 1000, 100_000).unwrap();
        assert_eq!(
            out.title.as_deref(),
            Some(*title),
            "golden blob {blob} drifted (title)"
        );
        assert_eq!(
            out.template.as_deref(),
            *template,
            "golden blob {blob} drifted (template)"
        );
    }
}
