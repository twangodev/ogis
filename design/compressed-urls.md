# Design Spec: Compressed URLs (`/c/<blob>`)

**Status:** Draft (rev. 2, post adversarial review) · **Date:** 2026-06-26 · **Scope:** stateless, self-contained compressed URLs as the production request form for OGIS. The current `?query` API is retained as a debugging surface. SDK (encoder) implementation is **out of scope** for this spec, but the wire format defined here is the contract every encoder must follow.

### Revision history
- **rev. 2** — post adversarial review. HMAC now signs the **uncompressed body bytes** (rev. 1 reconstructed a canonical query string — fragile across languages); `scale` is **integer milliunits**, not f32; `format` codes **aligned to the `OutputFormat` enum**; URL scheme-stripping made explicit (3-state); default-omission restricted to enumerable fields; empty-vs-absent pinned; template default = runtime default (not hardcoded `twilight`); color/extra split is **per-resolved-template**; reserved-vs-deleted template IDs disambiguated; presence-bit continuation flag reserved; signature truncated to **8 chars / 48-bit**.

---

## 1. Goals & non-goals

**Goals**
- A production URL form `ogis.dev/c/<blob>` that encodes *all* current request parameters in as few URL characters as practical.
- Stateless and CDN-trivial: the blob is fully self-contained; the server holds no per-URL state.
- A wire format that can be decoded **forever** (published `og:image` URLs are cached indefinitely by third-party scrapers we do not control).

**Non-goals (for v1)**
- Browser-native (no-library) encoding. Brotli is not encodable via the browser `CompressionStream` API; SDKs do the encoding. A deflate-raw mode is *reserved* (§4.2).
- A trained/shared compression dictionary (honest gain ≈1 char for a permanent N-SDK byte-sync burden — rejected; see §11).
- Byte-canonical URLs across SDKs (open question, §14).

**Size reality** (illustrative estimates from the eval benchmark — only the minimal row below is independently confirmed; the rest assume the schema-pack here and should be re-measured against the real codec). The free-text `title`/`description` are the entropy floor (~1.3 path chars per source character):

| Request shape | `?query` baseline | `/c/` (est.) | Win |
|---|---|---|---|
| minimal (`title=Hello World`) | 19 (confirmed) | ~22–24 | **worse** |
| typical blog (title+desc) | ~124 | ~100–115 | ~10–16% |
| full text | 229 | ~120 | ~48% |
| with logo URL | 157 | ~116 | ~26% |
| image+colors | 132 | ~83 | ~37% |
| structural (colors/scale/quality, little text) | 131 | ~44–47 | ~64% |
| long article (~250 text chars) | 356 | ~183 | shrinks toward 0 |
| CJK / non-ASCII | 289 | ~99–120 | ~58–66% |

Big wins are **structural/color-heavy** and **non-ASCII** requests; English text-heavy requests win modestly. `/c/` is the canonical form, not a magic shrink ray.

---

## 2. URL shape

```
https://ogis.dev/c/<blob>                 # HMAC disabled
https://ogis.dev/c/<blob>/<sig>           # HMAC enabled (OGIS_HMAC_SECRET set)
```

- `<blob>` = base64url (RFC 4648 §5, **no padding**) of the byte container in §4.
- `<sig>` = base64url-nopad of the **leading 6 bytes** of the HMAC-SHA256 (§7) = **8 chars**.
- base64url uses `[A-Za-z0-9-_]` only — no `/`, `+`, or `=` — so each is an unambiguous single path segment (two plain axum path params, no catch-all, no blob-contains-slash hazard). The no-padding requirement is load-bearing for this.
- Cost: `ceil(container_bytes * 4 / 3)` chars for `<blob>`, plus `/c/` (3), plus `/<sig>` (9: slash + 8) when signed.

---

## 3. Encoding pipeline (encoder side, normative)

