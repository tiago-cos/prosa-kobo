import request from 'supertest';
import { PROSA_URL } from '../common';

export const INVALID_RATING = 'The provided rating is invalid.';
export const INVALID_LOCATION = 'The provided location is invalid.';
export const INVALID_READING_STATUS = 'The provided reading status is invalid.';
export const INVALID_STATE = 'The provided state is invalid.';

export const EMPTY_STATE = { statistics: { reading_status: 'Unread' } };

export const ALICE_STATE = {
  location: {
    tag: 'kobo.4.2',
    source: 'OEBPS/229714655232534212_11-h-4.htm.xhtml'
  },
  statistics: {
    rating: 4.5,
    reading_status: 'Reading'
  }
};

export async function updateState(book_id: string, state: any, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).put(`/books/${book_id}/state`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send(state);
}

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
