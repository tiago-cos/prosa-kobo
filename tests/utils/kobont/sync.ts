import request from 'supertest';
import { MIDDLEWARE_URL } from '../common';

export async function sync(since?: number, jwt?: string) {
  let req = request(MIDDLEWARE_URL).get(`/v1/library/sync`);

  if (jwt !== undefined) req = req.auth(jwt, { type: 'bearer' });
  if (since !== undefined) req = req.set('X-Kobo-Synctoken', since.toString());

  return req.send();
}
