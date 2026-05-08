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

Each invocation discovers the live template list by calling the server's
`/templates` endpoint from k6's `setup()` block — no snapshot files or
manually-maintained lists. Per-template latency metrics are emitted for the
file-based templates (twilight, fish, …); the auto-composed gradient family
(864 templates) is reported via a single aggregate trend `template_gradient_all`.

Results land in `k6/results/`. Override defaults with `MODE`, `VUS`, `DURATION`,
`BASELINE_TEMPLATE` env vars.

### Modes

| `MODE`            | Behaviour |
|-------------------|-----------|
| `sequential`      | One VU rotates through templates with `REQUESTS_PER_TEMPLATE` hits each, capped by `SEQUENTIAL_MAX_ITERATIONS`. Good for per-template breakdown. |
| `baseline`        | `VUS` constant, single template (`BASELINE_TEMPLATE`). Steady-state throughput on one template. |
| `concurrent`      | `VUS` random across the filtered template list. Realistic mixed traffic with cache pressure. |
| `cache_pressure`  | Like `concurrent` but with a unique color override per iteration so the gradient cache key never repeats. Worst-case floor. |

### Gradient template controls

The gradient family contains 864 auto-composed `gradient-{gradient}-{layout}`
templates. Including all of them in every run is rarely useful — the gradient
cache budget is finite and the test would mostly measure cold renders.

| Env var             | Default     | Effect |
|---------------------|-------------|--------|
| `INCLUDE_GRADIENTS` | `true`      | Set `false` to test only file-based templates. |
| `INCLUDE_STATIC`    | `true`      | Set `false` to skip file-based templates (twilight, fish, …). |
| `GRADIENT_LAYOUTS`  | `centered`  | Comma-separated layout names to include. Set to `all` for every layout (864 templates). |
| `GRADIENT_SAMPLE`   | unset       | Cap the gradient list to this many entries (after layout filtering, sorted order). |

### Examples

```bash
# steady-state on one warm gradient
MODE=baseline BASELINE_TEMPLATE=gradient-aurora-centered DURATION=30s \
  docker compose -f docker-compose.benchmark.yml --profile bench run --rm k6

# realistic mixed traffic, cache fits comfortably (80 templates, ~240MB)
MODE=concurrent GRADIENT_LAYOUTS=centered DURATION=60s \
  docker compose -f docker-compose.benchmark.yml --profile bench run --rm k6

# stress: every gradient + every layout, cache evicts
MODE=concurrent GRADIENT_LAYOUTS=all DURATION=60s \
  docker compose -f docker-compose.benchmark.yml --profile bench run --rm k6

# worst case: cache key never repeats
MODE=cache_pressure DURATION=30s \
  docker compose -f docker-compose.benchmark.yml --profile bench run --rm k6
```

## Results

Benchmarks run on GitHub Actions with identical hardware.

See [latest runs](https://github.com/twangodev/ogis/actions/workflows/rust.yml) for results.
