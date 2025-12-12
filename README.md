# ogis: Open Graph Images as a Service

[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/twangodev/ogis/rust.yml?logo=rust)](https://github.com/twangodev/ogis/actions)
[![Docker Pulls](https://img.shields.io/docker/pulls/twango/ogis?logo=docker)](https://hub.docker.com/r/twango/ogis)
[![Docker Image Size](https://img.shields.io/docker/image-size/twango/ogis/latest?logo=docker)](https://hub.docker.com/r/twango/ogis)
[![npm](https://img.shields.io/npm/v/ogis)](https://www.npmjs.com/package/ogis)
[![License](https://img.shields.io/github/license/twangodev/ogis)](https://github.com/twangodev/ogis/blob/main/LICENSE)

Generate beautiful [Open Graph images](https://ogp.me/) via URL. No design skills required.

<p align="center">
  <img src="https://img.ogis.dev/?template=gradient-storm&title=Open%20Graph%20Images&subtitle=Made%20Simple&description=Generate%20beautiful%20social%20images%20in%20milliseconds&logo=https://ogis.dev/logo-light.png" alt="OGIS Example" width="600"/>
</p>

## Quick Start

Install the SDK:

```bash
npm install ogis
```

### Next.js (App Router)

```tsx
// app/blog/[slug]/page.tsx
import { OgisClient } from 'ogis';

const ogis = new OgisClient({ baseUrl: 'https://img.ogis.dev' });

export async function generateMetadata({ params }) {
  const post = await getPost(params.slug);

  return {
    openGraph: {
      images: [ogis.generateUrl({
        title: post.title,
        description: post.excerpt,
        template: 'twilight'
      })]
    }
  };
}
```

### SvelteKit

```svelte
<script lang="ts">
  import { OgisClient } from 'ogis';

  const ogis = new OgisClient({ baseUrl: 'https://img.ogis.dev' });
  const ogImage = ogis.generateUrl({
    title: 'My Page',
    template: 'minimal'
  });
</script>

<svelte:head>
  <meta property="og:image" content={ogImage} />
  <meta property="og:image:width" content="1200" />
  <meta property="og:image:height" content="630" />
</svelte:head>
```

### Astro

```astro
---
import { OgisClient } from 'ogis';

const ogis = new OgisClient({ baseUrl: 'https://img.ogis.dev' });
const ogImage = ogis.generateUrl({
  title: frontmatter.title,
  description: frontmatter.description
});
---

<head>
  <meta property="og:image" content={ogImage} />
</head>
```

### Nuxt

```vue
<script setup lang="ts">
import { OgisClient } from 'ogis';

const ogis = new OgisClient({ baseUrl: 'https://img.ogis.dev' });

useSeoMeta({
  ogImage: ogis.generateUrl({
    title: 'My Page',
    template: 'modern'
  })
});
</script>
```

### Plain HTML / Any Framework

No SDK needed - just construct the URL:

```html
<meta property="og:image" content="https://img.ogis.dev/?title=Hello%20World&template=twilight" />
<meta property="og:image:width" content="1200" />
<meta property="og:image:height" content="630" />
```

## Templates

Browse all templates at [ogis.dev/playground](https://ogis.dev/playground).

<table>
  <tbody>
  <tr>
    <td align="center">
      <img src="https://img.ogis.dev/?template=twilight&title=Dark%20%26%20Bold&subtitle=Default%20Template&description=Perfect%20for%20tech%20and%20development%20content&logo=https://ogis.dev/logo-light.png" alt="Twilight" width="100%"/>
      <br/><code>twilight</code>
    </td>
    <td align="center">
      <img src="https://img.ogis.dev/?template=daybreak&title=Light%20%26%20Fresh&subtitle=Morning%20Vibes&description=Clean%20and%20professional%20design&logo=https://ogis.dev/logo-dark.png" alt="Daybreak" width="100%"/>
      <br/><code>daybreak</code>
    </td>
    <td align="center">
      <img src="https://img.ogis.dev/?template=minimal&title=Simple%20%26%20Clean&subtitle=Minimalist%20Design&description=Ultra-clean%20centered%20layout" alt="Minimal" width="100%"/>
      <br/><code>minimal</code>
    </td>
  </tr>
  <tr>
    <td align="center">
      <img src="https://img.ogis.dev/?template=stripe&title=Enterprise%20Ready&subtitle=Production%20Grade&description=Scalable%20image%20generation%20for%20your%20platform&logo=https://ogis.dev/logo-dark.png" alt="Stripe" width="100%"/>
      <br/><code>stripe</code>
    </td>
    <td align="center">
      <img src="https://img.ogis.dev/?template=gradient-aurora&title=Make%20It%20Pop!&subtitle=Endless%20Possibilities&description=Beautiful%20images%20in%20milliseconds&logo=https://ogis.dev/logo-light.png" alt="Gradient Aurora" width="100%"/>
      <br/><code>gradient-aurora</code>
    </td>
    <td align="center">
      <img src="https://img.ogis.dev/?template=hero&title=Dramatic%20Impact&subtitle=Full%20Background&description=Photo-heavy%20content%20with%20overlay&logo=https://ogis.dev/logo-light.png" alt="Hero" width="100%"/>
      <br/><code>hero</code>
    </td>
  </tr>
  </tbody>
</table>

## Parameters

All parameters are optional and passed as query parameters or SDK options.

| Parameter | Description | Example |
|-----------|-------------|---------|
| `title` | Main heading text | `My Blog Post` |
| `description` | Secondary text below title | `A deep dive into...` |
| `subtitle` | Small text above title | `Tutorial` |
| `template` | Template name (see above) | `twilight` |
| `logo` | URL to logo image | `https://example.com/logo.png` |
| `image` | URL to background/hero image | `https://example.com/hero.jpg` |

Templates may support additional color customization parameters. See the [playground](https://ogis.dev/playground) for template-specific options.

## Self-Hosting

[![Deploy on Railway](https://img.shields.io/badge/Deploy%20on-Railway-0B0D0E?style=for-the-badge&logo=railway&logoColor=white)](https://railway.com/deploy/Ax4OH3?referralCode=rXZ78U)
[![Deploy to Google Cloud](https://img.shields.io/badge/Deploy%20to-Google%20Cloud-4285F4?style=for-the-badge&logo=googlecloud&logoColor=white)](https://deploy.cloud.run/?git_repo=https://github.com/twangodev/ogis)

### Docker

```bash
docker run -d -p 3000:3000 twango/ogis:latest
```

### Docker Compose

```bash
docker compose up -d
```

### From Source

```bash
cargo build --release
./target/release/ogis
```

### Configuration

Configure via environment variables (prefixed with `OGIS_`) or CLI arguments:

| Environment Variable | CLI Argument | Default | Description |
|---------------------|--------------|---------|-------------|
| `OGIS_PORT` | `--port` | `3000` | Server port |
| `OGIS_HOST` | `--host` | `0.0.0.0` | Server host |
| `OGIS_HMAC_SECRET` | `--hmac-secret` | - | Enable HMAC authentication |
| `OGIS_DEFAULT_TEMPLATE` | `--default-template` | `twilight` | Default template |
| `OGIS_MAX_INPUT_LENGTH` | `--max-input-length` | `500` | Max text length |
| `OGIS_CACHE_SIZE` | `--cache-size` | `1000` | Image cache size |

Run `ogis --help` for all options.

## Authentication

For private instances, enable HMAC-SHA256 signature validation:

```bash
# Server
docker run -d -p 3000:3000 -e OGIS_HMAC_SECRET=your-secret twango/ogis:latest
```

```ts
// Client
const ogis = new OgisClient({
  baseUrl: 'https://your-instance.com',
  hmacSecret: process.env.OGIS_SECRET
});

// URLs are automatically signed
const url = ogis.generateUrl({ title: 'Secure Image' });
// => https://your-instance.com/?title=Secure+Image&signature=abc123...
```

See [Authentication docs](https://ogis.dev/docs/api/authentication) for details.

## Documentation

- [Getting Started](https://ogis.dev/docs/getting-started)
- [API Reference](https://ogis.dev/docs/api)
- [Self-Hosting Guide](https://ogis.dev/docs/self-hosting)
- [Authentication](https://ogis.dev/docs/api/authentication)

## License

[AGPL-3.0](https://github.com/twangodev/ogis/blob/main/LICENSE)