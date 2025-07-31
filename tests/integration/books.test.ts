import fs from 'fs';
import path from 'path';
import { BOOK_DIR, DEVICE_NOT_LINKED, UNAUTHENTICATED } from '../utils/common';
import { deleteBook, getBook, INVALID_BOOK_TOKEN } from '../utils/kobont/book';
import { authDevice, linkDevice } from '../utils/kobont/devices';
import { getMetadata } from '../utils/kobont/metadata';
import { downloadBook as getProsaBook, uploadBook } from '../utils/prosa/books';
import { createApiKey, registerUser } from '../utils/prosa/users';

describe('Fetch book', () => {
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

    const getMetadataResponse = await getMetadata(uploadResponse.text, authResponse.body.AccessToken);
    expect(getMetadataResponse.status).toBe(200);

    const token = getMetadataResponse.body[0].DownloadUrls[0].Url.split('?token=')[1];
    const downloadBookResponse = await getBook(uploadResponse.text, token);
    expect(downloadBookResponse.status).toBe(200);

    let epub = path.join(BOOK_DIR, 'The_Great_Gatsby.epub');
    let originalSize = fs.statSync(epub).size;
    let downloadedSize = downloadBookResponse.body.length;

    expect(downloadedSize).toBeGreaterThan(originalSize);
  });

  test('Incorrect token', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadResponse.status).toBe(200);

    const uploadResponse2 = await uploadBook(userId, 'The_Great_Gatsby.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadResponse2.status).toBe(200);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Read'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const getMetadataResponse = await getMetadata(uploadResponse.text, authResponse.body.AccessToken);
    expect(getMetadataResponse.status).toBe(200);

    const token = getMetadataResponse.body[0].DownloadUrls[0].Url.split('?token=')[1];
    let downloadBookResponse = await getBook(uploadResponse.text, 'incorrect');
    expect(downloadBookResponse.status).toBe(403);
    expect(downloadBookResponse.body.message).toBe(INVALID_BOOK_TOKEN);

    downloadBookResponse = await getBook(uploadResponse.text);
    expect(downloadBookResponse.status).toBe(403);
    expect(downloadBookResponse.body.message).toBe(INVALID_BOOK_TOKEN);

    downloadBookResponse = await getBook(uploadResponse2.text, token);
    expect(downloadBookResponse.status).toBe(403);
    expect(downloadBookResponse.body.message).toBe(INVALID_BOOK_TOKEN);
  });
});

describe('Delete book', () => {
  test('Simple', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadResponse.status).toBe(200);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Read', 'Delete'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const getMetadataResponse = await getMetadata(uploadResponse.text, authResponse.body.AccessToken);
    expect(getMetadataResponse.status).toBe(200);

    const token = getMetadataResponse.body[0].DownloadUrls[0].Url.split('?token=')[1];
    const downloadBookResponse = await getBook(uploadResponse.text, token);
    expect(downloadBookResponse.status).toBe(200);

    const deleteBookResponse = await deleteBook(uploadResponse.text, authResponse.body.AccessToken);
    expect(deleteBookResponse.status).toBe(204);

    const getProsaBookResponse = await getProsaBook(uploadResponse.text, { apiKey: createApiKeyResponse.body.key });
    expect(getProsaBookResponse.status).toBe(404);
  });

  test('Non-existent book', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Read', 'Delete'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    // If book wasn't found, then we should still return 204 to avoid errors in the Kobo device
    const deleteBookResponse = await deleteBook('non-existent', authResponse.body.AccessToken);
    expect(deleteBookResponse.status).toBe(204);
  });

  test('No auth', async () => {
    const deleteBookResponse = await deleteBook('non-existent');
    expect(deleteBookResponse.status).toBe(401);
    expect(deleteBookResponse.body.message).toBe(UNAUTHENTICATED);
  });

  test('Not linked', async () => {
    const { response: authResponse } = await authDevice();
    expect(authResponse.status).toBe(200);

    const deleteBookResponse = await deleteBook('non-existent', authResponse.body.AccessToken);
    expect(deleteBookResponse.status).toBe(401);
    expect(deleteBookResponse.body.message).toBe(DEVICE_NOT_LINKED);
  });
});
