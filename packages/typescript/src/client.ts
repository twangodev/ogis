import { generateHmacSignature } from './hmac.js';
import type { MetaTag, OgisClientOptions, OgisParams } from './types.js';
import { OG_IMAGE_DIMENSIONS } from './types.js';
import {
  buildQueryString,
  mergeParams,
  normalizeBaseUrl,
  validateBaseUrl,
} from './utils.js';

/**
 * OGIS client for generating OpenGraph image URLs
 *
 * @example
 * ```ts
 * import { OgisClient } from 'ogis';
 *
 * const ogis = new OgisClient({
 *   baseUrl: 'https://img.ogis.dev',
 *   hmacSecret: process.env.OGIS_SECRET,
 *   defaults: {
 *     template: 'twilight',
 *     logo: 'https://example.com/logo.png'
 *   }
 * });
 *
 * // Generate signed URL
 * const url = ogis.generateUrl({
 *   title: 'Hello World',
 *   description: 'My awesome post'
 * });
 * ```
 */
export class OgisClient {
  private readonly baseUrl: string;
  private readonly hmacSecret?: string;
  private readonly defaults?: Partial<OgisParams>;

  /**
   * Create a new OGIS client
   *
   * @param options - Client configuration options
   * @throws {Error} If baseUrl is invalid
   */
  constructor(options: OgisClientOptions) {
    validateBaseUrl(options.baseUrl);
    this.baseUrl = normalizeBaseUrl(options.baseUrl);
    if (options.hmacSecret !== undefined) {
      this.hmacSecret = options.hmacSecret;
    }
    if (options.defaults !== undefined) {
      this.defaults = options.defaults;
    }
  }

  /**
   * Generate OpenGraph image URL
   *
   * Merges default parameters with provided parameters,
   * and signs the request with HMAC if secret is configured.
   *
   * @param params - Image generation parameters
   * @returns Complete URL ready to use in meta tags
   *
   * @example
   * ```ts
   * const url = ogis.generateUrl({
   *   title: 'Hello World',
   *   description: 'My description',
   *   template: 'fish'
   * });
   * // Returns: "https://ogis.example.com/?description=My+description&template=fish&title=Hello+World&signature=abc123"
   * ```
   */
  generateUrl(params: OgisParams = {}): string {
    const mergedParams = mergeParams(this.defaults, params);

    // Add HMAC signature if secret is configured
    if (this.hmacSecret) {
      const signature = generateHmacSignature(this.hmacSecret, mergedParams);
      mergedParams.signature = signature;
    }

    const queryString = buildQueryString(mergedParams);
    return `${this.baseUrl}/${queryString}`;
  }

  /**
   * Generate OpenGraph meta tags for the image
   *
   * Returns an array of meta tag objects that can be easily
   * inserted into HTML or used with frameworks.
   *
   * @param params - Image generation parameters
   * @returns Array of meta tag objects
   *
   * @example
   * ```ts
   * const tags = ogis.generateMetaTags({ title: 'Hello' });
   * // Returns:
   * // [
   * //   { property: 'og:image', content: 'https://ogis.example.com/...' },
   * //   { property: 'og:image:width', content: '1200' },
   * //   { property: 'og:image:height', content: '630' },
   * //   { property: 'og:image:type', content: 'image/png' }
   * // ]
   *
   * // React example:
   * tags.map(tag => (
   *   <meta key={tag.property} property={tag.property} content={tag.content} />
   * ))
   *
   * // Next.js metadata example:
   * export const metadata = {
   *   openGraph: {
   *     images: [tags[0].content]
   *   }
   * }
   * ```
   */
  generateMetaTags(params: OgisParams = {}): MetaTag[] {
    const imageUrl = this.generateUrl(params);

    return [
      {
        property: 'og:image',
        content: imageUrl,
      },
      {
        property: 'og:image:width',
        content: OG_IMAGE_DIMENSIONS.width.toString(),
      },
      {
        property: 'og:image:height',
        content: OG_IMAGE_DIMENSIONS.height.toString(),
      },
      {
        property: 'og:image:type',
        content: OG_IMAGE_DIMENSIONS.type,
      },
    ];
  }
}
