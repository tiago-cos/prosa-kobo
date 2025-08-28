import { createHash } from 'crypto';
import request from 'supertest';
import { MIDDLEWARE_URL, randomString } from '../common';

export const DEVICE_NOT_FOUND = 'The requested device does not exist or is not accessible.';
export const DEVICE_ALREADY_LINKED = 'This device is already linked.';
export const DEVICE_ALREADY_UNLINKED = 'This device is already unlinked.';
export const INVALID_API_KEY = 'The provided api key is invalid.';
export const MISSING_API_KEY = 'The api key must be provided.';

function generateDeviceId(deviceId: string, userKey: string): string {
  const hash = createHash('sha256')
    .update(deviceId + userKey)
    .digest()
    .toString('base64')
    .replace(/\+/g, '-')
    .replace(/\//g, '_');

  return hash;
}

export async function getUnlinkedDevices() {
  let req = request(MIDDLEWARE_URL).get('/devices/unlinked');
  return req.send();
}

export async function getLinkedDevices(api_key?: string) {
  let req = request(MIDDLEWARE_URL).get('/devices/linked');

  if (api_key !== undefined) req = req.set('api-key', api_key);

  return req.send();
}

export async function linkDevice(device_id: string, api_key: string) {
  let req = request(MIDDLEWARE_URL).post('/devices/linked');

  return req.send({ device_id: device_id, api_key: api_key });
}

export async function unlinkDevice(device_id: string, api_key: string) {
  let req = request(MIDDLEWARE_URL).delete(`/devices/linked/${device_id}`);

  if (api_key !== undefined) req = req.set('api-key', api_key);

  return req.send();
}

export async function authDevice(deviceId?: string, userKey?: string) {
  let req = request(MIDDLEWARE_URL).post('/v1/auth/device');

  if (deviceId === undefined) deviceId = randomString(16);
  if (userKey === undefined) userKey = randomString(16);

  const body = {
    AffiliateName: 'Kobo',
    AppVersion: '1.0.1',
    ClientKey: 'client-key',
    DeviceId: deviceId,
    PlatformId: 'unimportant',
    SerialNumber: '30241001',
    UserKey: userKey
  };

  const response = await req.send(body);

  deviceId = generateDeviceId(deviceId, userKey);

  return { response, deviceId, userKey };
}

export async function authRefreshDevice(refreshToken: string) {
  let req = request(MIDDLEWARE_URL).post('/v1/auth/refresh');

  const body = {
    AppVersion: '1.0.1',
    ClientKey: 'client-key',
    PlatformId: 'unimportant',
    RefreshToken: refreshToken
  };

  return req.send(body);
}
