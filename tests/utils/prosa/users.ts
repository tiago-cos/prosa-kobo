import request from 'supertest';
import { PROSA_URL, randomString } from '../common';

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
