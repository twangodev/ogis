import { describe, expect, it } from 'vitest';
import { buildCanonicalQuery, generateHmacSignature } from '../src/hmac.js';

describe('HMAC signature generation', () => {
  it('generates HMAC signature', () => {
    const signature = generateHmacSignature('test-secret', {
      title: 'Hello',
      description: 'World',
    });

    expect(signature).toBeTruthy();
    expect(typeof signature).toBe('string');
    expect(signature).toMatch(/^[a-f0-9]{64}$/); // SHA256 hex output
  });

  it('builds canonical query string', () => {
    const canonical = buildCanonicalQuery({
      z: 'last',
      a: 'first',
      m: 'middle',
    });

    expect(canonical).toBe('a=first&m=middle&z=last');
  });

  it('excludes signature from canonical query', () => {
    const canonical = buildCanonicalQuery({
      title: 'Hello',
      signature: 'should-be-excluded',
    });

    expect(canonical).toBe('title=Hello');
  });

  it('filters undefined values', () => {
    const canonical = buildCanonicalQuery({
      title: 'Hello',
      description: undefined,
    });

    expect(canonical).toBe('title=Hello');
  });

  it('URL encodes special characters', () => {
    const canonical = buildCanonicalQuery({
      title: 'Hello&World',
      desc: 'key=value',
    });

    expect(canonical).toContain('Hello%26World');
    expect(canonical).toContain('key%3Dvalue');
  });

  it('generates consistent signatures', () => {
    const params = { title: 'Test', description: 'Desc' };
    const sig1 = generateHmacSignature('secret', params);
    const sig2 = generateHmacSignature('secret', params);

    expect(sig1).toBe(sig2);
  });

  it('generates different signatures for different params', () => {
    const sig1 = generateHmacSignature('secret', { title: 'A' });
    const sig2 = generateHmacSignature('secret', { title: 'B' });

    expect(sig1).not.toBe(sig2);
  });

  it('generates different signatures for different secrets', () => {
    const params = { title: 'Test' };
    const sig1 = generateHmacSignature('secret1', params);
    const sig2 = generateHmacSignature('secret2', params);

    expect(sig1).not.toBe(sig2);
  });
});