# Compressed URLs (`/c/<blob>`) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a stateless `GET /c/<blob>[/<sig>]` production URL form that encodes all OGIS request parameters into a compact, brotli-compressed, base64url blob, decoded server-side into the same `OgParams` the `?query` route uses.

**Architecture:** A new `src/wire/` module implements a versioned binary codec (schema-pack body → header byte → brotli/raw min() → base64url) plus a Rust reference encoder (for tests, golden vectors, and as the canonical SDK reference). Template/color names compress to integer IDs via committed append-only registries. HMAC signs the uncompressed body bytes (truncated to 6 bytes). The existing render path is extracted into a shared `render_response(state, params)` that both `?query` and `/c/` call.

**Tech Stack:** Rust, axum 0.8, `brotli` crate (new), `base64` 0.22 (`URL_SAFE_NO_PAD`), `hmac` 0.12 / `sha2` 0.10 (existing), `serde_json` (existing).

**Spec:** `design/compressed-urls.md` (rev. 2). Read it before starting — this plan implements it.

## Global Constraints

- **Format version:** v1 = high nibble `0x1` of the header byte. Header byte = `(version << 4) | mode`. Modes: `0`=raw, `1`=brotli, `2`=reserved (decode-reject in v1).
- **base64url, no padding** everywhere (`URL_SAFE_NO_PAD`). Load-bearing for single-path-segment routing.
- **Signature:** `HMAC-SHA256(secret, [version_byte] ++ uncompressed_body)`, truncated to the **leading 6 bytes** → 8 base64url chars. Verify with `Mac::verify_truncated_left` (constant-time).
- **Never-expand:** emit `min(raw, brotli)` container by base64url length, recorded in the mode nibble.
- **Decode order:** length-gate → base64url-decode → header → bounded-decompress → HMAC verify → parse → `params.validate()` → render. Any failure → **400** (`ApiError`), auth failure → **401**. **Never** a fallback image on bad input.
- **Caps:** `MAX_COLORS = 32`, `MAX_EXTRA = 32`. `max_field_len = state.max_input_length` (default 1000). `max_decoded` derived in the route (§8.1 of spec).
- **Registries are append-only & CI-guarded:** `src/wire/template-ids.json`, `src/wire/color-ids.json`. IDs never change/reuse; the guard test fails on mutation or a shrinking live set.
- **Text/URL fields encode `Option<String>` exactly:** clear bit ⇒ `None`; set bit + `varint(0)` ⇒ `Some("")`. Decoder leaves text/URL fields `None` for clear bits (landing-page defaults are applied downstream by `with_defaults`/`get_effective_logo`, unchanged).
- **Template:** encoder emits explicit ID whenever `params.template.is_some()` (no default-equality omission); `None` ⇒ omit. Decoder maps a registry ID with no live template to `None` (→ runtime default); an unregistered ID ⇒ 400.
- TDD, DRY, YAGNI, frequent commits. Run `cargo fmt` and `cargo clippy` before each commit.

---

## File Structure

- `src/wire/mod.rs` — module root; `WireError`, `FORMAT_VERSION`, caps, top-level `encode`/`decode`.
- `src/wire/varint.rs` — LEB128 varint + byte readers.
- `src/wire/registry.rs` — embedded template/color ID registries + lookups; the regen/guard test.
- `src/wire/template-ids.json`, `src/wire/color-ids.json` — committed registries (data).
- `src/wire/body.rs` — schema-pack `pack_body`/`unpack_body`.
- `src/wire/container.rs` — header byte, brotli compress/bounded-decompress, base64url, `encode_container`/`decode_container`.
- `src/auth/blob.rs` — `/c/` HMAC `sign`/`verify` (body-byte signing, 6-byte truncation).
- `src/routes/render.rs` — shared `render_response(State, OgParams)` extracted from `index.rs::generate`.
- `src/routes/compressed.rs` — `GET /c/{blob}` and `/c/{blob}/{sig}` handlers.
- `src/error.rs` — add `ErrorCode::InvalidCompressedUrl` + `From<WireError> for ApiError` (modify).
- `src/main.rs`, `src/routes/mod.rs`, `src/auth/mod.rs`, `Cargo.toml` — wiring (modify).
- `tests/compressed_urls.rs` — golden vectors + end-to-end.

## Shared Interfaces (defined once; tasks reference these exact signatures)

```rust
// src/wire/mod.rs
pub const FORMAT_VERSION: u8 = 1;       // header high nibble
pub const MAX_COLORS: usize = 32;
pub const MAX_EXTRA: usize = 32;
pub enum WireError { /* Task 1 */ }

pub fn encode(p: &OgParams, reg: &Registry, templates: &TemplateMap, secret: Option<&[u8]>)
    -> Result<(String /*blob*/, Option<String> /*sig*/), WireError>;
pub fn decode(blob: &str, sig: Option<&str>, reg: &Registry, secret: Option<&[u8]>,
              max_field_len: usize, max_decoded: usize) -> Result<OgParams, WireError>;

// src/wire/varint.rs
pub fn write_varint(out: &mut Vec<u8>, v: u64);
pub fn read_varint(input: &mut &[u8]) -> Result<u64, WireError>;
pub fn read_bytes<'a>(input: &mut &'a [u8], n: usize) -> Result<&'a [u8], WireError>;

// src/wire/registry.rs
pub struct Registry { /* maps */ }
impl Registry {
    pub fn load() -> &'static Registry;                 // lazy, from include_str!
    pub fn template_id(&self, name: &str) -> Option<u16>;
    pub fn template_name(&self, id: u16) -> Option<&str>;
    pub fn color_id(&self, name: &str) -> Option<u16>;
    pub fn color_name(&self, id: u16) -> Option<&str>;
}

// src/wire/body.rs
pub fn pack_body(p: &OgParams, reg: &Registry, templates: &TemplateMap) -> Result<Vec<u8>, WireError>;
pub fn unpack_body(bytes: &[u8], reg: &Registry, max_field_len: usize) -> Result<OgParams, WireError>;

// src/wire/container.rs
pub fn encode_container(body: &[u8]) -> String;
pub fn decode_container(blob: &str, max_decoded: usize) -> Result<(u8 /*version*/, Vec<u8> /*body*/), WireError>;

// src/auth/blob.rs
pub const SIG_LEN: usize = 6;
pub fn sign(secret: &[u8], version: u8, body: &[u8]) -> String;            // 8-char base64url
pub fn verify(secret: &[u8], version: u8, body: &[u8], seg: &str) -> Result<(), WireError>;

// src/routes/render.rs
pub async fn render_response(state: AppState, params: OgParams) -> axum::response::Response;
```

---

## Task 1: Module skeleton, `WireError`, error mapping

**Files:**
- Modify: `Cargo.toml` (add `brotli`)
- Create: `src/wire/mod.rs`
- Modify: `src/main.rs:1-12` (add `mod wire;`)
- Modify: `src/error.rs` (add `ErrorCode::InvalidCompressedUrl`, `From<WireError>`)

**Interfaces:**
- Produces: `WireError`, `FORMAT_VERSION`, `MAX_COLORS`, `MAX_EXTRA`, `From<WireError> for ApiError`.

- [ ] **Step 1: Add the brotli dependency**

