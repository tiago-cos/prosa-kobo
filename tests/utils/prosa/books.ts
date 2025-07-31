import request from 'supertest';
import path from 'path';
import fs from 'fs';
import { BOOK_DIR } from '../common';
import { PROSA_URL } from '../common';

export const BOOK_CONFLICT = 'This book is already in your library.';
export const BOOK_NOT_FOUND = 'The requested book does not exist or is not accessible.';
export const INVALID_BOOK = 'The provided EPUB data is invalid.';
export const INVALID_PAGINATION = 'The requested pagination is invalid.';

const bookCache: Record<string, Buffer> = {};

function preloadFiles() {
  for (const filename of fs.readdirSync(BOOK_DIR)) {
    const fullPath = path.join(BOOK_DIR, filename);
    bookCache[filename] = fs.readFileSync(fullPath);
  }
}

preloadFiles();

export async function uploadBook(owner_id?: string, epub_name?: string, auth?: { jwt?: string; apiKey?: string }) {
  if (epub_name === undefined) throw new Error('EPUB name is required.');

  const epubBuffer = bookCache[epub_name!];
  if (!epubBuffer) throw new Error(`EPUB file not preloaded: ${epub_name}`);

  let req = request(PROSA_URL).post(`/books`);

  if (owner_id !== undefined) req = req.field('owner_id', owner_id);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.attach('epub', epubBuffer);
}

export async function downloadBook(book_id: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).get(`/books/${book_id}`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}

export async function getBookFileMetadata(book_id: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).get(`/books/${book_id}/file-metadata`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}

export async function deleteBook(book_id: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).delete(`/books/${book_id}`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}

export async function searchBooks(username?: string, title?: string, author?: string, page?: any, size?: any, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).get(`/books`);

  if (username) req = req.query({ username });
  if (title) req = req.query({ title });
  if (author) req = req.query({ author });
  if (page) req = req.query({ page });
  if (size) req = req.query({ size });

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}
