# Benchmarks

Performance benchmarks comparing OG image generation solutions.

## What We Test

- **Latency** — Response time percentiles (P50, P95, P99)
- **Throughput** — Requests per second under load

## Providers

| Provider | Status |
|----------|--------|
| ogis | Tested |
| @vercel/og | Tested |

## Setup

Install [k6](https://k6.io/docs/get-started/installation/), then run:

```bash
k6 run ogis.js
k6 run vercel-og.js
```

## Results

Benchmarks run on GitHub Actions with identical hardware.

See [latest runs](https://github.com/twangodev/ogis/actions/workflows/rust.yml) for results.