```
OgParams
  → schema-pack            (§5: compact binary body, version-tagged)   = BODY
  → sig = trunc6( HMAC-SHA256(secret, version_byte ‖ BODY) )            (§7, only if signing)
  → choose min(                                                          (§6: never-expand)
       container(mode=0, BODY),                 // raw
       container(mode=1, brotli_q11(BODY)) )    // compressed
  → base64url-nopad
  → "/c/" + blob  (+ "/" + base64url-nopad(sig) when signing)
```

The signature is computed over the **uncompressed** BODY, so it is independent of the compression-mode choice. The encoder MUST emit whichever of `{raw, brotli}` produces the shorter base64url string, recording the choice in the mode nibble (§4.2).

---

## 4. Container layout

### 4.1 Header byte (byte 0)

```
bit  7 6 5 4 | 3 2 1 0
     [version]|[ mode ]
```

- **version** (bits 4–7): schema-pack body format version. **v1 = `0x1`.** Read *before* decompression so the decoder dispatches both the decompressor and the body parser. Never repurpose a version number.
- **mode** (bits 0–3): compression applied to the body that follows.
  - `0x0` = none (raw schema-pack body)
  - `0x1` = brotli (generic window, quality 11, **no custom dictionary**)
  - `0x2` = *reserved* (deflate-raw, RFC 1951) — for a future browser-native encode path. **In v1 this is decode-REJECTED (400).** It may only be emitted once a future version makes deflate-raw decode a *mandatory permanent* requirement (see §9).
  - `0x3`–`0xF` = reserved (decode-rejected).

The header byte is **not** part of the signed data except for the explicit `version_byte` domain-separation prefix in §7 (the mode nibble varies and is excluded).

### 4.2 Body

Bytes `1..` are the schema-pack body (§5), optionally compressed per the mode nibble. Decompression is bounded (§8).

---

## 5. Schema-pack body, version 1 (normative)

All integers little-endian. Lengths are **LEB128 unsigned varints** (1 byte <128; ≤2 bytes given `max_input_length` defaults to 1000). All string lengths are **UTF-8 byte counts** (matches `String::len()` / `params.validate`).

```
[0..2]   presence  : u16 LE  field-presence bitfield (§5.1)
--- enumerable fields, each emitted iff its presence bit is set, in this order ---
template : u16 LE          (bit 5)   stable template ID (§5.2)
format   : u8              (bit 6)   format code (§5.4); png(0) is default → omitted
scale    : u16 LE          (bit 7)   milliunits = round(scale*1000) (§5.5)
quality  : u8              (bit 8)   1..=100
--- variable fields, each iff its bit is set, in this order ---
title       : varint(len)+UTF-8                       (bit 0)
description : varint(len)+UTF-8                       (bit 1)
subtitle    : varint(len)+UTF-8                       (bit 2)
logo        : u8 scheme-tag + varint(len)+UTF-8       (bit 3)   (§5.6)
image       : u8 scheme-tag + varint(len)+UTF-8       (bit 4)   (§5.6)
--- blocks ---
colors  : (bit 9)  u8 count N (≤MAX_COLORS), then N × { u16 LE color-id (§5.3) + 3 B RGB },
                   entries sorted ascending by color-id (canonical ordering)
extra   : (bit 10) u8 count N (≤MAX_EXTRA), then N × { varint(klen)+key + varint(vlen)+val },
                   entries sorted ascending by raw key bytes (canonical ordering)
```

Canonical block ordering (colors by id, extra by key bytes) makes the body a deterministic function of the logical request — needed for cross-SDK URL stability and reproducible golden vectors. (It is *not* required for signature validity, since the server signs the literal received body bytes — §7.)

### 5.1 Presence bitfield (u16)

| bit | meaning |
|---|---|
| 0 | title present |
| 1 | description present |
| 2 | subtitle present |
| 3 | logo present |
| 4 | image present |
| 5 | template present |
| 6 | format present (non-png) |
| 7 | scale present (non-1.0) |
| 8 | quality present (non-90) |
| 9 | colors block present |
| 10 | extra-overrides block present |
| 11–14 | **reserved, MUST be 0** in v1 (future optional fields) |
| 15 | **continuation flag** — reserved permanently for "extended presence follows"; **decode-rejected (400) in v1.** Reserved *now* (before bit exhaustion) so the append-only continuation path stays open. |

