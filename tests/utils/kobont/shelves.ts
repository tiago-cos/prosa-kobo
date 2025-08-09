import request from 'supertest';
import { MIDDLEWARE_URL } from '../common';

export async function createShelf(shelfName: string, jwt?: string) {
  let req = request(MIDDLEWARE_URL).post(`/v1/library/tags`);

  if (jwt !== undefined) req = req.auth(jwt, { type: 'bearer' });

  const body = {
    Items: [],
    Name: shelfName
  };

  return req.send(body);
}

export async function deleteShelf(shelfId: string, jwt?: string) {
  let req = request(MIDDLEWARE_URL).delete(`/v1/library/tags/${shelfId}`);

  if (jwt !== undefined) req = req.auth(jwt, { type: 'bearer' });

  return req.send();
}

export async function renameShelf(shelfId: string, shelfName: string, jwt?: string) {
  let req = request(MIDDLEWARE_URL).put(`/v1/library/tags/${shelfId}`);

  if (jwt !== undefined) req = req.auth(jwt, { type: 'bearer' });

  const body = {
    Name: shelfName
  };

  return req.send(body);
}

export async function addBooksToShelf(shelfId: string, bookIds: string[], jwt?: string) {
  let req = request(MIDDLEWARE_URL).post(`/v1/library/tags/${shelfId}/items`);

  if (jwt !== undefined) req = req.auth(jwt, { type: 'bearer' });

  const body = {
    Items: bookIds.map((id) => ({
      RevisionId: id,
      Type: 'ProductRevisionTagItem'
    }))
  };

  return req.send(body);
}

export async function deleteBooksFromShelf(shelfId: string, bookIds: string[], jwt?: string) {
  let req = request(MIDDLEWARE_URL).post(`/v1/library/tags/${shelfId}/items/delete`);

  if (jwt !== undefined) req = req.auth(jwt, { type: 'bearer' });

  const body = {
    Items: bookIds.map((id) => ({
      RevisionId: id,
      Type: 'ProductRevisionTagItem'
    }))
  };

  return req.send(body);
}
