/**
 * @twangodev/ogis - TypeScript client for OGIS OpenGraph image generation
 *
 * @example
 * ```ts
 * import { OgisClient } from '@twangodev/ogis';
 *
 * const ogis = new OgisClient({
 *   baseUrl: 'https://ogis.example.com',
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
