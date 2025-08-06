import { DEVICE_NOT_LINKED, UNAUTHENTICATED } from '../utils/common';
import { authDevice, linkDevice } from '../utils/kobont/devices';
import { getRating, getReviews, getState, MISSING_BOOK_ID, updateRating, updateState } from '../utils/kobont/state';
import { uploadBook } from '../utils/prosa/books';
import { getState as getProsaState, patchState as patchKoboState } from '../utils/prosa/state';
import { createApiKey, registerUser } from '../utils/prosa/users';

describe('Fetch state', () => {
  test('Simple', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadResponse.status).toBe(200);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Read'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const getStateResponse = await getState(uploadResponse.text, authResponse.body.AccessToken);
    expect(getStateResponse.status).toBe(200);
    expect(getStateResponse.body).toHaveLength(1);
    expect(getStateResponse.body[0].EntitlementId).toEqual(uploadResponse.text);
    expect(getStateResponse.body[0].StatusInfo.Status).toEqual('ReadyToRead');
  });

  test('Non-existent book', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Read'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const getStateResponse = await getState('non-existent', authResponse.body.AccessToken);
    expect(getStateResponse.status).toBe(404);
  });

  test('Wrong capabilities', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadResponse.status).toBe(200);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Create'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    let linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const getStateResponse = await getState(uploadResponse.text, authResponse.body.AccessToken);
    expect(getStateResponse.status).toBe(403);
  });

  test('No auth', async () => {
    const getStateResponse = await getState('non-existent');
    expect(getStateResponse.status).toBe(401);
    expect(getStateResponse.body.message).toBe(UNAUTHENTICATED);
  });

  test('Not linked', async () => {
    const { response: authResponse } = await authDevice();
    expect(authResponse.status).toBe(200);

    const getStateResponse = await getState('non-existent', authResponse.body.AccessToken);
    expect(getStateResponse.status).toBe(401);
    expect(getStateResponse.body.message).toBe(DEVICE_NOT_LINKED);
  });
});

describe('Update state', () => {
  test('Simple', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadResponse.status).toBe(200);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Update'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    let expectedResponse: any = { statistics: { reading_status: 'Unread' } };

    let getStateResponse = await getProsaState(uploadResponse.text, { jwt: registerResponse.body.jwt_token });
    expect(getStateResponse.status).toBe(200);
    expect(getStateResponse.body).toEqual(expectedResponse);

    expectedResponse = {
      RequestResult: 'Success',
      UpdateResults: [
        {
          CurrentBookmarkResult: {
            Result: 'Success'
          },
          EntitlementId: uploadResponse.text,
          StatisticsResult: {
            Result: 'Success'
          },
          StatusInfoResult: {
            Result: 'Success'
          }
        }
      ]
    };

    const updateStateResponse = await updateState(uploadResponse.text, 'kobo.4.2', 'OEBPS/229714655232534212_11-h-4.htm.xhtml', 'Reading', authResponse.body.AccessToken);
    expect(updateStateResponse.status).toBe(200);
    expect(updateStateResponse.body).toEqual(expectedResponse);

    expectedResponse = {
      location: {
        tag: 'kobo.4.2',
        source: 'OEBPS/229714655232534212_11-h-4.htm.xhtml'
      },
      statistics: { reading_status: 'Reading' }
    };

    getStateResponse = await getProsaState(uploadResponse.text, { jwt: registerResponse.body.jwt_token });
    expect(getStateResponse.status).toBe(200);
    expect(getStateResponse.body).toEqual(expectedResponse);
  });

  test('Non-existent book', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Update'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const updateStateResponse = await updateState('non-existent', 'kobo.4.2', 'OEBPS/229714655232534212_11-h-4.htm.xhtml', 'Reading', authResponse.body.AccessToken);
    expect(updateStateResponse.status).toBe(404);
  });

  test('Wrong capabilities', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadResponse.status).toBe(200);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Read'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const updateStateResponse = await updateState(uploadResponse.text, 'kobo.4.2', 'OEBPS/229714655232534212_11-h-4.htm.xhtml', 'Reading', authResponse.body.AccessToken);
    expect(updateStateResponse.status).toBe(403);
  });

  test('No auth', async () => {
    const updateStateResponse = await updateState('non-existent', 'kobo.4.2', 'OEBPS/229714655232534212_11-h-4.htm.xhtml', 'Reading');
    expect(updateStateResponse.status).toBe(401);
    expect(updateStateResponse.body.message).toBe(UNAUTHENTICATED);
  });

  test('Not linked', async () => {
    const { response: authResponse } = await authDevice();
    expect(authResponse.status).toBe(200);

    const updateStateResponse = await updateState('non-existent', 'kobo.4.2', 'OEBPS/229714655232534212_11-h-4.htm.xhtml', 'Reading', authResponse.body.AccessToken);
    expect(updateStateResponse.status).toBe(401);
    expect(updateStateResponse.body.message).toBe(DEVICE_NOT_LINKED);
  });
});

