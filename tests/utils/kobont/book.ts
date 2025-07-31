import request from 'supertest';
import { MIDDLEWARE_URL } from '../common';

export const INVALID_BOOK_TOKEN = 'The provided book token is invalid.';

export async function getBook(bookId: string, token?: string) {
  let req = request(MIDDLEWARE_URL).get(`/books/${bookId}`);

  if (token !== undefined) req = req.query({ token: token });

  return req.send();
}

export async function deleteBook(bookId: string, jwt?: string) {
  let req = request(MIDDLEWARE_URL).delete(`/v1/library/${bookId}`);

  if (jwt !== undefined) req = req.auth(jwt, { type: 'bearer' });

  return req.send();
}
