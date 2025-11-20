import { describe, expect, it } from 'vitest';
import {
  buildQueryString,
  mergeParams,
  normalizeBaseUrl,
  validateBaseUrl,
} from '../src/utils.js';

describe('Utility functions', () => {
  describe('mergeParams', () => {
    it('merges parameters correctly', () => {
      const merged = mergeParams(
        { template: 'fish', logo: 'default.png' },
        { title: 'Hello', template: 'custom' }
      );

      expect(merged.title).toBe('Hello');
      expect(merged.template).toBe('custom'); // Overridden
      expect(merged.logo).toBe('default.png'); // From defaults
    });

    it('filters undefined values', () => {
      const merged = mergeParams(
        { template: 'fish' },
        { title: 'Hello', description: undefined }
      );

      expect(merged).toEqual({ template: 'fish', title: 'Hello' });
    });

    it('works with undefined defaults', () => {
      const merged = mergeParams(undefined, { title: 'Hello' });

      expect(merged).toEqual({ title: 'Hello' });
    });
  });

  describe('buildQueryString', () => {
    it('builds query string', () => {
      const qs = buildQueryString({ title: 'Hello', desc: 'World' });

      expect(qs).toContain('?');
      expect(qs).toContain('title=Hello');
      expect(qs).toContain('desc=World');
    });

    it('returns empty string for empty params', () => {
      const qs = buildQueryString({});

      expect(qs).toBe('');
    });

    it('URL encodes values', () => {
      const qs = buildQueryString({ title: 'Hello World' });

      expect(qs).toContain('Hello+World');
    });
  });

  describe('validateBaseUrl', () => {
    it('throws on empty URL', () => {
      expect(() => validateBaseUrl('')).toThrow('baseUrl is required');
    });

    it('throws on invalid URL', () => {
      expect(() => validateBaseUrl('not-a-url')).toThrow('Invalid baseUrl');
    });

    it('throws on non-HTTP protocol', () => {
      expect(() => validateBaseUrl('ftp://example.com')).toThrow(
        'http or https'
      );
    });

    it('accepts valid HTTPS URL', () => {
      expect(() => validateBaseUrl('https://example.com')).not.toThrow();
    });

    it('accepts valid HTTP URL', () => {
      expect(() => validateBaseUrl('http://localhost:3000')).not.toThrow();
    });
  });

  describe('normalizeBaseUrl', () => {
    it('removes trailing slash', () => {
      expect(normalizeBaseUrl('https://example.com/')).toBe(
        'https://example.com'
      );
    });

    it('leaves URL without trailing slash unchanged', () => {
      expect(normalizeBaseUrl('https://example.com')).toBe(
        'https://example.com'
      );
    });
  });
});
