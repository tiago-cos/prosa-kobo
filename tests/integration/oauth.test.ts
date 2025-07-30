import { randomString } from '../utils/common';
import { authDevice, linkDevice } from '../utils/kobont/devices';
import { getInitializationResponse } from '../utils/kobont/initialization';
import { generateOauthConfigs as generateOauthConfig, getOauthConfigurations, getOauthToken, MISSING_DEVICE_ID } from '../utils/kobont/oauth';

describe('Oauth configuration', () => {
  test('Simple', async () => {
    const deviceId = randomString(16);
    const expectedResponse = await generateOauthConfig(deviceId);

    const oauthConfigurationsResponse = await getOauthConfigurations(deviceId);
    expect(oauthConfigurationsResponse.status).toBe(200);
    expect(oauthConfigurationsResponse.body).toEqual(expectedResponse);
  });
});

describe('Oauth token', () => {
  test('Simple', async () => {
    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const apiKey = randomString(16);
    const linkResponse = await linkDevice(deviceId, apiKey);
    expect(linkResponse.status).toBe(200);

    const oauthTokenResponse = await getOauthToken(deviceId);
    expect(oauthTokenResponse.status).toBe(200);
    expect(oauthTokenResponse.body).toHaveProperty('id_token');
    expect(oauthTokenResponse.body).toHaveProperty('access_token');
    expect(oauthTokenResponse.body).toHaveProperty('expires_in');
    expect(oauthTokenResponse.body).toHaveProperty('token_type');
    expect(oauthTokenResponse.body).toHaveProperty('refresh_token');
    expect(oauthTokenResponse.body).toHaveProperty('scope');
    expect(oauthTokenResponse.body.scope).toBe('openid profile kobo_profile public_api_authenticated public_api_anonymous offline_access');
    expect(oauthTokenResponse.body.token_type).toBe('Bearer');

    // To verify that token works
    const initializationResponse = await getInitializationResponse(oauthTokenResponse.body.access_token);
    expect(initializationResponse.status).toBe(200);
  });

  test('No device id', async () => {
    const oauthTokenResponse = await getOauthToken();
    expect(oauthTokenResponse.status).toBe(401);
    expect(oauthTokenResponse.body.message).toBe(MISSING_DEVICE_ID);
  });
});