In `Cargo.toml` under `[dependencies]` add:
```toml
brotli = "7"
```

- [ ] **Step 2: Create the wire module with WireError**

Create `src/wire/mod.rs`:
```rust
//! Compressed-URL (`/c/<blob>`) wire codec. See design/compressed-urls.md.
pub mod body;
pub mod container;
pub mod registry;
pub mod varint;

use crate::params::OgParams;
use crate::templates::TemplateMap;
use registry::Registry;

pub const FORMAT_VERSION: u8 = 1;
pub const MAX_COLORS: usize = 32;
pub const MAX_EXTRA: usize = 32;

#[derive(Debug, thiserror::Error)]
pub enum WireError {
    #[error("blob truncated")]
    Truncated,
    #[error("trailing bytes after body")]
    TrailingBytes,
    #[error("malformed varint")]
    Malformed,
    #[error("invalid base64url")]
    BadBase64,
    #[error("unsupported format version {0}")]
    BadVersion(u8),
    #[error("unsupported compression mode {0}")]
    BadMode(u8),
    #[error("reserved presence bit set")]
    ReservedBit,
    #[error("invalid format code")]
    BadFormat,
    #[error("invalid scheme tag")]
    BadSchemeTag,
    #[error("invalid UTF-8")]
    BadUtf8,
    #[error("field exceeds maximum length")]
    FieldTooLong,
    #[error("too many entries")]
    TooManyEntries,
    #[error("unknown template id")]
    UnknownTemplate,
    #[error("unknown color id")]
    UnknownColor,
    #[error("decompressed body too large")]
    TooLarge,
    #[error("brotli error")]
    BadBrotli,
    #[error("unauthorized")]
    Unauthorized,
}

/// Encode `params` to `(blob, optional sig)`. `secret` present ⇒ sign.
pub fn encode(
    p: &OgParams,
    reg: &Registry,
    templates: &TemplateMap,
    secret: Option<&[u8]>,
) -> Result<(String, Option<String>), WireError> {
    let body = body::pack_body(p, reg, templates)?;
    let sig = secret.map(|s| crate::auth::blob::sign(s, FORMAT_VERSION, &body));
    Ok((container::encode_container(&body), sig))
}

/// Decode a `/c/` blob (+ optional sig) into `OgParams`.
pub fn decode(
    blob: &str,
    sig: Option<&str>,
    reg: &Registry,
    secret: Option<&[u8]>,
    max_field_len: usize,
    max_decoded: usize,
) -> Result<OgParams, WireError> {
    let (version, body) = container::decode_container(blob, max_decoded)?;
    match (secret, sig) {
        (Some(s), Some(seg)) => crate::auth::blob::verify(s, version, &body, seg)?,
        (Some(_), None) => return Err(WireError::Unauthorized),
        (None, _) => {} // auth disabled
    }
    body::unpack_body(&body, reg, max_field_len)
}
```

- [ ] **Step 3: Register the module**

In `src/main.rs`, add `mod wire;` to the module list (keep alphabetical, after `mod telemetry;` / `mod templates;`).

- [ ] **Step 4: Add error code + conversion**

In `src/error.rs`, add to `enum ErrorCode` under the "Validation errors (400)" group:
```rust
    InvalidCompressedUrl,
```
And add, after the existing `From` conversions:
```rust
impl From<crate::wire::WireError> for ApiError {
    fn from(e: crate::wire::WireError) -> Self {
        use crate::wire::WireError;
        match e {
            WireError::Unauthorized => ApiError::new(
                StatusCode::UNAUTHORIZED,
                ErrorCode::AuthMissingSignature,
                "Authentication required or invalid signature",
            ),
            other => ApiError::new(
                StatusCode::BAD_REQUEST,
                ErrorCode::InvalidCompressedUrl,
                other.to_string(),
            ),
        }
    }
}
```

- [ ] **Step 5: Stub the submodules so it compiles**

Create empty-but-valid stubs (filled by later tasks):
```rust
// src/wire/varint.rs
use super::WireError;
pub fn write_varint(_out: &mut Vec<u8>, _v: u64) { todo!() }
pub fn read_varint(_input: &mut &[u8]) -> Result<u64, WireError> { todo!() }
pub fn read_bytes<'a>(_input: &mut &'a [u8], _n: usize) -> Result<&'a [u8], WireError> { todo!() }
```
```rust
// src/wire/registry.rs
pub struct Registry;
impl Registry {
    pub fn load() -> &'static Registry { todo!() }
    pub fn template_id(&self, _n: &str) -> Option<u16> { todo!() }
    pub fn template_name(&self, _id: u16) -> Option<&str> { todo!() }
    pub fn color_id(&self, _n: &str) -> Option<u16> { todo!() }
    pub fn color_name(&self, _id: u16) -> Option<&str> { todo!() }
}
```
```rust
// src/wire/body.rs
use super::{registry::Registry, WireError};
use crate::{params::OgParams, templates::TemplateMap};
pub fn pack_body(_p: &OgParams, _reg: &Registry, _t: &TemplateMap) -> Result<Vec<u8>, WireError> { todo!() }
pub fn unpack_body(_b: &[u8], _reg: &Registry, _max: usize) -> Result<OgParams, WireError> { todo!() }
```
```rust
// src/wire/container.rs
use super::WireError;
pub fn encode_container(_body: &[u8]) -> String { todo!() }
pub fn decode_container(_blob: &str, _max: usize) -> Result<(u8, Vec<u8>), WireError> { todo!() }
```
Also stub `src/auth/blob.rs` and add `pub mod blob;` to `src/auth/mod.rs`:
```rust
// src/auth/blob.rs
use crate::wire::WireError;
pub const SIG_LEN: usize = 6;
pub fn sign(_secret: &[u8], _version: u8, _body: &[u8]) -> String { todo!() }
pub fn verify(_secret: &[u8], _version: u8, _body: &[u8], _seg: &str) -> Result<(), WireError> { todo!() }
```

- [ ] **Step 6: Verify it compiles**

Run: `cargo build`
Expected: builds (warnings about `todo!()`/unused are fine).

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock src/wire src/auth/blob.rs src/auth/mod.rs src/main.rs src/error.rs
git commit -m "feat(wire): scaffold compressed-URL codec module and error mapping"
```

---

## Task 2: LEB128 varint + byte readers

**Files:**
- Modify: `src/wire/varint.rs`

**Interfaces:**
- Produces: `write_varint`, `read_varint`, `read_bytes` (signatures above).

- [ ] **Step 1: Write failing tests**

Replace `src/wire/varint.rs` test section — add at the bottom:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_boundary_values() {
        for v in [0u64, 1, 127, 128, 16_383, 16_384, 1_000, u32::MAX as u64] {
            let mut buf = Vec::new();
            write_varint(&mut buf, v);
            let mut slice = &buf[..];
            assert_eq!(read_varint(&mut slice).unwrap(), v);
            assert!(slice.is_empty(), "consumed all bytes for {v}");
        }
    }

    #[test]
    fn small_values_are_one_byte() {
        let mut buf = Vec::new();
        write_varint(&mut buf, 127);
        assert_eq!(buf.len(), 1);
    }

    #[test]
    fn truncated_varint_errors() {
        let mut slice = &[0x80u8][..]; // continuation bit set, no follow-up
        assert!(matches!(read_varint(&mut slice), Err(WireError::Truncated)));
    }

    #[test]
    fn read_bytes_advances_and_bounds() {
        let data = [1u8, 2, 3, 4];
        let mut slice = &data[..];
        assert_eq!(read_bytes(&mut slice, 2).unwrap(), &[1, 2]);
        assert_eq!(slice, &[3, 4]);
        assert!(matches!(read_bytes(&mut slice, 3), Err(WireError::Truncated)));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib wire::varint`
