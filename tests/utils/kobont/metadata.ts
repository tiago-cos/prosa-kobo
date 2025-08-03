import request from 'supertest';
import { MIDDLEWARE_URL } from '../common';

export async function getMetadata(bookId: string, jwt?: string) {
  let req = request(MIDDLEWARE_URL).get(`/v1/library/${bookId}/metadata`);

  if (jwt !== undefined) req = req.auth(jwt, { type: 'bearer' });

  return req.send();
}

export async function generateAliceMetadata(bookId: string) {
  const template = JSON.stringify(ALICE_METADATA_TEMPLATE);
  const replaced = template.replace(/{bookId}/g, bookId);
  return JSON.parse(replaced);
}

export async function generateDefaultMetadata(bookId: string) {
  const template = JSON.stringify(DEFAULT_METADATA_TEMPLATE);
  const replaced = template.replace(/{bookId}/g, bookId);
  return JSON.parse(replaced);
}

export function normalizeMetadata(book: any) {
  return {
    ...book,
    CoverImageId: book.CoverImageId.split('?')[0],
    DownloadUrls: book.DownloadUrls.map((dl: any) => ({
      ...dl,
      Url: dl.Url.split('?')[0]
    }))
  };
}

const ALICE_METADATA_TEMPLATE = [
  {
    CrossRevisionId: '{bookId}',
    RevisionId: '{bookId}',
    Publisher: {
      Name: null,
      Imprint: null
    },
    PublicationDate: '2008-06-27T00:00:00.0000000Z',
    Language: 'en',
    Isbn: 'http://www.gutenberg.org/11',
    Subtitle: null,
    Genre: null,
    Slug: null,
    CoverImageId: '{bookId}',
    IsSocialEnabled: true,
    WorkId: '{bookId}',
    ExternalIds: [],
    IsPreOrder: false,
    ContributorRoles: [
      {
        Name: 'Lewis Carroll',
        Role: 'Author'
      }
    ],
    IsInternetArchive: false,
    IsAnnotationExportDisabled: false,
    IsAiSummaryDisabled: false,
    EntitlementId: '{bookId}',
    Title: "Alice's Adventures in Wonderland",
    Description: null,
    Categories: [],
    DownloadUrls: [
      {
        DrmType: 'None',
        Format: 'KEPUB',
        Url: 'http://192.168.93.72:5001/books/{bookId}',
        Platform: 'Generic',
        Size: 204018
      }
    ],
    Contributors: ['Lewis Carroll'],
    Series: null,
    CurrentDisplayPrice: {
      TotalAmount: -1,
      CurrencyCode: ''
    },
    CurrentLoveDisplayPrice: {
      TotalAmount: 0
    },
    IsEligibleForKoboLove: false,
    PhoneticPronunciations: null,
    RelatedGroupId: null,
    Locale: {
      LanguageCode: '',
      ScriptCode: '',
      CountryCode: ''
    }
  }
];

const DEFAULT_METADATA_TEMPLATE = [
  {
    CrossRevisionId: '{bookId}',
    RevisionId: '{bookId}',
    Publisher: {
      Name: null,
      Imprint: null
    },
    PublicationDate: null,
    Language: null,
    Isbn: null,
    Subtitle: null,
    Genre: null,
    Slug: null,
    CoverImageId: '{bookId}',
    IsSocialEnabled: true,
    WorkId: '{bookId}',
    ExternalIds: [],
    IsPreOrder: false,
    ContributorRoles: [],
    IsInternetArchive: false,
    IsAnnotationExportDisabled: false,
    IsAiSummaryDisabled: false,
    EntitlementId: '{bookId}',
    Title: null,
    Description: null,
    Categories: [],
    DownloadUrls: [
      {
        DrmType: 'None',
        Format: 'KEPUB',
        Url: 'http://192.168.93.72:5001/books/{bookId}',
        Platform: 'Generic',
        Size: 204018
      }
    ],
    Contributors: [],
    Series: null,
    CurrentDisplayPrice: {
      TotalAmount: -1,
      CurrencyCode: ''
    },
    CurrentLoveDisplayPrice: {
      TotalAmount: 0
    },
    IsEligibleForKoboLove: false,
    PhoneticPronunciations: null,
    RelatedGroupId: null,
    Locale: {
      LanguageCode: '',
      ScriptCode: '',
      CountryCode: ''
    }
  }
];