describe('Fetch rating', () => {
  test('Simple', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadResponse.status).toBe(200);

    const patchStateResponse = await patchKoboState(uploadResponse.text, { statistics: { rating: 2.3 } }, { jwt: registerResponse.body.jwt_token });
    expect(patchStateResponse.status).toBe(204);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Read'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const getRatingResponse = await getRating(uploadResponse.text, authResponse.body.AccessToken);
    expect(getRatingResponse.status).toBe(200);
    expect(getRatingResponse.body.Items).toHaveLength(1);
    expect(getRatingResponse.body.Items[0].Id).toEqual(uploadResponse.text);
    expect(getRatingResponse.body.Items[0].Rating).toEqual(2);
  });

  test('No rating', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadResponse.status).toBe(200);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Read'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const getRatingResponse = await getRating(uploadResponse.text, authResponse.body.AccessToken);
    expect(getRatingResponse.status).toBe(200);
    expect(getRatingResponse.body.Items).toEqual([]);
  });

  test('Non-existent book', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Read'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const getRatingResponse = await getRating('non-existent', authResponse.body.AccessToken);
    expect(getRatingResponse.status).toBe(404);
  });

  test('No book id', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Read'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const getRatingResponse = await getRating(undefined, authResponse.body.AccessToken);
    expect(getRatingResponse.status).toBe(400);
    expect(getRatingResponse.body.message).toEqual(MISSING_BOOK_ID);
  });

  test('Wrong capabilities', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadResponse.status).toBe(200);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Delete'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const getRatingResponse = await getRating(uploadResponse.text, authResponse.body.AccessToken);
    expect(getRatingResponse.status).toBe(403);
  });

  test('No auth', async () => {
    const getRatingResponse = await getRating('non-existent');
    expect(getRatingResponse.status).toBe(401);
    expect(getRatingResponse.body.message).toBe(UNAUTHENTICATED);
  });

  test('Not linked', async () => {
    const { response: authResponse } = await authDevice();
    expect(authResponse.status).toBe(200);

    const getRatingResponse = await getRating('non-existent', authResponse.body.AccessToken);
    expect(getRatingResponse.status).toBe(401);
    expect(getRatingResponse.body.message).toBe(DEVICE_NOT_LINKED);
  });
});

describe('Update rating', () => {
  test('Simple', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadResponse.status).toBe(200);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Read', 'Update'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    let getRatingResponse = await getRating(uploadResponse.text, authResponse.body.AccessToken);
    expect(getRatingResponse.status).toBe(200);
    expect(getRatingResponse.body.Items).toHaveLength(0);

    const updateRatingResponse = await updateRating(uploadResponse.text, 5, authResponse.body.AccessToken);
    expect(updateRatingResponse.status).toBe(200);

    getRatingResponse = await getRating(uploadResponse.text, authResponse.body.AccessToken);
    expect(getRatingResponse.status).toBe(200);
    expect(getRatingResponse.body.Items).toHaveLength(1);
    expect(getRatingResponse.body.Items[0].Id).toEqual(uploadResponse.text);
    expect(getRatingResponse.body.Items[0].Rating).toEqual(5);
  });

  test('Non-existent book', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Read', 'Update'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const updateRatingResponse = await updateRating('non-existent', 5, authResponse.body.AccessToken);
    expect(updateRatingResponse.status).toBe(404);
  });

  test('Wrong capabilities', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadResponse.status).toBe(200);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Delete'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const updateRatingResponse = await updateRating(uploadResponse.text, 5, authResponse.body.AccessToken);
    expect(updateRatingResponse.status).toBe(403);
  });

  test('No auth', async () => {
    const updateRatingResponse = await updateRating('non-existent', 5);
    expect(updateRatingResponse.status).toBe(401);
    expect(updateRatingResponse.body.message).toBe(UNAUTHENTICATED);
  });

  test('Not linked', async () => {
    const { response: authResponse } = await authDevice();
    expect(authResponse.status).toBe(200);

    const updateRatingResponse = await updateRating('non-existent', 5, authResponse.body.AccessToken);
    expect(updateRatingResponse.status).toBe(401);
    expect(updateRatingResponse.body.message).toBe(DEVICE_NOT_LINKED);
  });
});

describe('Fetch reviews mock', () => {
  test('Simple', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Read', 'Update'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const expectedResponse = {
      ReviewSummary: {},
      Cursor: '1',
      Items: [],
      TotalPageCount: 10,
      CurrentPageIndex: 1
    };

    let getReviewsResponse = await getReviews('unimportant', authResponse.body.AccessToken);
    expect(getReviewsResponse.status).toBe(200);
    expect(getReviewsResponse.body).toEqual(expectedResponse);
  });

  test('No auth', async () => {
    const getReviewsResponse = await getReviews('non-existent');
    expect(getReviewsResponse.status).toBe(401);
    expect(getReviewsResponse.body.message).toBe(UNAUTHENTICATED);
  });

  test('Not linked', async () => {
    const { response: authResponse } = await authDevice();
    expect(authResponse.status).toBe(200);

    const getReviewsResponse = await getReviews('non-existent', authResponse.body.AccessToken);
    expect(getReviewsResponse.status).toBe(401);
    expect(getReviewsResponse.body.message).toBe(DEVICE_NOT_LINKED);
  });
});
