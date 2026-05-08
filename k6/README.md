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

Run via Docker (no host install required):

```bash
docker compose -f docker-compose.benchmark.yml --profile bench run --rm k6
docker compose -f docker-compose.benchmark.yml --profile bench-vercel-og run --rm k6-vercel-og
```

Results land in `k6/results/`. Override defaults with `MODE`, `VUS`, `DURATION`, `BASELINE_TEMPLATE` env vars.

## Results

Benchmarks run on GitHub Actions with identical hardware.

See [latest runs](https://github.com/twangodev/ogis/actions/workflows/rust.yml) for results.
