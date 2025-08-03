import request from 'supertest';
import { PROSA_URL } from '../common';

export const COVER_NOT_FOUND = 'The requested cover does not exist or is not accessible.';

export async function getCover(book_id: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).get(`/books/${book_id}/cover`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}

export async function deleteCover(book_id: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).delete(`/books/${book_id}/cover`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}
