import request from 'supertest';
import { MIDDLEWARE_URL } from '../common';

export const INVALID_COVER_TOKEN = 'The provided cover token is invalid.';

export async function getCover(bookId: string, height?: number, width?: number, token?: string) {
  let req = request(MIDDLEWARE_URL).get(`/images/${bookId}`);

  if (token !== undefined) req = req.query({ token: token });
  if (height !== undefined) req = req.query({ height: height });
  if (width !== undefined) req = req.query({ width: width });

  return req.send();
}
