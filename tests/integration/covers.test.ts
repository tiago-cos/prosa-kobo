import { getCover, INVALID_COVER_TOKEN } from '../utils/kobont/covers';
import { authDevice, linkDevice, unlinkDevice } from '../utils/kobont/devices';
import { getMetadata } from '../utils/kobont/metadata';
import { deleteBook as deleteProsaBook, uploadBook } from '../utils/prosa/books';
import { deleteCover as deleteProsaCover, getCover as getProsaCover } from '../utils/prosa/covers';
import { createApiKey, registerUser } from '../utils/prosa/users';

describe('Fetch image', () => {
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

    const token = getMetadataResponse.body[0].CoverImageId.split('?token=')[1];
    const downloadCoverResponse = await getCover(uploadResponse.text, token);
    expect(downloadCoverResponse.status).toBe(200);

    const expectedResponse = await getProsaCover(uploadResponse.text, { jwt: registerResponse.body.jwt_token });
    expect(expectedResponse.status).toBe(200);

    expect(downloadCoverResponse.body).toEqual(expectedResponse.body);
  });

  test('Non-existent book', async () => {
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

    const deleteBookResponse = await deleteProsaBook(uploadResponse.text, { apiKey: createApiKeyResponse.body.key });
    expect(deleteBookResponse.status).toBe(204);

    const token = getMetadataResponse.body[0].CoverImageId.split('?token=')[1];
    const downloadBookResponse = await getCover(uploadResponse.text, token);
    expect(downloadBookResponse.status).toBe(404);
  });

  test('Non-existent cover', async () => {
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

    const deleteCoverResponse = await deleteProsaCover(uploadResponse.text, { apiKey: createApiKeyResponse.body.key });
    expect(deleteCoverResponse.status).toBe(204);

    const token = getMetadataResponse.body[0].CoverImageId.split('?token=')[1];
    const downloadBookResponse = await getCover(uploadResponse.text, token);
    expect(downloadBookResponse.status).toBe(404);
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

    const token = getMetadataResponse.body[0].CoverImageId.split('?token=')[1];
    let downloadCoverResponse = await getCover(uploadResponse.text, 'incorrect');
    expect(downloadCoverResponse.status).toBe(403);
    expect(downloadCoverResponse.body.message).toBe(INVALID_COVER_TOKEN);

    downloadCoverResponse = await getCover(uploadResponse.text);
    expect(downloadCoverResponse.status).toBe(403);
    expect(downloadCoverResponse.body.message).toBe(INVALID_COVER_TOKEN);

    downloadCoverResponse = await getCover(uploadResponse2.text, token);
    expect(downloadCoverResponse.status).toBe(403);
    expect(downloadCoverResponse.body.message).toBe(INVALID_COVER_TOKEN);

    downloadCoverResponse = await getCover('non-existent', token);
    expect(downloadCoverResponse.status).toBe(403);
    expect(downloadCoverResponse.body.message).toBe(INVALID_COVER_TOKEN);
  });

  test('Wrong capabilities', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadResponse.status).toBe(200);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Read'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const createApiKeyResponse2 = await createApiKey(userId, 'Test Key', ['Create', 'Update', 'Delete'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse2.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    let linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const getMetadataResponse = await getMetadata(uploadResponse.text, authResponse.body.AccessToken);
    expect(getMetadataResponse.status).toBe(200);

    const unlinkResponse = await unlinkDevice(deviceId, createApiKeyResponse.body.key);
    expect(unlinkResponse.status).toBe(200);

    linkResponse = await linkDevice(deviceId, createApiKeyResponse2.body.key);
    expect(linkResponse.status).toBe(200);

    const token = getMetadataResponse.body[0].CoverImageId.split('?token=')[1];
    const downloadCoverResponse = await getCover(uploadResponse.text, token);
    expect(downloadCoverResponse.status).toBe(403);
  });
});
