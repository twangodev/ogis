---
title: Self Hosting
description: Deploy ogis on your own infrastructure
---

# Self Hosting

Run your own ogis instance for full control over your image generation.

## One-Click Deploy

[![Deploy on Railway](https://img.shields.io/badge/Deploy%20on-Railway-0B0D0E?style=for-the-badge&logo=railway&logoColor=white)](https://railway.com/deploy/Ax4OH3?referralCode=rXZ78U)
[![Deploy to Google Cloud](https://img.shields.io/badge/Deploy%20to-Google%20Cloud-4285F4?style=for-the-badge&logo=googlecloud&logoColor=white)](https://deploy.cloud.run/?git_repo=https://github.com/twangodev/ogis)

## Docker

The recommended way to run ogis:

```bash
docker run -d -p 3000:3000 twango/ogis:latest
```

With environment variables:

```bash
docker run -d \
  -p 3000:3000 \
  -e OGIS_DEFAULT_TEMPLATE=minimal \
  -e OGIS_HMAC_SECRET=your-secret-key \
  twango/ogis:latest
```

## Docker Compose

For development or simple deployments:

```yaml
# docker-compose.yml
services:
  ogis:
    image: twango/ogis:latest
    ports:
      - "3000:3000"
    environment:
      - OGIS_DEFAULT_TEMPLATE=twilight
      - OGIS_CACHE_SIZE=1000
    restart: unless-stopped
```

```bash
docker compose up -d
```

## Building from Source

Requirements: Rust 1.70+

```bash
git clone https://github.com/twangodev/ogis.git
cd ogis
cargo build --release
./target/release/ogis
```

## Configuration

Configure ogis via environment variables (prefixed with `OGIS_`) or CLI arguments.

### Server Options

| Environment Variable | CLI Argument | Default | Description |
|---------------------|--------------|---------|-------------|
| `OGIS_PORT` | `--port` | `3000` | Server port |
| `OGIS_HOST` | `--host` | `0.0.0.0` | Server bind address |

### Image Generation

| Environment Variable | CLI Argument | Default | Description |
|---------------------|--------------|---------|-------------|
| `OGIS_DEFAULT_TEMPLATE` | `--default-template` | `twilight` | Default template when not specified |
| `OGIS_DEFAULT_TITLE` | `--default-title` | `ogis` | Default title text |
| `OGIS_DEFAULT_DESCRIPTION` | `--default-description` | - | Default description text |
| `OGIS_MAX_INPUT_LENGTH` | `--max-input-length` | `500` | Maximum text length |

### Caching

| Environment Variable | CLI Argument | Default | Description |
|---------------------|--------------|---------|-------------|
| `OGIS_CACHE_SIZE` | `--cache-size` | `1000` | Number of images to cache in memory |

### Security

| Environment Variable | CLI Argument | Default | Description |
|---------------------|--------------|---------|-------------|
| `OGIS_HMAC_SECRET` | `--hmac-secret` | - | Enable HMAC authentication (see [Authentication](/docs/api/authentication)) |

### Image Fetching

| Environment Variable | CLI Argument | Default | Description |
|---------------------|--------------|---------|-------------|
| `OGIS_IMAGE_TIMEOUT` | `--image-timeout` | `10` | Timeout for fetching external images (seconds) |
| `OGIS_IMAGE_MAX_SIZE` | `--image-max-size` | `5242880` | Maximum image size (bytes, default 5MB) |
| `OGIS_IMAGE_HTTPS_ONLY` | `--image-https-only` | `true` | Only allow HTTPS URLs for images |

Run `ogis --help` for the complete list of options.

## Health Checks

ogis exposes a health endpoint for container orchestration:

```bash
curl http://localhost:3000/health
```

Returns `200 OK` when the service is ready.

## Reverse Proxy

### Nginx

```nginx
server {
    listen 80;
    server_name ogis.example.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_cache_valid 200 1h;
    }
}
```

### Caddy

```caddy
ogis.example.com {
    reverse_proxy localhost:3000
}
```

## Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ogis
spec:
  replicas: 2
  selector:
    matchLabels:
      app: ogis
  template:
    metadata:
      labels:
        app: ogis
    spec:
      containers:
      - name: ogis
        image: twango/ogis:latest
        ports:
        - containerPort: 3000
        env:
        - name: OGIS_CACHE_SIZE
          value: "2000"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
---
apiVersion: v1
kind: Service
metadata:
  name: ogis
spec:
  selector:
    app: ogis
  ports:
  - port: 80
    targetPort: 3000
```

## Using Your Instance

Point the SDK to your self-hosted instance:

```typescript
import { OgisClient } from 'ogis';

const ogis = new OgisClient({
  baseUrl: 'https://ogis.example.com'
});
```

For private instances, see [Authentication](/docs/api/authentication).
