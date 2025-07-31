import { INVALID_TOKEN, randomString } from '../utils/common';
import { authDevice, authRefreshDevice, DEVICE_ALREADY_LINKED, DEVICE_ALREADY_UNLINKED, DEVICE_NOT_FOUND, getLinkedDevices, getUnlinkedDevices, INVALID_API_KEY, linkDevice, MISSING_API_KEY, unlinkDevice } from '../utils/kobont/devices';

describe('Device auth', () => {
  test('Simple', async () => {
    const { response, userKey } = await authDevice();
    expect(response.status).toBe(200);
    expect(response.body.UserKey).toEqual(userKey);

    const refreshResponse = await authRefreshDevice(response.body.RefreshToken);
    expect(refreshResponse.status).toBe(200);
  });

  test('Invalid refresh token', async () => {
    const refreshResponse = await authRefreshDevice('invalid');
    expect(refreshResponse.status).toBe(401);
    expect(refreshResponse.body.message).toBe(INVALID_TOKEN);
  });
});

describe('Device linking', () => {
  test('Complete', async () => {
    const apiKey = randomString(16);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    let linkedResponse = await getLinkedDevices(apiKey);
    expect(linkedResponse.status).toBe(200);
    expect(linkedResponse.body).not.toEqual(expect.arrayContaining([deviceId]));

    let unlinkedResponse = await getUnlinkedDevices();
    expect(unlinkedResponse.status).toBe(200);
    expect(unlinkedResponse.body).toEqual(expect.arrayContaining([expect.objectContaining({ device_id: deviceId })]));

    const linkResponse = await linkDevice(deviceId, apiKey);
    expect(linkResponse.status).toBe(200);

    linkedResponse = await getLinkedDevices(apiKey);
    expect(linkedResponse.status).toBe(200);
    expect(linkedResponse.body).toEqual(expect.arrayContaining([deviceId]));

    unlinkedResponse = await getUnlinkedDevices();
    expect(unlinkedResponse.status).toBe(200);
    expect(unlinkedResponse.body).not.toEqual(expect.arrayContaining([expect.objectContaining({ device_id: deviceId })]));

    const unlinkResponse = await unlinkDevice(deviceId, apiKey);
    expect(unlinkResponse.status).toBe(200);

    linkedResponse = await getLinkedDevices(apiKey);
    expect(linkedResponse.status).toBe(200);
    expect(linkedResponse.body).not.toEqual(expect.arrayContaining([deviceId]));

    unlinkedResponse = await getUnlinkedDevices();
    expect(unlinkedResponse.status).toBe(200);
    expect(unlinkedResponse.body).toEqual(expect.arrayContaining([expect.objectContaining({ device_id: deviceId })]));
  });

  test('Get linked devices without api key', async () => {
    let linkedResponse = await getLinkedDevices();
    expect(linkedResponse.status).toBe(400);
    expect(linkedResponse.body.message).toBe(MISSING_API_KEY);
  });

  test('Get linked devices invalid api key', async () => {
    let linkedResponse = await getLinkedDevices('this is an invalid key');
    expect(linkedResponse.status).toBe(400);
    expect(linkedResponse.body.message).toBe(INVALID_API_KEY);
  });

  test('Link non-existent device', async () => {
    let linkResponse = await linkDevice('non-existent', 'dummyKey');
    expect(linkResponse.status).toBe(404);
    expect(linkResponse.body.message).toBe(DEVICE_NOT_FOUND);
  });

  test('Link already linked device', async () => {
    const apiKey = randomString(16);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    let linkResponse = await linkDevice(deviceId, apiKey);
    expect(linkResponse.status).toBe(200);

    linkResponse = await linkDevice(deviceId, apiKey);
    expect(linkResponse.status).toBe(409);
    expect(linkResponse.body.message).toBe(DEVICE_ALREADY_LINKED);
  });

  test('Link device invalid key', async () => {
    let linkResponse = await linkDevice('non-existent', 'this is an invalid key');
    expect(linkResponse.status).toBe(400);
    expect(linkResponse.body.message).toBe(INVALID_API_KEY);
  });

  test('Unlink device non-existent device', async () => {
    let unlinkResponse = await unlinkDevice('non-existent', 'dummyKey');
    expect(unlinkResponse.status).toBe(404);
    expect(unlinkResponse.body.message).toBe(DEVICE_NOT_FOUND);
  });

  test('Unlink already unlinked device', async () => {
    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    let unlinkResponse = await unlinkDevice(deviceId, 'dummyKey');
    expect(unlinkResponse.status).toBe(409);
    expect(unlinkResponse.body.message).toBe(DEVICE_ALREADY_UNLINKED);
  });

  test('Unlink device invalid key', async () => {
    let unlinkResponse = await unlinkDevice('non-existent', 'this is as invalid key');
    expect(unlinkResponse.status).toBe(400);
    expect(unlinkResponse.body.message).toBe(INVALID_API_KEY);
  });
});
