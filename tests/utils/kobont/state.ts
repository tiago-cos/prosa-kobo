import request from 'supertest';
import { MIDDLEWARE_URL } from '../common';

export const MISSING_BOOK_ID = 'A book ID must be provided.';

export async function getState(bookId: string, jwt?: string) {
  let req = request(MIDDLEWARE_URL).get(`/v1/library/${bookId}/state`);

  if (jwt !== undefined) req = req.auth(jwt, { type: 'bearer' });

  return req.send();
}

export async function updateState(bookId: string, tag: string, source: string, status: string, jwt?: string) {
  let req = request(MIDDLEWARE_URL).put(`/v1/library/${bookId}/state`);

  if (jwt !== undefined) req = req.auth(jwt, { type: 'bearer' });

  const body = {
    ReadingStates: [
      {
        EntitlementId: bookId,
        LastModified: 'placeholder',
        StatusInfo: {
          LastModified: 'placeholder',
          Status: status
        },
        Statistics: {
          LastModified: 'placeholder',
          SpentReadingMinutes: 0,
          RemainingTimeMinutes: 0
        },
        CurrentBookmark: {
          LastModified: 'placeholder',
          ProgressPercent: 0,
          ContentSourceProgressPercent: 0,
          Location: {
            Value: tag,
            Type: 'KoboSpan',
            Source: source
          }
        }
      }
    ]
  };

  return req.send(body);
}

export async function getRating(bookId?: string, jwt?: string) {
  let req = request(MIDDLEWARE_URL).get(`/v1/user/reviews`);

  if (jwt !== undefined) req = req.auth(jwt, { type: 'bearer' });
  if (bookId !== undefined) req = req.query({ ProductIds: bookId });

  return req.send();
}

export async function updateRating(bookId: string, rating: number, jwt?: string) {
  let req = request(MIDDLEWARE_URL).post(`/v1/products/${bookId}/rating/${rating}`);

  if (jwt !== undefined) req = req.auth(jwt, { type: 'bearer' });

  return req.send();
}

export async function getReviews(bookId: string, jwt?: string) {
  let req = request(MIDDLEWARE_URL).get(`/v1/products/${bookId}/reviews`);

  if (jwt !== undefined) req = req.auth(jwt, { type: 'bearer' });

  return req.send();
}