Expected: FAIL (`todo!()` panics).

- [ ] **Step 3: Implement**

Replace the non-test contents of `src/wire/varint.rs`:
```rust
use super::WireError;

/// Append `v` as an unsigned LEB128 varint.
pub fn write_varint(out: &mut Vec<u8>, mut v: u64) {
    loop {
        let mut byte = (v & 0x7f) as u8;
        v >>= 7;
        if v != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if v == 0 {
            break;
        }
    }
}

/// Read an unsigned LEB128 varint from the front of `input`, advancing it.
pub fn read_varint(input: &mut &[u8]) -> Result<u64, WireError> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    loop {
        let byte = *input.first().ok_or(WireError::Truncated)?;
        *input = &input[1..];
        result |= ((byte & 0x7f) as u64) << shift;
        if byte & 0x80 == 0 {
            return Ok(result);
        }
        shift += 7;
        if shift >= 64 {
            return Err(WireError::Malformed);
        }
    }
}

/// Split `n` bytes off the front of `input`, advancing it.
pub fn read_bytes<'a>(input: &mut &'a [u8], n: usize) -> Result<&'a [u8], WireError> {
    if input.len() < n {
        return Err(WireError::Truncated);
    }
    let (head, tail) = input.split_at(n);
    *input = tail;
    Ok(head)
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib wire::varint`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt && git add src/wire/varint.rs && git commit -m "feat(wire): LEB128 varint and bounded byte readers"
```

---

## Task 3: Template/color ID registries + guard test

**Files:**
- Modify: `src/wire/registry.rs`
- Create: `src/wire/template-ids.json` (initially `{}`)
- Create: `src/wire/color-ids.json` (initially `{}`)

**Interfaces:**
- Consumes: `templates::load_templates()` (existing), `TemplateMap.templates` keys, `TemplateMap.colors` per-template keys.
- Produces: `Registry::{load, template_id, template_name, color_id, color_name}`.

**Design:** registries are `name → u16` maps embedded via `include_str!`. A test regenerates them (append-only) when `OGIS_REGEN_WIRE_IDS=1`, and otherwise guards that every live template/color has a stable committed ID and the committed set has not shrunk relative to runtime.

- [ ] **Step 1: Create empty registry data files**

`src/wire/template-ids.json`:
```json
{}
```
`src/wire/color-ids.json`:
```json
{}
```

- [ ] **Step 2: Implement the Registry**

Replace `src/wire/registry.rs`:
```rust
use std::collections::HashMap;
use std::sync::OnceLock;

/// Append-only `name → id` maps for templates and color names, embedded at build time.
pub struct Registry {
    template_to_id: HashMap<String, u16>,
    id_to_template: HashMap<u16, String>,
    color_to_id: HashMap<String, u16>,
    id_to_color: HashMap<u16, String>,
}

const TEMPLATE_IDS_JSON: &str = include_str!("template-ids.json");
const COLOR_IDS_JSON: &str = include_str!("color-ids.json");

fn invert(map: &HashMap<String, u16>) -> HashMap<u16, String> {
    map.iter().map(|(k, v)| (*v, k.clone())).collect()
}

