import request from 'supertest';
import { MIDDLEWARE_URL } from '../common';

export type checkContentRequest = {
  ContentId: string;
  etag: string;
};

export async function checkForChanges(books: checkContentRequest[]) {
  let req = request(MIDDLEWARE_URL).post(`/api/v3/content/checkforchanges`);

  return req.send(books);
}

export async function getAnnotations(bookId: string, jwt?: string) {
  let req = request(MIDDLEWARE_URL).get(`/api/v3/content/${bookId}/annotations`);

  if (jwt !== undefined) req = req.auth(jwt, { type: 'bearer' });

  return req.send();
}

export type annotationRequest = {
  startChar: number;
  startTag: string;
  endChar: number;
  endTag: string;
  source: string;
  note?: string;
};

export async function addAnnotation(bookId: string, annotation: annotationRequest, jwt?: string) {
  let req = request(MIDDLEWARE_URL).patch(`/api/v3/content/${bookId}/annotations`);

  if (jwt !== undefined) req = req.auth(jwt, { type: 'bearer' });

  let type = annotation.note !== undefined ? 'note' : 'highlight';

  let startTag = annotation.startTag.replace('.', '\\.');
  let endTag = annotation.endTag.replace('.', '\\.');

  const body: any = {
    updatedAnnotations: [
      {
        clientLastModifiedUtc: 'placeholder',
        highlightedText: 'placeholder',
        id: 'placeholder',
        location: {
          span: {
            chapterFilename: annotation.source,
            chapterProgress: 0,
            chapterTitle: 'placeholder',
            endChar: annotation.endChar,
            endPath: `span#${endTag}`,
            startChar: annotation.startChar,
            startPath: `span#${startTag}`
          }
        },
        type: type
      }
    ]
  };

  if (annotation.note !== undefined) {
    body.updatedAnnotations[0].noteText = annotation.note;
  }

  return req.send(body);
}

export async function updateAnnotation(bookId: string, annotationId: string, annotation: annotationRequest, jwt?: string) {
  let req = request(MIDDLEWARE_URL).patch(`/api/v3/content/${bookId}/annotations`);

  if (jwt !== undefined) req = req.auth(jwt, { type: 'bearer' });

  let type = annotation.note !== undefined ? 'note' : 'highlight';

  let startTag = annotation.startTag.replace('.', '\\.');
  let endTag = annotation.endTag.replace('.', '\\.');

  const body: any = {
    updatedAnnotations: [
      {
        clientLastModifiedUtc: 'placeholder',
        highlightedText: 'placeholder',
        id: annotationId,
        location: {
          span: {
            chapterFilename: annotation.source,
            chapterProgress: 0,
            chapterTitle: 'placeholder',
            endChar: annotation.endChar,
            endPath: `span#${endTag}`,
            startChar: annotation.startChar,
            startPath: `span#${startTag}`
          }
        },
        type: type
      }
    ]
  };

  if (annotation.note !== undefined) {
    body.updatedAnnotations[0].noteText = annotation.note;
  }

  return req.send(body);
}

export async function deleteAnnotation(bookId: string, annotationId: string, jwt?: string) {
  let req = request(MIDDLEWARE_URL).patch(`/api/v3/content/${bookId}/annotations`);

  if (jwt !== undefined) req = req.auth(jwt, { type: 'bearer' });

  const body = { deletedAnnotationIds: [annotationId] };

  return req.send(body);
}
