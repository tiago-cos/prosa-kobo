import request from 'supertest';
import { PROSA_URL } from '../common';

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