impl Registry {
    pub fn load() -> &'static Registry {
        static REG: OnceLock<Registry> = OnceLock::new();
        REG.get_or_init(|| {
            let template_to_id: HashMap<String, u16> =
                serde_json::from_str(TEMPLATE_IDS_JSON).expect("template-ids.json");
            let color_to_id: HashMap<String, u16> =
                serde_json::from_str(COLOR_IDS_JSON).expect("color-ids.json");
            Registry {
                id_to_template: invert(&template_to_id),
                id_to_color: invert(&color_to_id),
                template_to_id,
                color_to_id,
            }
        })
    }

    pub fn template_id(&self, name: &str) -> Option<u16> {
        self.template_to_id.get(name).copied()
    }
    pub fn template_name(&self, id: u16) -> Option<&str> {
        self.id_to_template.get(&id).map(String::as_str)
    }
    pub fn color_id(&self, name: &str) -> Option<u16> {
        self.color_to_id.get(name).copied()
    }
    pub fn color_name(&self, id: u16) -> Option<&str> {
        self.id_to_color.get(&id).map(String::as_str)
    }
}
```

- [ ] **Step 3: Write the regen/guard test**

Add to `src/wire/registry.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    /// Live names from the loaded templates: template names + union of all per-template color names.
    fn live_names() -> (Vec<String>, Vec<String>) {
        let map = crate::templates::load_templates();
        let mut templates: Vec<String> = map.templates.keys().cloned().collect();
        templates.sort();
        let mut colors: std::collections::BTreeSet<String> = Default::default();
        for palette in map.colors.values() {
            for key in palette.keys() {
                colors.insert(key.clone());
            }
        }
        (templates, colors.into_iter().collect())
    }

    /// Append missing names to `committed` with the next free id (append-only).
    fn extend(committed: &mut BTreeMap<String, u16>, names: &[String]) {
        let mut next = committed.values().copied().max().map(|m| m + 1).unwrap_or(0);
        for n in names {
            if !committed.contains_key(n) {
                committed.insert(n.clone(), next);
                next += 1;
            }
        }
    }

    #[test]
    fn registries_cover_all_live_names() {
        let (templates, colors) = live_names();
        let reg = Registry::load();

        if std::env::var("OGIS_REGEN_WIRE_IDS").is_ok() {
            let mut t: BTreeMap<String, u16> =
                serde_json::from_str(TEMPLATE_IDS_JSON).unwrap();
            let mut c: BTreeMap<String, u16> =
                serde_json::from_str(COLOR_IDS_JSON).unwrap();
            extend(&mut t, &templates);
            extend(&mut c, &colors);
            std::fs::write(
                concat!(env!("CARGO_MANIFEST_DIR"), "/src/wire/template-ids.json"),
                serde_json::to_string_pretty(&t).unwrap(),
            )
            .unwrap();
            std::fs::write(
                concat!(env!("CARGO_MANIFEST_DIR"), "/src/wire/color-ids.json"),
                serde_json::to_string_pretty(&c).unwrap(),
            )
            .unwrap();
            return; // regenerated; rerun without the env var to assert
        }

        let missing_t: Vec<_> = templates.iter().filter(|n| reg.template_id(n).is_none()).collect();
        let missing_c: Vec<_> = colors.iter().filter(|n| reg.color_id(n).is_none()).collect();
        assert!(
            missing_t.is_empty() && missing_c.is_empty(),
            "registry missing live names — run `OGIS_REGEN_WIRE_IDS=1 cargo test registries_cover_all_live_names`.\n  templates: {missing_t:?}\n  colors: {missing_c:?}"
        );
    }
}
```

- [ ] **Step 4: Populate the registries (one-time regen)**

Run: `OGIS_REGEN_WIRE_IDS=1 cargo test --lib wire::registry::tests::registries_cover_all_live_names`
Expected: PASS; `git diff --stat` shows `template-ids.json` (~872 entries) and `color-ids.json` populated.

- [ ] **Step 5: Verify the guard passes without the env var**

Run: `cargo test --lib wire::registry`
Expected: PASS (every live name has an ID).

- [ ] **Step 6: Commit**

```bash
cargo fmt && git add src/wire/registry.rs src/wire/template-ids.json src/wire/color-ids.json
git commit -m "feat(wire): append-only template/color ID registries with CI guard"
```

---

## Task 4: Schema-pack body (`pack_body` / `unpack_body`)

**Files:**
- Modify: `src/wire/body.rs`

**Interfaces:**
- Consumes: `write_varint`/`read_varint`/`read_bytes` (Task 2), `Registry` (Task 3), `OgParams` (`src/params.rs`: pub fields `title, description, subtitle, logo, image, template, signature, format, scale, quality, extra`), `TemplateMap.{default, colors}`, `generator::OutputFormat` (`from_str`).
- Produces: `pack_body`, `unpack_body` (round-trip inverse).

- [ ] **Step 1: Write the failing round-trip test**

Add to `src/wire/body.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::OgParams;
    use crate::templates::load_templates;
    use crate::wire::registry::Registry;
    use std::collections::HashMap;

    fn blank() -> OgParams {
        OgParams {
            title: None, description: None, subtitle: None, logo: None, image: None,
            template: None, signature: None, format: None, scale: None, quality: None,
            extra: HashMap::new(),
        }
    }

    fn roundtrip(p: &OgParams) -> OgParams {
        let reg = Registry::load();
        let t = load_templates();
        let bytes = pack_body(p, reg, &t).unwrap();
        unpack_body(&bytes, reg, 1000).unwrap()
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
    fn url_schemes_roundtrip_verbatim() {
        for url in ["https://cdn.x/a.png", "http://x/y", "data:image/png;base64,AAA", "//cdn/x", ""] {
            let mut p = blank();
            p.logo = Some(url.into());
            assert_eq!(roundtrip(&p).logo.as_deref(), Some(url));
        }
    }

    #[test]
    fn reserved_bit_rejected() {
        let reg = Registry::load();
        // presence with bit 15 set, nothing else
        let bytes = 0x8000u16.to_le_bytes();
        assert!(matches!(unpack_body(&bytes, reg, 1000), Err(WireError::ReservedBit)));
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test --lib wire::body`
Expected: FAIL (`todo!()`).

- [ ] **Step 3: Implement `pack_body`/`unpack_body`**

Replace the non-test contents of `src/wire/body.rs`:
```rust
use crate::generator::OutputFormat;
use crate::params::OgParams;
use crate::templates::TemplateMap;
use std::collections::HashMap;

use super::registry::Registry;
use super::varint::{read_bytes, read_varint, write_varint};
use super::{WireError, MAX_COLORS, MAX_EXTRA};

const B_TITLE: u16 = 1 << 0;
const B_DESC: u16 = 1 << 1;
const B_SUB: u16 = 1 << 2;
const B_LOGO: u16 = 1 << 3;
const B_IMAGE: u16 = 1 << 4;
const B_TEMPLATE: u16 = 1 << 5;
const B_FORMAT: u16 = 1 << 6;
const B_SCALE: u16 = 1 << 7;
const B_QUALITY: u16 = 1 << 8;
const B_COLORS: u16 = 1 << 9;
const B_EXTRA: u16 = 1 << 10;
const RESERVED_MASK: u16 = 0b1111_1000_0000_0000; // bits 11..=15 (incl. continuation)

fn is_six_lower_hex(s: &str) -> bool {
    s.len() == 6 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn hex6_to_rgb(s: &str) -> [u8; 3] {
    let n = u32::from_str_radix(s, 16).unwrap(); // validated by is_six_lower_hex
    [(n >> 16) as u8, (n >> 8) as u8, n as u8]
}

fn rgb_to_hex6(rgb: &[u8; 3]) -> String {
    format!("{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2])
}

fn write_str(out: &mut Vec<u8>, s: &str) {
    write_varint(out, s.len() as u64);
    out.extend_from_slice(s.as_bytes());
}

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

pub fn pack_body(p: &OgParams, reg: &Registry, templates: &TemplateMap) -> Result<Vec<u8>, WireError> {
    let resolved = p.template.as_deref().unwrap_or(&templates.default);
    let template_colors = templates.colors.get(resolved);

    // Partition extra into a typed colors block (only template colors with exactly
    // 6-lowercase-hex values) and a verbatim extra block — matching extract_colors().
    let mut colors: Vec<(u16, [u8; 3])> = Vec::new();
    let mut extras: Vec<(&str, &str)> = Vec::new();
    for (k, v) in &p.extra {
        let packable = template_colors.is_some_and(|c| c.contains_key(k)) && is_six_lower_hex(v);
        match packable.then(|| reg.color_id(k)).flatten() {
            Some(id) => colors.push((id, hex6_to_rgb(v))),
            None => extras.push((k.as_str(), v.as_str())),
        }
    }
    if colors.len() > MAX_COLORS || extras.len() > MAX_EXTRA {
        return Err(WireError::TooManyEntries);
    }
    colors.sort_by_key(|(id, _)| *id);
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
    presence |= ((!colors.is_empty()) as u16) * B_COLORS;
    presence |= ((!extras.is_empty()) as u16) * B_EXTRA;

    let mut out = Vec::new();
    out.extend_from_slice(&presence.to_le_bytes());
    if let Some(name) = &p.template {
        let id = reg.template_id(name).ok_or(WireError::UnknownTemplate)?;
        out.extend_from_slice(&id.to_le_bytes());
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
    if let Some(t) = &p.title { write_str(&mut out, t); }
    if let Some(d) = &p.description { write_str(&mut out, d); }
    if let Some(s) = &p.subtitle { write_str(&mut out, s); }
    if let Some(l) = &p.logo { write_url(&mut out, l); }
    if let Some(i) = &p.image { write_url(&mut out, i); }
    if !colors.is_empty() {
        out.push(colors.len() as u8);
        for (id, rgb) in &colors {
            out.extend_from_slice(&id.to_le_bytes());
            out.extend_from_slice(rgb);
        }
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
    std::str::from_utf8(bytes).map(str::to_string).map_err(|_| WireError::BadUtf8)
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

pub fn unpack_body(bytes: &[u8], reg: &Registry, max_field_len: usize) -> Result<OgParams, WireError> {
    let mut input = bytes;
    let presence = read_u16(&mut input)?;
    if presence & RESERVED_MASK != 0 {
        return Err(WireError::ReservedBit);
    }

    let mut p = OgParams {
        title: None, description: None, subtitle: None, logo: None, image: None,
        template: None, signature: None, format: None, scale: None, quality: None,
        extra: HashMap::new(),
    };

    if presence & B_TEMPLATE != 0 {
        let id = read_u16(&mut input)?;
        let name = reg.template_name(id).ok_or(WireError::UnknownTemplate)?;
        p.template = Some(name.to_string());
    }
    if presence & B_FORMAT != 0 {
        p.format = Some(match read_u8(&mut input)? {
            1 => "jpeg",
            2 => "webp",
            _ => return Err(WireError::BadFormat),
        }
        .to_string());
    }
    if presence & B_SCALE != 0 {
        p.scale = Some(read_u16(&mut input)? as f32 / 1000.0);
    }
    if presence & B_QUALITY != 0 {
        p.quality = Some(read_u8(&mut input)?);
    }
    if presence & B_TITLE != 0 { p.title = Some(read_string(&mut input, max_field_len)?); }
    if presence & B_DESC != 0 { p.description = Some(read_string(&mut input, max_field_len)?); }
    if presence & B_SUB != 0 { p.subtitle = Some(read_string(&mut input, max_field_len)?); }
    if presence & B_LOGO != 0 { p.logo = Some(read_url(&mut input, max_field_len)?); }
    if presence & B_IMAGE != 0 { p.image = Some(read_url(&mut input, max_field_len)?); }

    if presence & B_COLORS != 0 {
        let n = read_u8(&mut input)? as usize;
        if n > MAX_COLORS {
            return Err(WireError::TooManyEntries);
        }
        for _ in 0..n {
            let id = read_u16(&mut input)?;
            let rgb: [u8; 3] = read_bytes(&mut input, 3)?.try_into().unwrap();
            let name = reg.color_name(id).ok_or(WireError::UnknownColor)?;
            p.extra.insert(name.to_string(), rgb_to_hex6(&rgb));
        }
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
```

- [ ] **Step 4: Add the colors/extra split test**

Append to the `tests` module in `src/wire/body.rs`:
```rust
    #[test]
    fn template_color_packs_other_extra_verbatim() {
        // Pick a real template + one of its color names from the loaded set.
        let t = load_templates();
        let (tpl, color_key) = t
            .colors
            .iter()
            .find_map(|(name, palette)| palette.keys().next().map(|k| (name.clone(), k.clone())))
            .expect("a template with at least one color");

        let mut p = blank();
        p.template = Some(tpl);
        p.extra.insert(color_key.clone(), "1a2b3c".into()); // packs into colors block
        p.extra.insert("subreddit".into(), "rust".into()); // verbatim text override
        p.extra.insert(color_key.clone() + "_x", "ABCDEF".into()); // unknown key → verbatim, case kept

        let out = roundtrip(&p);
        assert_eq!(out.extra.get(&color_key).map(String::as_str), Some("1a2b3c"));
        assert_eq!(out.extra.get("subreddit").map(String::as_str), Some("rust"));
        assert_eq!(out.extra.get(&(color_key + "_x")).map(String::as_str), Some("ABCDEF"));
    }
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --lib wire::body`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
cargo fmt && cargo clippy --lib && git add src/wire/body.rs
git commit -m "feat(wire): schema-pack body encode/decode with per-template color split"
```

---

## Task 5: Container — header, brotli min(), bounded decompress, base64url

**Files:**
- Modify: `src/wire/container.rs`

**Interfaces:**
- Consumes: `FORMAT_VERSION`, `WireError`.
- Produces: `encode_container`, `decode_container`.

- [ ] **Step 1: Write failing tests**

Add to `src/wire/container.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_small_and_large_bodies() {
        for body in [vec![], b"hi".to_vec(), vec![7u8; 500], (0..255).collect::<Vec<u8>>()] {
            let blob = encode_container(&body);
            let (version, out) = decode_container(&blob, 100_000).unwrap();
            assert_eq!(version, FORMAT_VERSION);
            assert_eq!(out, body);
        }
    }

    #[test]
    fn picks_smaller_encoding() {
        // Highly compressible → brotli should win and decode back.
        let body = vec![0u8; 2000];
        let blob = encode_container(&body);
        assert!(blob.len() < 2000 * 4 / 3);
        assert_eq!(decode_container(&blob, 100_000).unwrap().1, body);
    }

    #[test]
    fn rejects_oversized_decompressed_body() {
        let body = vec![0u8; 5000]; // compresses tiny, inflates past the cap
        let blob = encode_container(&body);
        assert!(matches!(decode_container(&blob, 1000), Err(WireError::TooLarge)));
    }

    #[test]
    fn rejects_bad_version_and_mode() {
        // header 0x2X = version 2 (unsupported)
        let bytes = [0x20u8, 0, 0];
        let blob = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, bytes);
        assert!(matches!(decode_container(&blob, 1000), Err(WireError::BadVersion(2))));
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test --lib wire::container`
Expected: FAIL (`todo!()`).

- [ ] **Step 3: Implement**

Replace the non-test contents of `src/wire/container.rs`:
```rust
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use std::io::Read;

use super::{WireError, FORMAT_VERSION};

const MODE_RAW: u8 = 0;
const MODE_BROTLI: u8 = 1;

fn container_bytes(mode: u8, payload: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(payload.len() + 1);
    v.push((FORMAT_VERSION << 4) | mode);
    v.extend_from_slice(payload);
    v
}

fn brotli_compress(input: &[u8]) -> Vec<u8> {
    let mut params = brotli::enc::BrotliEncoderParams::default();
    params.quality = 11;
    let mut out = Vec::new();
    let mut reader = input;
    brotli::BrotliCompress(&mut reader, &mut out, &params).expect("brotli compress is infallible into a Vec");
    out
}

/// Encode `body` as the smaller of {raw, brotli} containers, base64url-nopad.
pub fn encode_container(body: &[u8]) -> String {
    let raw = URL_SAFE_NO_PAD.encode(container_bytes(MODE_RAW, body));
    let bro = URL_SAFE_NO_PAD.encode(container_bytes(MODE_BROTLI, &brotli_compress(body)));
    if bro.len() < raw.len() {
        bro
    } else {
        raw
    }
}

/// Decode a blob to `(version, uncompressed body)`, bounding the decompressed size.
pub fn decode_container(blob: &str, max_decoded: usize) -> Result<(u8, Vec<u8>), WireError> {
    let bytes = URL_SAFE_NO_PAD.decode(blob.as_bytes()).map_err(|_| WireError::BadBase64)?;
    let (&header, payload) = bytes.split_first().ok_or(WireError::Truncated)?;
    let version = header >> 4;
    let mode = header & 0x0f;
    if version != FORMAT_VERSION {
        return Err(WireError::BadVersion(version));
    }
    let body = match mode {
        MODE_RAW => {
            if payload.len() > max_decoded {
                return Err(WireError::TooLarge);
            }
            payload.to_vec()
        }
        MODE_BROTLI => decompress_bounded(payload, max_decoded)?,
        other => return Err(WireError::BadMode(other)),
    };
    Ok((version, body))
}

fn decompress_bounded(input: &[u8], max: usize) -> Result<Vec<u8>, WireError> {
    let mut decompressor = brotli::Decompressor::new(input, 4096);
    let mut out = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        let n = decompressor.read(&mut buf).map_err(|_| WireError::BadBrotli)?;
        if n == 0 {
            break;
        }
        if out.len() + n > max {
            return Err(WireError::TooLarge);
        }
        out.extend_from_slice(&buf[..n]);
    }
    Ok(out)
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib wire::container`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy --lib && git add src/wire/container.rs
git commit -m "feat(wire): versioned container with brotli min() and bounded decompress"
```

---

## Task 6: Blob HMAC signing (`auth::blob`)

**Files:**
- Modify: `src/auth/blob.rs`

**Interfaces:**
- Consumes: `hmac::Hmac`, `sha2::Sha256`, `base64` (existing deps), `WireError`.
- Produces: `SIG_LEN`, `sign`, `verify`.

- [ ] **Step 1: Write failing tests**

Add to `src/auth/blob.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_then_verify_ok() {
        let body = b"hello body";
        let seg = sign(b"secret", 1, body);
        assert_eq!(seg.len(), 8); // 6 bytes → 8 base64url chars
        assert!(verify(b"secret", 1, body, &seg).is_ok());
    }

    #[test]
    fn wrong_secret_or_body_rejected() {
        let seg = sign(b"secret", 1, b"body");
        assert!(verify(b"other", 1, b"body", &seg).is_err());
        assert!(verify(b"secret", 1, b"BODY", &seg).is_err());
        assert!(verify(b"secret", 2, b"body", &seg).is_err()); // version is signed
    }

    #[test]
    fn malformed_segment_rejected() {
        assert!(matches!(verify(b"s", 1, b"b", "!!!"), Err(WireError::Unauthorized)));
        assert!(matches!(verify(b"s", 1, b"b", "AAAA"), Err(WireError::Unauthorized))); // 3 bytes ≠ 6
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test --lib auth::blob`
Expected: FAIL (`todo!()`).

- [ ] **Step 3: Implement**

Replace the non-test contents of `src/auth/blob.rs`:
```rust
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::wire::WireError;

type HmacSha256 = Hmac<Sha256>;

/// Truncated tag length in bytes (→ 8 base64url chars). Pinned by the format version.
pub const SIG_LEN: usize = 6;

fn mac(secret: &[u8], version: u8, body: &[u8]) -> HmacSha256 {
    let mut m = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key size");
    m.update(&[version]);
    m.update(body);
    m
}

/// Sign `[version] ++ body`, returning the 8-char base64url segment.
pub fn sign(secret: &[u8], version: u8, body: &[u8]) -> String {
    let tag = mac(secret, version, body).finalize().into_bytes();
    URL_SAFE_NO_PAD.encode(&tag[..SIG_LEN])
}

/// Verify a `/c/` path-segment signature in constant time.
pub fn verify(secret: &[u8], version: u8, body: &[u8], seg: &str) -> Result<(), WireError> {
    let provided = URL_SAFE_NO_PAD.decode(seg.as_bytes()).map_err(|_| WireError::Unauthorized)?;
    if provided.len() != SIG_LEN {
        return Err(WireError::Unauthorized);
    }
    mac(secret, version, body)
        .verify_truncated_left(&provided)
        .map_err(|_| WireError::Unauthorized)
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib auth::blob`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt && git add src/auth/blob.rs && git commit -m "feat(auth): body-byte HMAC signing for /c/ with 6-byte truncation"
```

---

## Task 7: Top-level codec round-trip + committed golden vectors

**Files:**
- Create: `tests/compressed_urls.rs`
- Create: `tests/fixtures/c-golden.json` (generated in Step 4)

**Interfaces:**
- Consumes: `wire::{encode, decode}`, `wire::registry::Registry`, `templates::load_templates`.

> `wire::encode`/`decode`, `Registry`, and `load_templates` must be reachable from an integration test. Ensure `pub mod wire;` (Task 1) and that `templates::load_templates` is `pub` (it is). If `Registry`/`encode` are not yet `pub` at crate root, expose them: in `src/wire/mod.rs` they are already `pub`. Integration tests see the crate as `ogis::…` only if it is a lib; this repo is a binary, so put these as **`#[test]` functions inside the crate** instead: create `src/wire/tests_roundtrip.rs` and `mod tests_roundtrip;` under `#[cfg(test)]` in `src/wire/mod.rs`. (Adjust paths below accordingly.)

- [ ] **Step 1: Write the failing round-trip + golden test**

Create `src/wire/golden.rs` and add `#[cfg(test)] mod golden;` to `src/wire/mod.rs`:
```rust
use crate::params::OgParams;
use crate::templates::load_templates;
use crate::wire::{decode, encode, registry::Registry};
use std::collections::HashMap;

fn blank() -> OgParams {
    OgParams {
        title: None, description: None, subtitle: None, logo: None, image: None,
        template: None, signature: None, format: None, scale: None, quality: None,
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
    let reg = Registry::load();
    let t = load_templates();
    for (name, p) in cases() {
        let (blob, sig) = encode(&p, reg, &t, None).unwrap();
        assert!(sig.is_none());
        let out = decode(&blob, None, reg, None, 1000, 100_000).unwrap();
        assert_eq!(out.title, p.title, "case {name}");
        assert_eq!(out.template, p.template, "case {name}");
    }
}

#[test]
fn full_pipeline_roundtrips_signed() {
    let reg = Registry::load();
    let t = load_templates();
    let secret = b"test-secret";
    let (_, p) = cases().into_iter().nth(1).unwrap();
    let (blob, sig) = encode(&p, reg, &t, Some(secret)).unwrap();
    let sig = sig.unwrap();
    assert!(decode(&blob, Some(&sig), reg, Some(secret), 1000, 100_000).is_ok());
    // missing sig when secret set → unauthorized
    assert!(decode(&blob, None, reg, Some(secret), 1000, 100_000).is_err());
    // tampered sig → unauthorized
    assert!(decode(&blob, Some("AAAAAAAA"), reg, Some(secret), 1000, 100_000).is_err());
}
```

- [ ] **Step 2: Run to verify it passes the round-trips**

Run: `cargo test --lib wire::golden`
Expected: PASS (these exercise the already-built codec).

- [ ] **Step 3: Add the immortality golden-vector test**

Append to `src/wire/golden.rs`:
```rust
/// Frozen (blob → params) vectors. If this fails after a code change, the wire
/// format drifted and previously-published URLs would break. Regenerate ONLY by
/// a deliberate version bump.
#[test]
fn golden_vectors_decode_stably() {
    let reg = Registry::load();
    // (blob, expected title) pinned from a known-good build.
    let vectors: &[(&str, &str)] = &[
        // FILL from Step 4 output, e.g. ("EAM...", "Understanding Rust Ownership"),
    ];
    for (blob, title) in vectors {
        let out = decode(blob, None, reg, None, 1000, 100_000).unwrap();
        assert_eq!(out.title.as_deref(), Some(*title), "golden blob {blob} drifted");
    }
}
```

- [ ] **Step 4: Generate the golden blob and paste it in**

Add a temporary printer and run it once:
```rust
#[test]
#[ignore]
fn print_golden() {
    let reg = Registry::load();
    let t = load_templates();
    let (_, p) = cases().into_iter().nth(1).unwrap(); // "typical"
    let (blob, _) = encode(&p, reg, &t, None).unwrap();
    println!("GOLDEN_BLOB={blob}");
}
```
Run: `cargo test --lib wire::golden::print_golden -- --ignored --nocapture`
Paste the printed `blob` into the `vectors` array in Step 3 with title `"Understanding Rust Ownership"`. Then delete the `print_golden` fn.

- [ ] **Step 5: Run the full golden suite**

Run: `cargo test --lib wire::golden`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
cargo fmt && git add src/wire/golden.rs src/wire/mod.rs
git commit -m "test(wire): full-pipeline round-trip and frozen golden decode vectors"
```

---

## Task 8: Extract shared `render_response(state, params)`

**Files:**
- Create: `src/routes/render.rs`
- Modify: `src/routes/index.rs:64-335` (delegate)
- Modify: `src/routes/mod.rs:1-5` (add `pub mod render;`)

**Interfaces:**
- Produces: `pub async fn render_response(state: AppState, params: OgParams) -> axum::response::Response`.
- Consumes: unchanged generator/telemetry internals.

- [ ] **Step 1: Move the handler body into the shared fn**

Create `src/routes/render.rs`. Move the **entire body** of `generate` (current `src/routes/index.rs`, the code from line 68 `let span = …` through the final `}` of the match at line 334) **verbatim** into:
```rust
use super::timing::{CacheableDuration, ServerTiming};
use crate::{
    AppState,
    error::ApiError,
    generator::{self, GeneratorError, GradientCacheOutcome, Images, RenderOutput, TextContent},
    params::OgParams,
    telemetry,
};
use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use opentelemetry::KeyValue;
use std::time::{Duration, Instant};

struct RenderResult {
    output: RenderOutput,
    template_time: Duration,
    render_time: Duration,
    cache_outcome: GradientCacheOutcome,
}

fn extract_domain(url: &str) -> Option<String> {
    url::Url::parse(url).ok().and_then(|u| u.host_str().map(String::from))
}

#[tracing::instrument(
    name = "generate_og_image",
    skip_all,
    fields(
        template, ogis.has_logo, ogis.has_image, ogis.logo_domain,
        ogis.image_domain, ogis.format, ogis.scale, http.response.status_code,
    )
)]
pub async fn render_response(state: AppState, params: OgParams) -> Response {
    // <<< paste the verbatim body of the old `generate` here (lines 68..=334),
    //     unchanged. It already calls `.into_response()` in every branch, so the
    //     function returns `Response`. >>>
}

// also move `record_image_metrics` (index.rs:338-355) here.
fn record_image_metrics(image_type: &str, domain: &Option<String>, cached: bool, size: usize) {
    // <<< verbatim from index.rs >>>
}
```

- [ ] **Step 2: Make `generate` a thin wrapper**

Replace `src/routes/index.rs` body (keep the `#[utoipa::path]` doc attribute on `generate`) so the function is:
```rust
use crate::{AppState, params::OgParams};
use axum::extract::{Query, State};
use axum::response::IntoResponse;

#[utoipa::path(/* keep the existing attribute block unchanged */)]
pub async fn generate(
    State(state): State<AppState>,
    Query(params): Query<OgParams>,
) -> impl IntoResponse {
    crate::routes::render::render_response(state, params).await
}
```
Remove the now-moved `RenderResult`, `extract_domain`, `record_image_metrics`, and the unused imports from `index.rs`.

- [ ] **Step 3: Register the module**

In `src/routes/mod.rs`, add `pub mod render;` to the module list.

- [ ] **Step 4: Verify the query route still works**

Run: `cargo build && cargo test`
Expected: builds; existing tests pass. Manually: `cargo run` then `curl -s 'localhost:3000/?title=Hi' -o /tmp/a.png && file /tmp/a.png` → PNG.

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy && git add src/routes/render.rs src/routes/index.rs src/routes/mod.rs
git commit -m "refactor(routes): extract shared render_response for /c/ reuse"
```

---

## Task 9: `/c/` route handlers

**Files:**
- Create: `src/routes/compressed.rs`
- Modify: `src/routes/mod.rs` (add `pub mod compressed;` + routes)

**Interfaces:**
- Consumes: `wire::decode`, `Registry::load`, `render::render_response`, `AppState` (`hmac_validator`, `templates`, `max_input_length`).
- Produces: `generate_compressed`, `generate_compressed_signed`.

> **HMAC secret access:** `AppState.hmac_validator: Option<Arc<HmacValidator>>` holds the secret but does not currently expose the raw bytes. Add `pub fn secret(&self) -> &[u8]` to `HmacValidator` in `src/auth/hmac.rs` (returns `&self.secret`) and re-export remains via `auth::HmacValidator`. The `/c/` path verifies the blob signature itself (not the query middleware), so `/c/` routes are registered WITHOUT `hmac_auth_middleware`.

- [ ] **Step 1: Write a failing handler test**

Create `src/routes/compressed.rs` with a test that builds state-free decode expectations via the router (full e2e lives in Task 10); here unit-test the decode→error mapping helper:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_decoded_grows_and_admits_full_payload() {
        // Monotonic in input length, and large enough for the 5 full-length
        // text/URL fields (so a legitimate max-size request is never falsely capped).
        assert!(max_decoded_len(1000) > max_decoded_len(100));
        assert!(max_decoded_len(1000) >= 5 * 1000);
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test --lib routes::compressed`
Expected: FAIL (`max_decoded_len` undefined).

- [ ] **Step 3: Implement the handlers**

Replace the non-test contents of `src/routes/compressed.rs`:
```rust
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};

use crate::{error::ApiError, wire};
use crate::AppState;
use crate::wire::registry::Registry;

/// Decompressed-size cap derived from per-field limits (spec §8.1).
pub(crate) fn max_decoded_len(max_input_length: usize) -> usize {
    // 5 text/URL fields + MAX_EXTRA*(key+value), plus colors block, plus header/overhead.
    max_input_length * (5 + 2 * wire::MAX_EXTRA) + 5 * wire::MAX_COLORS + 64
}

async fn handle(state: AppState, blob: String, sig: Option<String>) -> Response {
    let reg = Registry::load();
    let secret: Option<Vec<u8>> = state.hmac_validator.as_ref().map(|v| v.secret().to_vec());
    let max_decoded = max_decoded_len(state.max_input_length);

    let params = match wire::decode(
        &blob,
        sig.as_deref(),
        reg,
        secret.as_deref(),
        state.max_input_length,
        max_decoded,
    ) {
        Ok(p) => p,
        Err(e) => return ApiError::from(e).into_response(),
    };

    // Reserved/deleted template id → fall back to the runtime default.
    let params = drop_dead_template(params, &state);

    crate::routes::render::render_response(state, params).await
}

/// If the decoded template name isn't a live template, clear it so the runtime
/// default applies (spec §5.2: reserved/deleted id → default render).
fn drop_dead_template(mut params: crate::params::OgParams, state: &AppState) -> crate::params::OgParams {
    if let Some(name) = &params.template {
        if !state.templates.templates.contains_key(name) {
            params.template = None;
        }
    }
    params
}

pub async fn generate_compressed(State(state): State<AppState>, Path(blob): Path<String>) -> Response {
    handle(state, blob, None).await
}

pub async fn generate_compressed_signed(
    State(state): State<AppState>,
    Path((blob, sig)): Path<(String, String)>,
) -> Response {
    handle(state, blob, Some(sig)).await
}
```

- [ ] **Step 4: Add the `secret()` accessor**

In `src/auth/hmac.rs`, inside `impl HmacValidator`, add:
```rust
    /// Raw secret bytes (used by the /c/ blob verifier).
    pub fn secret(&self) -> &[u8] {
        &self.secret
    }
```

- [ ] **Step 5: Run the unit test**

Run: `cargo test --lib routes::compressed`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
cargo fmt && git add src/routes/compressed.rs src/auth/hmac.rs
git commit -m "feat(routes): /c/ compressed-URL handlers with inline blob auth"
```

---

## Task 10: Wire routes into the router + end-to-end test

**Files:**
- Modify: `src/routes/mod.rs` (register `compressed` module + routes)
- Create: `tests/c_e2e.rs` (or `src/routes/compressed.rs` e2e via `axum::body`)

**Interfaces:**
- Consumes: `compressed::{generate_compressed, generate_compressed_signed}`.

- [ ] **Step 1: Register routes**

In `src/routes/mod.rs`: add `pub mod compressed;`, and in `create_router` add (after the `/templates` route, outside the query HMAC `route_layer`, so `/c/` is NOT covered by the query-signature middleware):
```rust
        .route("/c/{blob}", get(compressed::generate_compressed))
        .route("/c/{blob}/{sig}", get(compressed::generate_compressed_signed))
```
Keep the existing `metrics_middleware` applicable to these by placing them where the `metrics_middleware` `route_layer` already applies (i.e. add them before that `.route_layer(...)` call, alongside `/health` and `/templates`). Confirm the final `.layer(cors)` and `.with_state(state)` still wrap everything.

- [ ] **Step 2: Write the end-to-end test**

Create `src/routes/c_e2e.rs` and add `#[cfg(test)] mod c_e2e;` to `src/routes/mod.rs`:
```rust
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt; // oneshot

// Builds AppState the same way main.rs does, for an in-process router.
fn test_state() -> crate::AppState {
    // Reuse the smallest viable construction; mirror src/main.rs::run_server state assembly.
    // (If a test helper already exists, use it.)
    unimplemented!("assemble AppState: load_fonts, load_templates, ImageFetcher::new(...), no HMAC")
}

#[tokio::test]
async fn c_route_renders_png() {
    let state = test_state();
    let reg = crate::wire::registry::Registry::load();
    let templates = crate::templates::load_templates();
    let mut p = crate::params::OgParams {
        title: Some("Hello".into()), description: None, subtitle: None, logo: None,
        image: None, template: None, signature: None, format: None, scale: None,
        quality: None, extra: std::collections::HashMap::new(),
    };
    p.title = Some("Hello".into());
    let (blob, _) = crate::wire::encode(&p, reg, &templates, None).unwrap();

    let app = crate::routes::create_router(state);
    let res = app
        .oneshot(Request::builder().uri(format!("/c/{blob}")).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let ct = res.headers().get("content-type").unwrap();
    assert_eq!(ct, "image/png");
}

#[tokio::test]
async fn c_route_rejects_garbage_with_400() {
    let state = test_state();
    let app = crate::routes::create_router(state);
    let res = app
        .oneshot(Request::builder().uri("/c/not-a-real-blob").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
```

> **Note on `test_state()`:** if assembling `AppState` in a test is heavy, extract a `pub(crate) fn build_state(config: Config) -> AppState` from `main.rs::run_server` in this task and call it with a default `Config`. Add `tower` to `[dev-dependencies]` if not present (`tower = { version = "0.5", features = ["util"] }`).

- [ ] **Step 3: Run the e2e tests**

Run: `cargo test --lib routes::c_e2e`
Expected: PASS (PNG returned; garbage → 400).

- [ ] **Step 4: Manual smoke test**

Run: `cargo run` then in another shell encode a blob via a throwaway `--bin` or the printer from Task 7, and:
```bash
curl -s "localhost:3000/c/<blob>" -o /tmp/c.png && file /tmp/c.png
```
Expected: `PNG image data, 1200 x 630`.

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy && git add src/routes/mod.rs src/routes/c_e2e.rs Cargo.toml
git commit -m "feat(routes): register /c/ routes and add end-to-end tests"
```

---

## Task 11: Docs — OpenAPI + reference page

**Files:**
- Modify: `src/routes/docs.rs` (add `/c/{blob}` paths to the OpenAPI doc)
- Modify: `docs/api-reference/*` (add a Compressed URLs section)

- [ ] **Step 1: Add utoipa path annotations**

Add `#[utoipa::path]` get annotations for `/c/{blob}` and `/c/{blob}/{sig}` to the handlers in `src/routes/compressed.rs` (mirror the response codes from `index.rs::generate`: 200 png, 400 invalid compressed URL, 401 auth, 404/422/5xx as applicable), and register them in the `ApiDoc` `paths(...)` list in `src/routes/docs.rs`.

- [ ] **Step 2: Regenerate and verify the OpenAPI spec**

Run: `cargo run -- --export-openapi > /tmp/openapi.json && grep -c '"/c/' /tmp/openapi.json`
Expected: `≥ 1`.

- [ ] **Step 3: Write the reference doc**

Add a short MDX section under `docs/` describing: the `/c/<blob>[/<sig>]` shape, that it is the production form (query API is for debugging), that blobs are produced by SDKs, and the format-version / immortality guarantee. Link to `design/compressed-urls.md`.

- [ ] **Step 4: Commit**

```bash
git add src/routes/docs.rs src/routes/compressed.rs docs/
git commit -m "docs: document /c/ compressed URLs in OpenAPI and reference"
```

---

## Self-Review

**1. Spec coverage:**
- §2 URL shape → Tasks 9–10. §4 header/§5 body → Tasks 4–5. §5.2/§5.3 registries → Task 3. §5.5 scale milliunits → Task 4. §5.6 scheme tag → Task 4. §6 never-expand → Task 5. §7 HMAC body-signing/8-char → Task 6. §8 decode order/caps/400 → Tasks 5,9. §9 immortality (version byte, golden vectors, registry guard) → Tasks 3,5,7. §10 routing/shared core → Tasks 8–10. §11 brotli no-dict → Task 5. §12 caching → noted as forward-only (no task; correct — no output cache exists). §13 worked example → covered by Task 4 round-trips.
- **Gap intentionally deferred:** spec §9 "validation-policy tightening is breaking" and "per-template color SET drift" CI guards — Task 3 guards the name→id maps and the shrinking-template-set; the per-template color-SET and boundary-value golden vectors are a follow-up hardening task (note in PR). Add Task 3b if you want them now.

**2. Placeholder scan:** the only `<<< … >>>` markers are in Task 8, which is a deliberate **verbatim code move** (the body already exists in `index.rs:68-334`) — exact source line ranges are given. `test_state()` in Task 10 is flagged with a concrete construction path (extract `build_state` from `main.rs`). No other placeholders.

**3. Type consistency:** `pack_body(p, reg, templates)` / `unpack_body(bytes, reg, max)`; `encode(...) -> (String, Option<String>)` / `decode(...) -> OgParams`; `sign(secret, version, body) -> String` / `verify(...) -> Result<(), WireError>`; `decode_container -> (u8, Vec<u8>)`; `render_response(AppState, OgParams) -> Response`. Header nibble math `(FORMAT_VERSION << 4) | mode` and the signed `version` (nibble) are consistent between Task 5 and Task 6.

---

## Execution Handoff

**Plan complete and saved to `docs/superpowers/plans/2026-06-26-compressed-urls.md`. Two execution options:**

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints.

**Which approach?**
