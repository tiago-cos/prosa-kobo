import fs from 'fs';
import path from 'path';
import request from 'supertest';
import { BOOK_DIR, PROSA_URL } from '../common';

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

export async function deleteBook(book_id: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).delete(`/books/${book_id}`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}
