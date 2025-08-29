import request from 'supertest';
import { MIDDLEWARE_URL } from '../common';

export const MISSING_DEVICE_ID = 'No device id was provided.';

export async function getOauthConfigurations(deviceId: string) {
  let req = request(MIDDLEWARE_URL).get(`/oauth/${deviceId}/.well-known/openid-configuration`);
  return req.send();
}

export async function getOauthToken(deviceId?: string) {
  let req = request(MIDDLEWARE_URL).post('/oauth/connect/token');
  if (deviceId !== undefined) req.query({ device_id: deviceId });

  return req.send();
}

export async function generateOauthConfigs(deviceId: string) {
  const url = new URL(MIDDLEWARE_URL);
  let response = OAUTH_CONFIG_TEMPLATE.replace(/{host}/g, url.host);
  response = response.replace(/{device_id}/g, deviceId);
  return JSON.parse(response);
}

const OAUTH_CONFIG_TEMPLATE = '{ "token_endpoint": "http://{host}/oauth/connect/token?device_id={device_id}" }';
