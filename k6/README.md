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

## Running locally (Docker only — no host install)

The benchmark runs entirely in containers. You only need Docker.

```bash
# OGIS (builds from local source on first run)
docker compose -f docker-compose.benchmark.yml --profile bench run --rm k6

# @vercel/og
docker compose -f docker-compose.benchmark.yml --profile bench-vercel-og run --rm k6-vercel-og
```

Results land in `k6/results/` (`summary.md`, `summary.html`, `results.json`).

### Tuning

Override via environment variables:

```bash
MODE=concurrent VUS=20 DURATION=30s \
  docker compose -f docker-compose.benchmark.yml --profile bench run --rm k6
```

| Var | Default | Notes |
|-----|---------|-------|
| `MODE` | `baseline` | `baseline` (only `minimal` template), `sequential` (per-template), `concurrent` (random) |
| `VUS` | `10` | Virtual users (concurrent clients) |
| `DURATION` | `60s` | Run length for `baseline` / `concurrent` |
| `REQUESTS_PER_TEMPLATE` | `50` | Iterations per template in `sequential` mode |
| `UID` / `GID` | `1000` | UID/GID the k6 container runs as so it can write `k6/results/`. Bash exports `UID` automatically; export `GID` if yours differs (`export GID=$(id -g)`). |

### Comparing two branches

`baseline` mode is the only fair head-to-head when the template set differs between branches:

```bash
# On branch A
docker compose -f docker-compose.benchmark.yml build ogis
docker compose -f docker-compose.benchmark.yml --profile bench run --rm k6
mv k6/results k6/results-branch-a

# On branch B
git checkout other-branch
docker compose -f docker-compose.benchmark.yml build ogis
docker compose -f docker-compose.benchmark.yml --profile bench run --rm k6
mv k6/results k6/results-branch-b

diff k6/results-branch-a/summary.md k6/results-branch-b/summary.md
```

Force a rebuild after source changes with `docker compose build ogis`.

## CI

CI runs the same benchmark on GitHub Actions with `MODE` matrixed over `sequential`, `baseline`, and `concurrent`. See [latest runs](https://github.com/twangodev/ogis/actions/workflows/rust.yml).