**Default omission applies ONLY to enumerable fields** (template/format/scale/quality), which have fixed protocol defaults. A clear bit for one of these means "use the default." Of these, only **format=png, scale=1.0, quality=90 are compile-time constants** safe to omit. For **template**, a clear bit means *None → the server applies its runtime default at render* (§5.2); because a generic SDK cannot know a deployment's runtime default template, **encoders SHOULD always emit the explicit template ID** to pin appearance.

**Text/URL fields (bits 0–4) encode `Option<String>` exactly:**
- clear bit ⇒ `None`. The decoder MUST leave these `None` (it does **not** substitute `OGIS_DEFAULT_*`); the shared generation core applies landing-page defaults via `is_empty()`/`with_defaults()`/`get_effective_logo()` exactly as the query route, so an all-clear body (and a template-only body) reproduces the `?` landing page.
- set bit + `varint(0)` ⇒ `Some("")`. This is **load-bearing and distinct from absent**: `?title=` (→ `Some("")`, not the landing page) differs from `?` (→ `None`, landing page). Encoders MUST NOT collapse `Some("")` to absent.

**`signature` is never in the body** — it is a separate path segment (§7).

### 5.2 Template ID — the immortality crux

Templates are **not** a small enum. The runtime set is **872 names**: 8 file templates (`twilight`, `daybreak`, `minimal`, `stripe`, `hero`, `modern`, `fish`, `reddit`) + the cartesian product of **72 gradients × 12 layouts** named `gradient-<gradient>-<layout>` (e.g. `gradient-aurora-centered`).

**Hazard:** today IDs would be implied by `src/templates.rs` sorting `gradient_names`/`layout_names` and iterating the product. Adding **one** gradient `.yaml` shifts every later ID, silently re-mapping every published URL.

**Requirements:**
- Template IDs come from a **checked-in, append-only registry** `src/wire/template-ids.json` (`name → u16 id`), decoupled from filesystem order. IDs are assigned once and **never change**. New templates append the next free ID. Deleted templates keep their ID **reserved** forever.
- **Decode semantics:** a `u16` that is **present in the committed registry** but has no live template (a deleted/reserved name) → render the **server default template** (do not error — a deleted template must still render *something* for old cards). A `u16` that is **not in the committed registry at all** → **400** (garbage; surfaces encoder bugs). This deliberately diverges from the query route's 404 for unknown templates.
- Clear template bit ⇒ `None` ⇒ server applies its runtime default at render (`state.templates.default`).
- u16 → headroom to ~65k (current usage 872).

### 5.3 Colors block & the per-template split

Color overrides arrive as `extra` keys whose names match a *resolved-template* color name (value = 6 hex chars). The query path decides "is this a color?" **per resolved template**: `extract_colors` accepts a key only if `state.templates.colors[template]` contains it; everything else becomes a verbatim `ogis_<key>` text override (value case preserved).

**The encoder MUST replicate that exact predicate.** Pack an `extra` entry into the colors block **only if** (a) the key is a color of the **resolved template** (per the shipped per-template color-name set) **and** (b) the value is already **exactly 6 lowercase-hex chars**. Every other entry — a registry-known color name not defined by *this* template, an uppercase/non-hex value, an unknown key — goes **verbatim into the extra block** (§5, bit 10). This means SDKs ship the **per-template color-name set**, not just a flat global registry.

