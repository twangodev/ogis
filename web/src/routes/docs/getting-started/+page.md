---
title: Getting Started
description: Quick start guide for using ogis
---

# Getting Started

Generate your first Open Graph image in under a minute.

## Installation

Install the TypeScript SDK:

```bash
npm install ogis
```

## Basic Usage

```typescript
import { OgisClient } from 'ogis';

const ogis = new OgisClient({ baseUrl: 'https://img.ogis.dev' });

// Generate an image URL
const imageUrl = ogis.generateUrl({
  title: 'My Blog Post',
  description: 'An introduction to ogis',
  template: 'twilight'
});

console.log(imageUrl);
// => https://img.ogis.dev/?title=My+Blog+Post&description=An+introduction+to+ogis&template=twilight
```

## Framework Integration

### Next.js (App Router)

```tsx
// app/blog/[slug]/page.tsx
import { OgisClient } from 'ogis';

const ogis = new OgisClient({ baseUrl: 'https://img.ogis.dev' });

export async function generateMetadata({ params }) {
  const post = await getPost(params.slug);

  return {
    title: post.title,
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

  export let data;

  const ogis = new OgisClient({ baseUrl: 'https://img.ogis.dev' });
  const ogImage = ogis.generateUrl({
    title: data.title,
    description: data.description,
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
const { title, description } = Astro.props;

const ogImage = ogis.generateUrl({ title, description });
---

<head>
  <meta property="og:image" content={ogImage} />
  <meta property="og:image:width" content="1200" />
  <meta property="og:image:height" content="630" />
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

### Plain HTML

No SDK needed — just construct the URL directly:

```html
<meta property="og:image" content="https://img.ogis.dev/?title=Hello%20World&template=twilight" />
<meta property="og:image:width" content="1200" />
<meta property="og:image:height" content="630" />
```

## Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `title` | Main heading text | `My Blog Post` |
| `description` | Secondary text below title | `A deep dive into...` |
| `subtitle` | Small text above title | `Tutorial` |
| `template` | Template name | `twilight`, `minimal`, `modern` |
| `logo` | URL to logo image | `https://example.com/logo.png` |
| `image` | URL to background/hero image | `https://example.com/hero.jpg` |

## Using Meta Tags Helper

The SDK includes a helper to generate all required meta tags:

```typescript
const ogis = new OgisClient({ baseUrl: 'https://img.ogis.dev' });

const tags = ogis.generateMetaTags({
  title: 'My Page',
  template: 'twilight'
});

// Returns:
// [
//   { property: 'og:image', content: 'https://img.ogis.dev/?...' },
//   { property: 'og:image:width', content: '1200' },
//   { property: 'og:image:height', content: '630' },
//   { property: 'og:image:type', content: 'image/png' }
// ]
```

## Setting Defaults

Configure default parameters that apply to all generated URLs:

```typescript
const ogis = new OgisClient({
  baseUrl: 'https://img.ogis.dev',
  defaults: {
    template: 'twilight',
    logo: 'https://example.com/logo.png'
  }
});

// Logo and template are automatically included
const url = ogis.generateUrl({ title: 'My Post' });
```

## Next Steps

- Browse templates in the [Playground](/playground)
- [Self-host](/docs/self-hosting) your own instance
- Add [authentication](/docs/api/authentication) for private instances
