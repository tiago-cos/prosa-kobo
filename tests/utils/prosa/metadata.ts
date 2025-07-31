import request from 'supertest';
import { PROSA_URL } from '../common';

export const METADATA_NOT_FOUND = 'The requested metadata does not exist or is not accessible.';
export const METADATA_CONFLICT = 'This book already has metadata.';
export const INVALID_METADATA = 'The provided metadata is invalid.';

export const EXAMPLE_METADATA = {
  title: 'To Kill a Mockingbird',
  subtitle: 'A novel by Harper Lee',
  description: 'To Kill a Mockingbird is a novel by Harper Lee, published in 1960. It is a classic of modern American literature and deals with serious issues like racial injustice and moral growth.',
  publisher: 'J.B. Lippincott & Co.',
  publication_date: -299819049000,
  isbn: '978-0-06-112008-4',
  contributors: [
    {
      name: 'Harper Lee',
      role: 'Author'
    }
  ],
  genres: ['American Literature', 'Classics', 'Fiction'],
  series: {
    title: 'To Kill a Mockingbird',
    number: 1
  },
  page_count: 281,
  language: 'English'
};

export const ALICE_METADATA = {
  contributors: [
    {
      name: 'Lewis Carroll',
      role: 'Author'
    }
  ],
  genres: ['Alice (Fictitious character from Carroll) -- Juvenile fiction', "Children's stories", 'Fantasy fiction', 'Imaginary places -- Juvenile fiction'],
  isbn: 'http://www.gutenberg.org/11',
  language: 'en',
  publication_date: 1214524800000,
  title: "Alice's Adventures in Wonderland"
};

export async function addMetadata(book_id: string, metadata: any, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).post(`/books/${book_id}/metadata`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send(metadata);
}

export async function updateMetadata(book_id: string, metadata: any, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).put(`/books/${book_id}/metadata`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send(metadata);
}

export async function patchMetadata(book_id: string, metadata: any, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).patch(`/books/${book_id}/metadata`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send(metadata);
}

export async function getMetadata(book_id: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).get(`/books/${book_id}/metadata`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}

export async function deleteMetadata(book_id: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).delete(`/books/${book_id}/metadata`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}

export async function addMetadataRequest(book_id: string, providers?: string[], auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).post(`/metadata-requests`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  const body: any = {};
  body.book_id = book_id;
  if (providers !== undefined) body.metadata_providers = providers;

  return req.send(body);
}

export async function listMetadataRequests(user_id?: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).get(`/metadata-requests`);

  if (user_id) req = req.query({ user_id: user_id });
  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}
