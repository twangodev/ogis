---
title: Self Hosting
description: Deploy OGIS on your own infrastructure
---

# Self Hosting

## Docker

```bash
docker compose up -d
```

Or run directly:

```bash
docker run -d -p 3000:3000 ogis
```

## Building from Source

```bash
cargo build --release
./target/release/ogis
```

## Configuration

Run `ogis --help` to see all available CLI options. Environment variables are prefixed with `OGIS_` (e.g., `OGIS_PORT`).

For authentication setup, see [Authentication](/docs/api/authentication).