- Color name → `u16 color-id` via a checked-in append-only `src/wire/color-ids.json` (same rules as §5.2).
- **Decode merges both blocks into a single `extra: HashMap<String,String>`** = `{color-id→name : "<6 lowercase hex, no '#'>"}` ∪ `{extra-block key : value}`. **No separate typed colors field exists post-decode**, so `extract_colors`/`extract_text_overrides` run **unchanged** (`extract_colors` itself prepends `#`, lowercases, and requires exactly 6 hex chars). Color entries MUST be written `extra[name] = "rrggbb"` (lowercase, no `#`).

### 5.4 Format code (append-only registry)

`format` byte aligns to `OutputFormat` (`src/generator/render.rs`): **`1 = jpeg`, `2 = webp`** (`0 = png` is the default, omitted). This matches the Rust enum ordinal, but the wire codes are an **independent append-only registry** — never reuse a code; add new formats with new codes. (Query aliases like `jpg` normalize to `jpeg` on decode.)

### 5.5 Scale (integer milliunits)

`scale` is a **u16 LE milliunit** value = `round(scale * 1000)`, present iff != 1.0 (bit 7). Validate against `[100, round(per_template_max_scale * 1000)]` (`max_scale` is per-template, defaults 1.0, and can exceed 2.55 — u16 covers up to 65.535). Integer (not f32) because a float has no language-agnostic canonical form; the integer is exact and identical across SDKs. (0.001 granularity is finer than any perceptible scale difference.)

### 5.6 Logo / image scheme tag

Each URL field, when present, is prefixed by a **1-byte scheme tag**, then `varint(len)+UTF-8`:
- `0` = verbatim (no scheme stripped) — used for empty `Some("")`, `data:`, scheme-relative `//host/x`, schemeless `host/x`, or anything not starting with a known scheme.
- `1` = `https://` stripped (re-prepended on decode).
- `2` = `http://` stripped.

The encoder uses `1`/`2` **only when the value literally starts with that scheme**; otherwise `0`. This keeps the stored/refetched string byte-identical to the original for all legal values (`validate` checks only byte length).

---

## 6. Never-expand rule

