import waitOn from 'wait-on';

export default async () => {
  try {
    await waitOn({
      resources: ['http://localhost:5001/health'],
      timeout: 1000
    });
    console.log('[jest.setup] Middleware is healthy and running.');
  } catch (err) {
    console.error('[jest.setup] Healthcheck failed: middleware is not running.');
    throw new Error('Cannot run tests: Middleware is not running.');
  }

  try {
    await waitOn({
      resources: ['http://localhost:5000/health'],
      timeout: 1000
    });
    console.log('[jest.setup] Server is healthy and running.');
  } catch (err) {
    console.error('[jest.setup] Healthcheck failed: Prosa server is not running.');
    throw new Error('Cannot run tests: Prosa server is not running.');
  }
};
