import { DEVICE_NOT_LINKED, randomString, UNAUTHENTICATED } from '../utils/common';
import { authDevice, linkDevice } from '../utils/kobont/devices';
import { generateGetTestsResponse, generateInitializationResponse, getInitializationResponse, getTests } from '../utils/kobont/initialization';

describe('Device initialization', () => {
  test('Simple', async () => {
    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const apiKey = randomString(16);
    const linkResponse = await linkDevice(deviceId, apiKey);
    expect(linkResponse.status).toBe(200);

    const expectedResponse = await generateInitializationResponse(deviceId);

    const initializationResponse = await getInitializationResponse(authResponse.body.AccessToken);
    expect(initializationResponse.status).toBe(200);
    expect(initializationResponse.body).toEqual(expectedResponse);
  });

  test('No auth', async () => {
    const initializationResponse = await getInitializationResponse();
    expect(initializationResponse.status).toBe(401);
    expect(initializationResponse.body.message).toBe(UNAUTHENTICATED);
  });

  test('Not linked', async () => {
    const { response: authResponse } = await authDevice();
    expect(authResponse.status).toBe(200);

    const initializationResponse = await getInitializationResponse(authResponse.body.AccessToken);
    expect(initializationResponse.status).toBe(401);
    expect(initializationResponse.body.message).toBe(DEVICE_NOT_LINKED);
  });
});

describe('Get tests', () => {
  test('Simple', async () => {
    const expectedResponse = await generateGetTestsResponse();

    const testsResponse = await getTests();
    expect(testsResponse.status).toBe(200);
    expect(testsResponse.body).toEqual(expectedResponse);
  });
});