For small payloads every general compressor expands the blob (framing + base64url's 4/3). The encoder computes both `mode=0` (raw body) and `mode=1` (brotli body) and emits the one with the shorter **base64url** length, recording the choice in the header nibble. Worst case the blob equals raw body + 1 header byte. (Trivial payloads may still be a few chars longer than `?query`; SDKs MAY fall back to `?query` for those, but `/c/` MUST accept them.)

---

## 7. HMAC authentication

When `OGIS_HMAC_SECRET` is set:

```
sig = HMAC-SHA256( secret, version_byte ‖ uncompressed_BODY )      // §5 body, pre-compression
seg = base64url_nopad( sig[0..6] )                                 // leading 6 bytes → 8 chars
URL = /c/<blob>/<seg>
```

- **Sign the uncompressed body bytes**, not a reconstructed query string. This sidesteps every cross-language canonicalization hazard (float/case/percent-encoding/sort-order): the server HMACs the **exact decompressed body bytes** it received, so any `(body, sig)` the holder of the secret produced verifies, and no one without the secret can forge one. It is stable across compression mode and brotli level (we sign the *uncompressed* body).
- **Truncate to the leading 6 bytes (48-bit).** Forgery requires *online* guessing (a guess can't be checked offline without the secret): ~2⁴⁷ requests, ≈450 years at 10k req/s — far beyond practical for the abuse-prevention threat model (stop unauthorized/abusive image generation). The length is **pinned by the format version**; do not shorten below 8 chars.
- **No secret configured ⇒ no `<seg>` segment** required; `/c/<blob>` is accepted (matches current optional-auth behavior).
- The `/c/` verifier base64url-decodes `<seg>` (the query route hex-decodes its `signature`; these are **not** interchangeable). Constant-time compare of the 6 bytes.
- `/c/` and `?query` signatures are **not transferable**: the query path signs raw string values (incl. aliases like `format=jpg` and verbatim default-valued params); `/c/` signs the normalized binary body (format collapsed, defaults dropped, colors merged into extra). Different routes, different signatures.
- **Verify after bounded decompression** (§8 step 4), before render. `build_canonical_query` is *not* used by `/c/`.

---

## 8. Decode pipeline & safety (server, normative)

The blob is attacker-controlled and is decompressed/deserialized **before** `params.validate()` runs.

1. **Length gate.** Reject if `<blob>` exceeds `MAX_ENCODED_LEN` (§8.1) → 400.
2. **base64url-decode** → container bytes. Non-alphabet / bad length → 400.
3. **Header.** Read byte 0. Unknown version or unsupported/reserved mode (incl. `0x2` in v1) → 400.
4. **Bounded decompress** the body into a writer capped at `MAX_DECODED_LEN` (§8.1); abort the instant output would exceed it. **Never** pre-allocate from any length/count prefix.
5. **HMAC verify** (if secret set), over `version_byte ‖ body` vs `<seg>` (§7). Mismatch/missing → 401.
6. **Parse schema-pack** (version per header). Enforce while parsing: bit 15 set → 400; reserved bits 11–14 set → 400; each text/URL length ≤ `max_input_length`; each extra **key and value** ≤ `max_input_length`; `colors` count ≤ `MAX_COLORS`; `extra` count ≤ `MAX_EXTRA`; every decoded field is **well-formed UTF-8** (reject a varint that splits a codepoint — never truncate). Truncated/garbage/over-cap → 400.
7. **Build `OgParams`:** default-fill **only** template/format/scale/quality (clear template bit ⇒ `None`); text/URL clear bits ⇒ `None`; reconstruct URLs via the scheme tag; **merge colors+extra into one `extra` map** (§5.3). Reserved/deleted template ID ⇒ default template; unregistered ID ⇒ 400.
8. **`params.validate(max_input_length, per_template_max_scale)`** — same validation as the query route, including the per-template `max_scale` lookup.
9. **Render** via the shared generation core (§10).

On **any** decode/decompress/parse/validate failure: return a **400** `ApiError` (auth failures 401) — **never a fallback image** (a 200 on garbage poisons third-party caches forever).

### 8.1 Constants
- `MAX_DECODED_LEN = text_field_count*max_input_length + MAX_EXTRA*2*max_input_length + MAX_COLORS*5 + overhead` (each extra is an independently-bounded key+value; colors are 5 bytes each). With defaults ≈ a few-tens of KB ceiling; one explicit constant derived from `max_input_length`.
- `MAX_ENCODED_LEN`: keep the full URL under the ~2000-char de-facto ceiling for the common case; hard-reject above ~8 KB.
- `MAX_COLORS`, `MAX_EXTRA`: small fixed caps (e.g. 32 / 32).

---

## 9. Format immortality — evolution rules

The dominant risk is wire/semantic drift breaking live `og:image` cards.

- **Version byte first**, read before decompression. Keep **every** historical decoder compiled in, forever; `match` on version.
- **v1 body is append-only:** add only *optional* fields, only via reserved presence bits 11–14, then the **bit-15 continuation** scheme; **never** reorder, retype, or remove existing fields/bits. The format-code (§5.4) and template/color ID registries are append-only.
- **Registries & sets are CI-guarded:** fail the build if any committed `name → id` changes (mutations forbidden, additions allowed); **also** fail on a **shrinking live set** — deleting one layout YAML silently drops 72 reserved template IDs (one gradient drops 12) to the default render, and renaming/removing a **per-template color name** makes an old color-override URL decode fine yet 400 at `extract_colors`. Guard the per-template color **sets**, not just the global name→id map.
- **Default-template drift is breaking:** changing `templates.yaml: default` re-maps every already-published *omitted-template* URL. Forbidden once `/c/` is live (another reason encoders should emit explicit template IDs, §5.1).
- **Validation-policy drift is part of the immortality surface:** tightening any bound (lowering a template's `max_scale`, lowering `max_input_length`, removing an accepted format) retroactively **400s** already-published URLs that decode fine. Ship golden `blob → 200` vectors at boundary values so a tightening that would break an old URL fails CI.
- **CI golden vectors:** checked-in `(params → blob)` and `(blob → params)` byte-exact round-trips, plus signature vectors and the boundary/case vectors above (uppercase color input → lowercase; `*~+&= ` in text; non-ASCII; scale omitted vs explicit-default).
- **Spec is the source of truth**; code conforms to it.

---

## 10. Routing & code structure

- Add `GET /c/{blob}` and `GET /c/{blob}/{sig}` (base64url has no `/`, so plain path params suffice).
- Decode → `OgParams`, then funnel into the **same** generation core as `routes/index.rs::generate` (extract the post-`OgParams` body into a shared `fn` so `/c/` and `?query` cannot drift).
- The existing query-signature `hmac_auth_middleware` stays scoped to `/`. Add a **separate** `/c/` verifier (base64url `<seg>` over the body, §7) — it is *not* a reuse of the query middleware (which reads the query and hex-decodes). `build_canonical_query` is not used by `/c/`.
- Keep `GET /` (`?query`) explicitly documented as the **debug** surface.

## 11. Compression choice (settled)

- **Brotli, quality 11, no custom dictionary.** Best dictionary-free ratio on short web text (built-in RFC 7932 dictionary; beat zstd-no-dict on all 10 benchmark payloads) and available in every SDK language. "Popular libraries are fine," so the brotli-vs-deflate browser tension doesn't bind.
- **Rejected:** gzip (~18 B header), zstd/xz/bz2 (net-negative on short inputs), any trained/preset dictionary (benchmark "wins" were a data-leakage artifact; on unseen URLs they collapse and *lose* to no-dict brotli, for a permanent sync cost).
- **Rust deps:** `brotli` (+ `flate2` only if mode `0x2` is ever implemented). `base64` already present.

## 12. Caching (forward note)

No request→PNG output cache exists today (the Moka cache holds fetched logo/image bytes by source URL). If one is added, key it on a **canonical normalization of the decoded params** (sorted keys, applied defaults, normalized scale/format/quality/extras) so `/c/` and `?query` share entries — **not** on raw blob bytes.

---

## 13. Worked example

Request: `title=Understanding Rust Ownership` (28 B) `&description=A practical guide to borrowing and lifetimes` (44 B) `&template=twilight`.

If the encoder follows the recommendation to **emit the explicit template ID**, `template` adds a bit + 2 bytes; the example below shows the *omitted-default* variant (template bit clear) for the smallest illustration:
```
header   : 0x10            (v1, mode 0 — raw wins this size)           1 B
presence : 0x0003          (bits 0,1: title, description)              2 B
title    : 0x1C + 28 UTF-8 bytes                                       29 B
desc     : 0x2C + 44 UTF-8 bytes                                       45 B
                                                              total =  77 B
```
`base64url_nopad(77 B)` = 103 chars → `…/c/<103>` = 106 chars vs `?query` ~124. (Emitting the explicit `twilight` ID adds ~3 chars but removes the default-drift risk.)

---

## 14. Open questions

1. Must `/c/` URLs be byte-canonical **across SDKs** (for third-party scraper-cache dedup)? If yes, pin one encoder/level as canonical. If no (recommended for v1), accept equivalent-but-different URLs; the internal cache (§12) dedups by decoded params anyway. *(The canonical block ordering in §5 makes the body deterministic; only the compression-mode choice and brotli build can still differ.)*
2. ~~`<sig>` encoding/length~~ — **resolved:** base64url-nopad of the leading 6 HMAC bytes = 8 chars (48-bit).
3. ~~Continuation scheme~~ — **resolved:** bit 15 reserved now as the continuation flag (§5.1).
4. Where do the generated registries live and who owns the CI guard (`src/wire/*.json`, §5.2/§5.3/§9)?
5. Should encoders be *required* (not just recommended) to emit explicit template IDs, eliminating default-template drift entirely at the cost of ~3 chars on every URL?
