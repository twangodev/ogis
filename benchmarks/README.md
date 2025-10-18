# OGIS vs Vercel OG Benchmark Suite

Comprehensive benchmark suite comparing OGIS performance against Next.js + @vercel/og.

## Structure

```
benchmarks/
├── docker-compose.yml      # Both services with equal resource limits
├── vercel-og/              # Next.js comparison service
│   ├── app/api/og/route.tsx
│   └── Dockerfile
├── python/
│   ├── benchmark.py        # Main benchmark runner
│   ├── scenarios.json      # Test scenarios
│   └── pyproject.toml      # UV dependencies
└── results/                # JSON output directory
```

## Prerequisites

- Docker & Docker Compose
- Python 3.11+ with uv
- Both services built and running

## Setup

### 1. Install Python dependencies

```bash
cd python
uv sync
```

### 2. Build and start services

```bash
cd benchmarks
docker-compose up --build -d
```

Wait for both services to be ready:
- OGIS: http://localhost:3000
- Vercel OG: http://localhost:3001

### 3. Verify services are running

```bash
# Test OGIS
curl "http://localhost:3000/generate?title=Test"

# Test Vercel OG
curl "http://localhost:3001/api/og?title=Test"
```

## Running Benchmarks

### Basic usage

```bash
cd python
uv run benchmark.py
```

### Custom options

```bash
# More latency runs for higher accuracy
uv run benchmark.py --runs 1000

# Longer throughput test
uv run benchmark.py --throughput-duration 30

# Custom service URLs
uv run benchmark.py --ogis-url http://ogis:3000 --vercel-url http://vercel:3001

# Custom output directory
uv run benchmark.py --output ./my-results
```

### Full command reference

```bash
uv run benchmark.py \
  --runs 1000 \
  --throughput-duration 30 \
  --ogis-url http://localhost:3000 \
  --vercel-url http://localhost:3001 \
  --output ../results
```

## What Gets Measured

### 1. Latency Metrics
- Mean response time
- Median (P50)
- P95 (95th percentile)
- P99 (99th percentile)
- Min/max response times

### 2. Throughput Metrics
- Total requests over duration
- Requests per second
- Failed request count

### 3. Resource Usage
- CPU percentage
- Memory usage (MB)
- Memory percentage

### 4. Image Quality
- PNG file size (bytes/KB)
- Image dimensions
- Image format

## Test Scenarios

Defined in `python/scenarios.json`:

1. **Simple Text**: Basic OG image with title and description
2. **With External Image**: OG image fetching and embedding a remote image

## Output

Results are saved as JSON in `results/comparison_{timestamp}.json`:

```json
[
  {
    "scenario": "simple_text",
    "timestamp": "2025-10-18T13:00:00",
    "ogis": {
      "latency": { "mean": 45.2, "p95": 78.1, "p99": 92.3 },
      "throughput": { "requests_per_second": 156.8 },
      "resources": { "cpu_percent": 32.5, "memory_usage_mb": 245.3 },
      "image": { "size_kb": 87.5 }
    },
    "vercel_og": {
      "latency": { "mean": 62.8, "p95": 105.2, "p99": 125.7 },
      "throughput": { "requests_per_second": 98.3 },
      "resources": { "cpu_percent": 45.2, "memory_usage_mb": 512.8 },
      "image": { "size_kb": 92.1 }
    },
    "comparison": {
      "latency_winner": "OGIS",
      "latency_improvement": "28.1%",
      "throughput_winner": "OGIS",
      "throughput_improvement": "59.5%"
    }
  }
]
```

## Docker Resource Limits

Both services run with identical resource constraints:
- CPU: 1-2 cores
- Memory: 512MB-1GB

Edit `docker-compose.yml` to adjust limits.

## Troubleshooting

### Services not starting

```bash
docker-compose logs ogis
docker-compose logs vercel-og
```

### Connection refused errors

Ensure services are fully initialized:
```bash
docker-compose ps
```

Both should show "Up" status.

### Benchmark fails to connect

Check if ports are accessible:
```bash
curl -I http://localhost:3000/health
curl -I http://localhost:3001/api/og?title=Test
```

## Stopping Services

```bash
docker-compose down
```

To also remove volumes:
```bash
docker-compose down -v
```

## Adding Custom Scenarios

Edit `python/scenarios.json`:

```json
{
  "scenarios": [
    {
      "name": "your_scenario",
      "description": "Description of test case",
      "params": {
        "title": "Your Title",
        "description": "Your description",
        "image": "https://example.com/image.png"
      }
    }
  ]
}
```

## Performance Tips

1. **Warm-up runs**: Run benchmarks twice, use second result
2. **Stable environment**: Close other applications during testing
3. **Multiple runs**: Run benchmarks several times and average results
4. **Network conditions**: Test with and without external image URLs