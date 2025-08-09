import request from 'supertest';
import { PROSA_URL } from '../common';

export const INVALID_SHELF_NAME = 'The provided shelf name is invalid.';
export const SHELF_NAME_CONFLICT = 'There is already a shelf with this name in your library.';
export const SHELF_NOT_FOUND = 'The requested shelf does not exist or is not accessible.';
export const SHELF_BOOK_CONFLICT = 'The provided book is already present in this shelf.';
export const SHELF_BOOK_NOT_FOUND = 'The provided book does not exist in this shelf, or is not accessible.';

export async function createShelf(name: string, ownerId?: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).post(`/shelves`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  const body: any = { name };

  if (ownerId !== undefined) body.owner_id = ownerId;

  return req.send(body);
}

export async function getShelfMetadata(shelfId: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).get(`/shelves/${shelfId}`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}

export async function updateShelf(shelfId: string, name: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).put(`/shelves/${shelfId}`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send({ name });
}

export async function deleteShelf(shelfId: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).delete(`/shelves/${shelfId}`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}

export async function searchShelves(username?: string, name?: string, page?: any, size?: any, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).get(`/shelves`);

  if (username) req = req.query({ username });
  if (name) req = req.query({ name });
  if (page) req = req.query({ page });
  if (size) req = req.query({ size });

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}

export async function addBookToShelf(shelfId: string, bookId: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).post(`/shelves/${shelfId}/books`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  const body = {
    book_id: bookId
  };

  return req.send(body);
}

export async function listBooksFromShelf(shelfId: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).get(`/shelves/${shelfId}/books`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}

export async function deleteBookFromShelf(shelfId: string, bookId: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).delete(`/shelves/${shelfId}/books/${bookId}`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}
