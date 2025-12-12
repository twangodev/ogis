/**
 * ogis - TypeScript client for OGIS OpenGraph image generation
 *
 * @example
 * ```ts
 * import { OgisClient } from 'ogis';
 *
 * const ogis = new OgisClient({
 *   baseUrl: 'https://img.ogis.dev',
 *   hmacSecret: process.env.OGIS_SECRET
 * });
 *
 * const url = ogis.generateUrl({ title: 'Hello World' });
 * ```
 *
 * @packageDocumentation
 */

export { OgisClient } from './client.js';
export type { MetaTag, OgisClientOptions, OgisParams } from './types.js';
export { OG_IMAGE_DIMENSIONS } from './types.js';
