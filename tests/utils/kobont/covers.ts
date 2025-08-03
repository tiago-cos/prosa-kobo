import request from 'supertest';
import { MIDDLEWARE_URL } from '../common';

export const INVALID_COVER_TOKEN = 'The provided cover token is invalid.';

export async function getCover(bookId: string, token?: string) {
  let req = request(MIDDLEWARE_URL).get(`/images/${bookId}`);

  if (token !== undefined) req = req.query({ token: token });

  return req.send();
}
