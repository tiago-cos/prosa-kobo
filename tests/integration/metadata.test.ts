import { DEVICE_NOT_LINKED, UNAUTHENTICATED, wait } from '../utils/common';
import { authDevice, linkDevice } from '../utils/kobont/devices';
import { generateAliceMetadata, getMetadata, normalizeMetadata } from '../utils/kobont/metadata';
import { uploadBook } from '../utils/prosa/books';
import { createApiKey, registerUser } from '../utils/prosa/users';

describe('Metadata', () => {
  test('Simple', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadResponse.status).toBe(200);

    // Wait for metadata to be extracted
    await wait(0.5);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Read'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const expectedResponse = await generateAliceMetadata(uploadResponse.text);

    const getMetadataResponse = await getMetadata(uploadResponse.text, authResponse.body.AccessToken);
    expect(getMetadataResponse.status).toBe(200);
    expect(normalizeMetadata(getMetadataResponse.body[0])).toEqual(expectedResponse[0]);
  });

  test('Non existent book', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    // Wait for metadata to be extracted
    await wait(0.5);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Read'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const getMetadataResponse = await getMetadata('non-existent', authResponse.body.AccessToken);
    expect(getMetadataResponse.status).toBe(404);
  });

  test('No auth', async () => {
    const getMetadataResponse = await getMetadata('non-existent');
    expect(getMetadataResponse.status).toBe(401);
    expect(getMetadataResponse.body.message).toBe(UNAUTHENTICATED);
  });

  test('Not linked', async () => {
    const { response: authResponse } = await authDevice();
    expect(authResponse.status).toBe(200);

    const getMetadataResponse = await getMetadata('non-existent', authResponse.body.AccessToken);
    expect(getMetadataResponse.status).toBe(401);
    expect(getMetadataResponse.body.message).toBe(DEVICE_NOT_LINKED);
  });
});
