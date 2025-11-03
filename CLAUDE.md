# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

OGIS is a Rust service that generates OpenGraph images by rendering SVG templates to PNG via HTTP API.

## Development Commands

```bash
cargo build                    # Debug build
cargo run                      # Run locally (port 3000)
cargo run --release            # Run optimized
cargo fmt                      # Format code
cargo clippy                   # Lint
cargo test                     # Run tests (none exist yet)
docker compose up -d           # Run with Docker
```

## Architecture

### Request Flow
`GET /?title=X&description=Y` → `routes/index.rs` → `params.rs` validation → fetch images → generate SVG → render PNG (1200x630)

### Key Modules
- `src/config.rs` - CLI args and env vars (`OGIS_*`)
- `src/params.rs` - Request parameter validation
- `src/image/` - Image fetching with SSRF protection (blocks private IPs)
- `src/generator/svg.rs` - SVG template processing (replaces `ogis_title`, `ogis_description`, etc.)
- `src/generator/png.rs` - SVG to PNG rendering

## Common Tasks

### Add new template
1. Create SVG in `templates/` with placeholders (`ogis_title`, etc.)
2. Register in `templates.yaml`

### Debug image fetch issues
- Set `RUST_LOG=debug` for detailed logs
- SSRF blocks private IPs (10.x, 192.168.x, etc.) - see `src/image/resolver.rs`
- Default: HTTPS only, 5MB limit, 10s timeout

### Modify defaults
Edit `src/config.rs` for default title/description/logo values.

## Security

### HMAC Authentication
- Optional signature-based authentication using HMAC-SHA256
- Enabled when `OGIS_HMAC_SECRET` is set
- Signature computed over canonical query string (params sorted alphabetically, excluding signature param)
- Code in `src/auth/` module

## Important Notes
- No unit tests exist
- Cache is in-memory only (Moka library)
- Templates and fonts bundled in Docker image
- API docs at `/docs` endpoint
