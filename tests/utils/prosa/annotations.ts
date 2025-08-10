import request from 'supertest';
import { PROSA_URL } from '../common';

export const ALICE_NOTE = {
  source: 'OEBPS/229714655232534212_11-h-10.htm.xhtml',
  start_tag: 'kobo.74.1',
  end_tag: 'kobo.74.2',
  start_char: 7,
  end_char: 4,
  note: 'I loved this part!'
};

export async function addAnnotation(book_id: string, annotation: any, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).post(`/books/${book_id}/annotations`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send(annotation);
}

export async function listAnnotations(book_id: string, auth?: { jwt?: string; apiKey?: string }) {
  let req = request(PROSA_URL).get(`/books/${book_id}/annotations`);

  if (auth?.jwt) req = req.auth(auth.jwt, { type: 'bearer' });
  if (auth?.apiKey) req = req.set('api-key', auth.apiKey);

  return req.send();
}
