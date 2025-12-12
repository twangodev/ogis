import { env } from '$env/dynamic/private';

const GRAPHQL_ENDPOINT = 'https://api.cloudflare.com/client/v4/graphql';

export interface CloudflareStats {
	requests: number;
	bytes: number;
	requestsFormatted: string;
	bytesFormatted: string;
	lastUpdated: string;
}

interface GraphQLResponse {
	data?: {
		viewer: {
			zones: Array<{
				httpRequests1dGroups: Array<{
					sum: {
						requests: number;
						bytes: number;
					};
				}>;
			}>;
		};
	};
	errors?: Array<{ message: string }>;
}

function formatNumber(num: number): string {
	if (num >= 1_000_000_000) {
		return (num / 1_000_000_000).toFixed(1) + 'B';
	}
	if (num >= 1_000_000) {
		return (num / 1_000_000).toFixed(1) + 'M';
	}
	if (num >= 1_000) {
		return (num / 1_000).toFixed(1) + 'K';
	}
	return num.toString();
}

function formatBytes(bytes: number): string {
	if (bytes >= 1024 ** 4) {
		return (bytes / 1024 ** 4).toFixed(2) + ' TB';
	}
	if (bytes >= 1024 ** 3) {
		return (bytes / 1024 ** 3).toFixed(2) + ' GB';
	}
	if (bytes >= 1024 ** 2) {
		return (bytes / 1024 ** 2).toFixed(2) + ' MB';
	}
	if (bytes >= 1024) {
		return (bytes / 1024).toFixed(2) + ' KB';
	}
	return bytes + ' B';
}

function zeroStats(): CloudflareStats {
	return {
		requests: 0,
		bytes: 0,
		requestsFormatted: '0',
		bytesFormatted: '0 B',
		lastUpdated: new Date().toISOString()
	};
}

export async function fetchCloudflareStats(): Promise<CloudflareStats> {
	const apiToken = env.CF_API_TOKEN;
	const zoneId = env.CF_ZONE_ID;

	if (!apiToken || !zoneId) {
		console.warn('Cloudflare credentials not configured, returning zero stats');
		return zeroStats();
	}

	// Query daily aggregated stats (allows longer time ranges than adaptive groups)
	const query = `
		query GetZoneStats($zoneTag: String!, $since: Date!) {
			viewer {
				zones(filter: { zoneTag: $zoneTag }) {
					httpRequests1dGroups(
						limit: 10000,
						filter: {
							date_geq: $since
						}
					) {
						sum {
							requests
							bytes
						}
					}
				}
			}
		}
	`;

	// Calculate date ~1 year ago (within Cloudflare's limit)
	const oneYearAgo = new Date();
	oneYearAgo.setFullYear(oneYearAgo.getFullYear() - 1);
	oneYearAgo.setDate(oneYearAgo.getDate() + 1); // Add buffer day
	const sinceDate = oneYearAgo.toISOString().split('T')[0];

	try {
		const response = await fetch(GRAPHQL_ENDPOINT, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
				Authorization: `Bearer ${apiToken}`
			},
			body: JSON.stringify({
				query,
				variables: {
					zoneTag: zoneId,
					since: sinceDate
				}
			})
		});

		if (!response.ok) {
			console.error('Cloudflare API error:', response.status, response.statusText);
			return zeroStats();
		}

		const result: GraphQLResponse = await response.json();

		if (result.errors?.length) {
			console.error('Cloudflare GraphQL errors:', result.errors);
			return zeroStats();
		}

		const zones = result.data?.viewer?.zones;
		if (!zones?.length || !zones[0].httpRequests1dGroups?.length) {
			console.warn('No Cloudflare data available');
			return zeroStats();
		}

		// Sum up all daily groups
		let totalRequests = 0;
		let totalBytes = 0;

		for (const group of zones[0].httpRequests1dGroups) {
			totalRequests += group.sum.requests;
			totalBytes += group.sum.bytes;
		}

		return {
			requests: totalRequests,
			bytes: totalBytes,
			requestsFormatted: formatNumber(totalRequests),
			bytesFormatted: formatBytes(totalBytes),
			lastUpdated: new Date().toISOString()
		};
	} catch (error) {
		console.error('Failed to fetch Cloudflare stats:', error);
		return zeroStats();
	}
}