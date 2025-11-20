/**
 * Configuration options for the OGIS client
 */
export interface OgisClientOptions {
  /**
   * Base URL of the OGIS service (e.g., 'https://ogis.example.com')
   */
  baseUrl: string;

  /**
   * Optional HMAC secret for request signing
   * When provided, all requests will be signed with HMAC-SHA256
   */
  hmacSecret?: string;

  /**
   * Default parameters to include in all requests
   * These can be overridden on a per-request basis
   */
  defaults?: Partial<OgisParams>;
}

/**
 * Parameters for generating an OpenGraph image
 */
export interface OgisParams {
  /**
   * Template name to use for rendering
   */
  template?: string;

  /**
   * Title text to display in the image
   */
  title?: string;

  /**
   * Description text to display in the image
   */
  description?: string;

  /**
   * Logo URL to display in the image
   */
  logo?: string;

  /**
   * Custom parameters specific to the template
   * Any additional key-value pairs will be passed to the template
   */
  [key: string]: string | undefined;
}

/**
 * OpenGraph meta tag object
 */
export interface MetaTag {
  /**
   * The meta tag property name (e.g., 'og:image')
   */
  property: string;

  /**
   * The meta tag content value
   */
  content: string;
}

/**
 * Standard OpenGraph image dimensions used by OGIS
 */
export const OG_IMAGE_DIMENSIONS = {
  width: 1200,
  height: 630,
  type: 'image/png',
} as const;
