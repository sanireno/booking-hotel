import test from 'node:test';
import assert from 'node:assert/strict';

globalThis.window = { BOOKING_API_URL:'http://api.example.test' };
const { request, ApiError, errorMessage } = await import('../api.js');

async function mockFetch(implementation, action) {
  const previous = globalThis.fetch;
  globalThis.fetch = implementation;
  try { await action(); } finally { globalThis.fetch = previous; }
}

test('public reads use the configured API and omit cookies and authentication', async () => {
  await mockFetch(async (url, options) => {
    assert.equal(url, 'http://api.example.test/hotels?limit=100&offset=0');
    assert.equal(options.method, 'GET');
    assert.equal(options.credentials, 'omit');
    assert.equal(options.cache, 'no-store');
    assert.equal(options.body, undefined);
    assert.equal(options.headers.Authorization, undefined);
    assert.equal(options.headers['Content-Type'], undefined);
    assert.ok(options.signal instanceof AbortSignal);
    return Response.json([{ id:'hotel-1' }]);
  }, async () => assert.deepEqual(await request('/hotels?limit=100&offset=0'), [{ id:'hotel-1' }]));
});

test('mutations serialize JSON and pass the exact Bearer token', async () => {
  await mockFetch(async (url, options) => {
    assert.equal(url, 'http://api.example.test/hotels/hotel-1');
    assert.equal(options.method, 'PUT');
    assert.equal(options.headers.Authorization, 'Bearer test-token');
    assert.equal(options.headers['Content-Type'], 'application/json');
    assert.deepEqual(JSON.parse(options.body), { name:'Hotel', description:null });
    return Response.json({ id:'hotel-1' });
  }, async () => assert.deepEqual(await request('/hotels/hotel-1', {
    method:'PUT', body:{ name:'Hotel', description:null }, token:'test-token',
  }), { id:'hotel-1' }));
});

test('204 delete responses do not attempt to parse JSON', async () => {
  await mockFetch(async () => new Response(null, { status:204 }),
    async () => assert.equal(await request('/hotels/hotel-1', { method:'DELETE' }), null));
});

test('API errors preserve status and server messages', async () => {
  await mockFetch(async () => Response.json({ error:'Hotel has bookings and cannot be deleted' }, { status:409 }), async () => {
    await assert.rejects(request('/hotels/hotel-1', { method:'DELETE' }), (error) => {
      assert.ok(error instanceof ApiError);
      assert.equal(error.status, 409);
      assert.match(errorMessage(error), /бронирования/);
      return true;
    });
  });
});

test('non-JSON failures still expose a useful API error', async () => {
  await mockFetch(async () => new Response('Bad gateway', { status:502 }), async () => {
    await assert.rejects(request('/hotels'), (error) => error.status === 502 && /временно/.test(errorMessage(error)));
  });
});

test('network failures are explicit and do not return fictional data', async () => {
  await mockFetch(async () => { throw new TypeError('Failed to fetch'); }, async () => {
    await assert.rejects(request('/hotels'), (error) => error instanceof ApiError && error.status === 0 && /сервером/.test(error.message));
  });
});

test('navigation cancellation is preserved for callers', async () => {
  const controller = new AbortController(); controller.abort();
  await mockFetch(async (_url, options) => {
    assert.equal(options.signal.aborted, true);
    throw new DOMException('Navigation cancelled', 'AbortError');
  }, async () => await assert.rejects(request('/hotels/one', { signal:controller.signal }), { name:'AbortError' }));
});

test('auth and validation errors are localized', () => {
  assert.match(errorMessage(new ApiError('Unauthorized', 401)), /снова/);
  assert.match(errorMessage(new ApiError('Forbidden', 403)), /прав/);
  assert.match(errorMessage(new ApiError('Email already exists', 409)), /email/);
  assert.match(errorMessage(new ApiError('Invalid name', 422)), /поля/);
});
