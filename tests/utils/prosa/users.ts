import request from 'supertest';
import { PROSA_URL, randomString } from '../common';

export const INVALID_CREDENTIALS = 'Invalid credentials provided.';
export const INVALID_USERNAME_PASSWORD = 'Username and password must not contain special characters.';
export const USERNAME_TOO_BIG = 'Username must not exceed 20 characters.';
export const PASSWORD_TOO_BIG = 'Password must not exceed 256 characters.';
export const USERNAME_IN_USE = 'The username is already taken.';
export const USER_NOT_FOUND = 'The requested user does not exist or is not accessible.';
export const API_KEY_NOT_FOUND = 'The requested key does not exist or is not accessible.';
export const INVALID_CAPABILITIES = 'Invalid or unsupported capabilities provided.';
export const INVALID_TIMESTAMP = 'Expiration timestamp is invalid or incorrectly formatted.';
export const INVALID_PROVIDERS = 'Invalid or unsupported metadata provider selection.';
export const MISSING_METADATA_PREFERENCE = 'Automatic metadata preference must be present.';
export const INVALID_PREFERENCES = 'Invalid or unsupported preferences provided.';
export const INVALID_TOKEN = 'The provided token is invalid.';
export const TOKEN_NOT_FOUND = 'The refresh token was not found or cannot be accessed.';

export async function registerUser(username?: string, password?: string, adminKey?: string) {
  username = username ?? randomString(16);
  password = password ?? randomString(16);

  const payload: Record<string, string> = { username, password };
  if (adminKey) {
    payload.admin_key = adminKey;
  }

  const response = await request(PROSA_URL).post('/auth/register').send(payload);

  return { response, username, password };
}

export async function loginUser(username: string, password: string) {
  const payload = {
    username,
    password
  };

  const response = await request(PROSA_URL).post(`/auth/login`).send(payload);

  return response;
}

export async function logoutUser(refresh_token: string) {
  const payload = {
    refresh_token
  };

  const response = await request(PROSA_URL).post(`/auth/logout`).send(payload);

  return response;
}

export async function refreshToken(refresh_token: string) {
  const payload = {
    refresh_token
  };

  const response = await request(PROSA_URL).post(`/auth/refresh`).send(payload);

  return response;
}

export async function createApiKey(user_id: string, keyName: string, capabilities: string[], expiresAt?: number, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).post(`/users/${user_id}/keys`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  const body: any = {
    name: keyName,
    capabilities
  };

  if (expiresAt) body.expires_at = expiresAt;

  return req.send(body);
}

export async function getApiKey(user_id: string, keyId: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).get(`/users/${user_id}/keys/${keyId}`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}

export async function getApiKeys(user_id: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).get(`/users/${user_id}/keys`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}

export async function deleteApiKey(user_id: string, keyId: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).delete(`/users/${user_id}/keys/${keyId}`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}

export async function getPreferences(user_id: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).get(`/users/${user_id}/preferences`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}

export async function updatePreferences(user_id: string, providers?: string[], automatic_metadata?: boolean, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).put(`/users/${user_id}/preferences`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  const body: any = {};
  if (providers !== undefined) body.metadata_providers = providers;
  if (automatic_metadata !== undefined) body.automatic_metadata = automatic_metadata;

  return req.send(body);
}

export async function patchPreferences(user_id: string, providers?: string[], automatic_metadata?: boolean, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).patch(`/users/${user_id}/preferences`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  const body: any = {};
  if (providers !== undefined) body.metadata_providers = providers;
  if (automatic_metadata !== undefined) body.automatic_metadata = automatic_metadata;

  return req.send(body);
}

export async function getUserProfile(user_id: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).get(`/users/${user_id}`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}

export async function updateUserProfile(user_id: string, username: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).put(`/users/${user_id}`);

  const payload = {
    username
  };

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send(payload);
}
