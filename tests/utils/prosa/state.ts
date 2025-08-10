import request from 'supertest';
import { PROSA_URL } from '../common';

export async function patchState(book_id: string, state: any, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).patch(`/books/${book_id}/state`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send(state);
}

export async function getState(book_id: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).get(`/books/${book_id}/state`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}
