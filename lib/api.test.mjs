import { afterAll, expect, test } from 'bun:test';

const originalWindow = globalThis.window;
const storage = new Map([['smartime_llm', '{"api_key":"legacy-demo"}']]);
globalThis.window = {
  localStorage: {
    removeItem: (key) => storage.delete(key),
    setItem: (key, value) => storage.set(key, value),
    getItem: (key) => storage.get(key) ?? null,
  },
};
const { API } = await import('./api.ts');
afterAll(() => {
  if (originalWindow === undefined) delete globalThis.window;
  else globalThis.window = originalWindow;
});

test('preview removes legacy storage and never retains submitted credentials', async () => {
  expect(storage.has('smartime_llm')).toBe(false);
  await API.saveLLMConfig({ api_key: 'synthetic-test-secret', model: 'demo', base_url: 'https://example.com/v1' });
  expect(storage.has('smartime_llm')).toBe(false);
  expect(API._mock.llm.api_key).toBe('');
  const status = await API.getLLMConfig();
  expect(status.has_api_key).toBe(false);
  expect('api_key' in status).toBe(false);
  expect(JSON.stringify(status)).not.toContain('synthetic-test-secret');
});
