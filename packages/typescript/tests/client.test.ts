import { describe, expect, it } from 'vitest';
import { OgisClient } from '../src/client.js';

describe('OgisClient', () => {
  it('generates URL without HMAC', () => {
    const ogis = new OgisClient({ baseUrl: 'https://ogis.example.com' });
    const url = ogis.generateUrl({ title: 'Hello' });

    expect(url).toContain('title=Hello');
    expect(url).not.toContain('signature');
  });

  it('generates URL with HMAC signature', () => {
    const ogis = new OgisClient({
      baseUrl: 'https://ogis.example.com',
      hmacSecret: 'test-secret',
    });
    const url = ogis.generateUrl({ title: 'Hello' });

    expect(url).toContain('signature=');
  });

  it('merges default parameters', () => {
    const ogis = new OgisClient({
      baseUrl: 'https://ogis.example.com',
      defaults: { template: 'fish', logo: 'https://example.com/logo.png' },
    });
    const url = ogis.generateUrl({ title: 'Hello' });

    expect(url).toContain('template=fish');
    expect(url).toContain('title=Hello');
    expect(url).toContain('logo=https');
  });

  it('generates meta tags', () => {
    const ogis = new OgisClient({ baseUrl: 'https://ogis.example.com' });
    const tags = ogis.generateMetaTags({ title: 'Hello' });

    expect(tags).toHaveLength(4);
    expect(tags[0]?.property).toBe('og:image');
    expect(tags[1]?.property).toBe('og:image:width');
    expect(tags[2]?.property).toBe('og:image:height');
    expect(tags[3]?.property).toBe('og:image:type');
  });

  it('throws on invalid base URL', () => {
    expect(() => new OgisClient({ baseUrl: '' })).toThrow('baseUrl is required');
    expect(() => new OgisClient({ baseUrl: 'not-a-url' })).toThrow('Invalid baseUrl');
  });

  it('normalizes base URL trailing slash', () => {
    const ogis = new OgisClient({ baseUrl: 'https://ogis.example.com/' });
    const url = ogis.generateUrl({ title: 'Test' });

    // Should not have double slash between domain and query string
    expect(url).not.toMatch(/com\/\/\?/);
    expect(url).toMatch(/^https:\/\/ogis\.example\.com\/\?/);
  });
});