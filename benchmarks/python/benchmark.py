#!/usr/bin/env python3
"""
Benchmark suite comparing OGIS vs Next.js @vercel/og
"""

import argparse
import asyncio
import json
import time
import uuid
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any
from urllib.parse import urlencode

import aiohttp
import docker
import requests
from PIL import Image
from io import BytesIO


class BenchmarkRunner:
    def __init__(self, ogis_url: str = "http://localhost:3000", vercel_url: str = "http://localhost:3001"):
        self.ogis_url = ogis_url
        self.vercel_url = vercel_url
        self.docker_client = docker.from_env()

    def load_scenarios(self, scenarios_file: Path) -> List[Dict[str, Any]]:
        """Load test scenarios from JSON file"""
        with open(scenarios_file) as f:
            data = json.load(f)
        return data["scenarios"]

    def build_url(self, base_url: str, endpoint: str, params: Dict[str, str], cache_bust: bool = False) -> str:
        """Build URL with query parameters and optional cache busting"""
        url_params = params.copy()
        if cache_bust:
            url_params['subtitle'] = str(uuid.uuid4())
        query = urlencode(url_params)
        return f"{base_url}{endpoint}?{query}"

    def measure_latency(self, base_url: str, endpoint: str, params: Dict[str, str], runs: int = 100) -> Dict[str, float]:
        """Measure latency statistics for sequential requests"""
        latencies = []

        print(f"  Running {runs} sequential requests...")
        for i in range(runs):
            # Generate unique URL for each request to prevent caching
            url = self.build_url(base_url, endpoint, params, cache_bust=True)
            start = time.time()
            try:
                response = requests.get(url, timeout=30)
                if response.status_code == 200:
                    latencies.append((time.time() - start) * 1000)  # Convert to ms
                else:
                    print(f"    Warning: Request {i+1} returned status {response.status_code}")
            except Exception as e:
                print(f"    Error on request {i+1}: {e}")

        if not latencies:
            return {"error": "All requests failed"}

        latencies.sort()
        return {
            "mean": sum(latencies) / len(latencies),
            "median": latencies[len(latencies) // 2],
            "p95": latencies[int(len(latencies) * 0.95)],
            "p99": latencies[int(len(latencies) * 0.99)],
            "min": latencies[0],
            "max": latencies[-1],
            "successful_requests": len(latencies),
            "failed_requests": runs - len(latencies)
        }

    async def measure_throughput(self, base_url: str, endpoint: str, params: Dict[str, str], duration_seconds: int = 10) -> Dict[str, float]:
        """Measure throughput with concurrent requests"""
        request_count = 0
        errors = 0
        start_time = time.time()

        async def make_request(session):
            nonlocal request_count, errors
            # Generate unique URL for each request to prevent caching
            url = self.build_url(base_url, endpoint, params, cache_bust=True)
            try:
                async with session.get(url, timeout=30) as response:
                    await response.read()
                    if response.status == 200:
                        request_count += 1
                    else:
                        errors += 1
            except Exception:
                errors += 1

        print(f"  Running throughput test for {duration_seconds} seconds...")
        async with aiohttp.ClientSession() as session:
            tasks = []
            while time.time() - start_time < duration_seconds:
                # Launch 10 concurrent requests at a time
                batch = [make_request(session) for _ in range(10)]
                tasks.extend(batch)
                await asyncio.gather(*batch)
                await asyncio.sleep(0.01)  # Small delay between batches

        elapsed = time.time() - start_time
        return {
            "total_requests": request_count,
            "failed_requests": errors,
            "duration_seconds": elapsed,
            "requests_per_second": request_count / elapsed if elapsed > 0 else 0
        }

    def get_container_stats(self, container_name: str) -> Dict[str, Any]:
        """Get Docker container resource usage statistics"""
        try:
            container = self.docker_client.containers.get(container_name)
            stats = container.stats(stream=False)

            # Calculate CPU percentage (handle macOS Docker differences)
            cpu_delta = stats['cpu_stats']['cpu_usage']['total_usage'] - \
                       stats['precpu_stats']['cpu_usage']['total_usage']
            system_delta = stats['cpu_stats']['system_cpu_usage'] - \
                          stats['precpu_stats']['system_cpu_usage']

            # Use online_cpus if available, otherwise fallback to percpu_usage length or 1
            num_cpus = stats['cpu_stats'].get('online_cpus')
            if num_cpus is None:
                percpu = stats['cpu_stats']['cpu_usage'].get('percpu_usage', [])
                num_cpus = len(percpu) if percpu else 1

            cpu_percent = (cpu_delta / system_delta) * num_cpus * 100.0 if system_delta > 0 else 0

            # Get memory usage
            memory_usage = stats['memory_stats']['usage']
            memory_limit = stats['memory_stats']['limit']
            memory_percent = (memory_usage / memory_limit) * 100.0 if memory_limit > 0 else 0

            return {
                "cpu_percent": round(cpu_percent, 2),
                "memory_usage_mb": round(memory_usage / (1024 * 1024), 2),
                "memory_percent": round(memory_percent, 2)
            }
        except Exception as e:
            return {"error": str(e)}

    def measure_image_size(self, url: str) -> Dict[str, Any]:
        """Download image and measure size"""
        try:
            response = requests.get(url, timeout=30)
            if response.status_code != 200:
                return {"error": f"HTTP {response.status_code}"}

            image_bytes = response.content
            img = Image.open(BytesIO(image_bytes))

            return {
                "size_bytes": len(image_bytes),
                "size_kb": round(len(image_bytes) / 1024, 2),
                "width": img.width,
                "height": img.height,
                "format": img.format
            }
        except Exception as e:
            return {"error": str(e)}

    def run_benchmark(self, scenario: Dict[str, Any], runs: int = 100, throughput_duration: int = 10) -> Dict[str, Any]:
        """Run complete benchmark for a scenario"""
        print(f"\nBenchmarking scenario: {scenario['name']}")
        print(f"Description: {scenario['description']}")

        results = {
            "scenario": scenario['name'],
            "description": scenario['description'],
            "timestamp": datetime.now().isoformat(),
            "ogis": {},
            "vercel_og": {}
        }

        # Test OGIS
        print("\n[OGIS]")
        print("Measuring latency...")
        results['ogis']['latency'] = self.measure_latency(self.ogis_url, "/", scenario['params'], runs)

        print("Measuring throughput...")
        results['ogis']['throughput'] = asyncio.run(self.measure_throughput(self.ogis_url, "/", scenario['params'], throughput_duration))

        print("Getting resource usage...")
        results['ogis']['resources'] = self.get_container_stats('ogis-benchmark')

        print("Measuring image size...")
        # For image size, use a single URL without cache busting
        ogis_url = self.build_url(self.ogis_url, "/", scenario['params'])
        results['ogis']['image'] = self.measure_image_size(ogis_url)

        # Test Vercel OG
        print("\n[Vercel OG]")
        print("Measuring latency...")
        results['vercel_og']['latency'] = self.measure_latency(self.vercel_url, "/api/og", scenario['params'], runs)

        print("Measuring throughput...")
        results['vercel_og']['throughput'] = asyncio.run(self.measure_throughput(self.vercel_url, "/api/og", scenario['params'], throughput_duration))

        print("Getting resource usage...")
        results['vercel_og']['resources'] = self.get_container_stats('vercel-og-benchmark')

        print("Measuring image size...")
        # For image size, use a single URL without cache busting
        vercel_url = self.build_url(self.vercel_url, "/api/og", scenario['params'])
        results['vercel_og']['image'] = self.measure_image_size(vercel_url)

        # Calculate comparison
        results['comparison'] = self.calculate_comparison(results)

        return results

    def calculate_comparison(self, results: Dict[str, Any]) -> Dict[str, Any]:
        """Calculate performance comparison between services"""
        comparison = {}

        # Latency comparison
        if 'error' not in results['ogis']['latency'] and 'error' not in results['vercel_og']['latency']:
            ogis_p95 = results['ogis']['latency']['p95']
            vercel_p95 = results['vercel_og']['latency']['p95']

            comparison['latency_winner'] = 'OGIS' if ogis_p95 < vercel_p95 else 'Vercel OG'
            comparison['latency_improvement'] = f"{abs(ogis_p95 - vercel_p95) / max(ogis_p95, vercel_p95) * 100:.1f}%"

        # Throughput comparison
        if 'error' not in results['ogis']['throughput'] and 'error' not in results['vercel_og']['throughput']:
            ogis_rps = results['ogis']['throughput']['requests_per_second']
            vercel_rps = results['vercel_og']['throughput']['requests_per_second']

            comparison['throughput_winner'] = 'OGIS' if ogis_rps > vercel_rps else 'Vercel OG'

            # Only calculate improvement if both services have non-zero throughput
            if ogis_rps > 0 and vercel_rps > 0:
                comparison['throughput_improvement'] = f"{abs(ogis_rps - vercel_rps) / min(ogis_rps, vercel_rps) * 100:.1f}%"
            else:
                comparison['throughput_improvement'] = 'N/A (one or both services failed)'

        # Resource usage comparison
        if 'error' not in results['ogis']['resources'] and 'error' not in results['vercel_og']['resources']:
            comparison['cpu_usage'] = {
                'ogis': results['ogis']['resources']['cpu_percent'],
                'vercel_og': results['vercel_og']['resources']['cpu_percent']
            }
            comparison['memory_usage'] = {
                'ogis': results['ogis']['resources']['memory_usage_mb'],
                'vercel_og': results['vercel_og']['resources']['memory_usage_mb']
            }

        return comparison


def main():
    parser = argparse.ArgumentParser(description='Benchmark OGIS vs Vercel OG')
    parser.add_argument('--runs', type=int, default=1000, help='Number of latency test runs')
    parser.add_argument('--throughput-duration', type=int, default=10, help='Throughput test duration in seconds')
    parser.add_argument('--ogis-url', default='http://localhost:3000', help='OGIS service URL')
    parser.add_argument('--vercel-url', default='http://localhost:3001', help='Vercel OG service URL')
    parser.add_argument('--output', default='../results', help='Output directory for results')

    args = parser.parse_args()

    # Setup
    runner = BenchmarkRunner(args.ogis_url, args.vercel_url)
    scenarios_file = Path(__file__).parent / 'scenarios.json'
    scenarios = runner.load_scenarios(scenarios_file)

    # Create output directory
    output_dir = Path(args.output)
    output_dir.mkdir(parents=True, exist_ok=True)

    # Run benchmarks
    all_results = []
    for scenario in scenarios:
        result = runner.run_benchmark(scenario, args.runs, args.throughput_duration)
        all_results.append(result)

    # Save results
    timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
    output_file = output_dir / f'comparison_{timestamp}.json'

    with open(output_file, 'w') as f:
        json.dump(all_results, f, indent=2)

    print(f"\n{'='*60}")
    print(f"Results saved to: {output_file}")
    print(f"{'='*60}")

    # Print summary
    for result in all_results:
        print(f"\n{result['scenario'].upper()}")
        print(f"  Latency Winner: {result['comparison'].get('latency_winner', 'N/A')}")
        print(f"  Throughput Winner: {result['comparison'].get('throughput_winner', 'N/A')}")


if __name__ == '__main__':
    main()