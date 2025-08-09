import { DEVICE_NOT_LINKED, UNAUTHENTICATED } from '../utils/common';
import { authDevice, linkDevice, unlinkDevice } from '../utils/kobont/devices';
import { addBooksToShelf, createShelf, deleteBooksFromShelf, deleteShelf, renameShelf } from '../utils/kobont/shelves';
import { uploadBook } from '../utils/prosa/books';
import { getShelfMetadata as getProsaShelfMetadata, listBooksFromShelf } from '../utils/prosa/shelves';
import { createApiKey, registerUser } from '../utils/prosa/users';

describe('Create shelf', () => {
  test('Simple', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Create'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const createShelfResponse = await createShelf('new-shelf', authResponse.body.AccessToken);
    expect(createShelfResponse.status).toBe(201);

    const getShelfMetadataResponse = await getProsaShelfMetadata(createShelfResponse.text, { jwt: registerResponse.body.jwt_token });
    expect(getShelfMetadataResponse.status).toBe(200);
    expect(getShelfMetadataResponse.body.name).toEqual('new-shelf');
  });

  test('Wrong capabilities', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Delete'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const createShelfResponse = await createShelf('new-shelf', authResponse.body.AccessToken);
    expect(createShelfResponse.status).toBe(403);
  });

  test('No auth', async () => {
    const createShelfResponse = await createShelf('new-shelf');
    expect(createShelfResponse.status).toBe(401);
    expect(createShelfResponse.body.message).toBe(UNAUTHENTICATED);
  });

  test('Not linked', async () => {
    const { response: authResponse } = await authDevice();
    expect(authResponse.status).toBe(200);

    const createShelfResponse = await createShelf('new-shelf', authResponse.body.AccessToken);
    expect(createShelfResponse.status).toBe(401);
    expect(createShelfResponse.body.message).toBe(DEVICE_NOT_LINKED);
  });
});

describe('Delete shelf', () => {
  test('Simple', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Create', 'Delete'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const createShelfResponse = await createShelf('new-shelf', authResponse.body.AccessToken);
    expect(createShelfResponse.status).toBe(201);

    let getShelfMetadataResponse = await getProsaShelfMetadata(createShelfResponse.text, { jwt: registerResponse.body.jwt_token });
    expect(getShelfMetadataResponse.status).toBe(200);
    expect(getShelfMetadataResponse.body.name).toEqual('new-shelf');

    const deleteShelfResponse = await deleteShelf(createShelfResponse.text, authResponse.body.AccessToken);
    expect(deleteShelfResponse.status).toBe(200);

    getShelfMetadataResponse = await getProsaShelfMetadata(createShelfResponse.text, { jwt: registerResponse.body.jwt_token });
    expect(getShelfMetadataResponse.status).toBe(404);
  });

  test('Non-existent shelf', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Create', 'Delete'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const deleteShelfResponse = await deleteShelf('non-existent', authResponse.body.AccessToken);
    expect(deleteShelfResponse.status).toBe(200);
  });

  test('Wrong capabilities', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Create'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const createShelfResponse = await createShelf('new-shelf', authResponse.body.AccessToken);
    expect(createShelfResponse.status).toBe(201);

    let getShelfMetadataResponse = await getProsaShelfMetadata(createShelfResponse.text, { jwt: registerResponse.body.jwt_token });
    expect(getShelfMetadataResponse.status).toBe(200);
    expect(getShelfMetadataResponse.body.name).toEqual('new-shelf');

    const deleteShelfResponse = await deleteShelf(createShelfResponse.text, authResponse.body.AccessToken);
    expect(deleteShelfResponse.status).toBe(403);
  });

  test('No auth', async () => {
    const deleteShelfResponse = await deleteShelf('non-existent');
    expect(deleteShelfResponse.status).toBe(401);
    expect(deleteShelfResponse.body.message).toBe(UNAUTHENTICATED);
  });

  test('Not linked', async () => {
    const { response: authResponse } = await authDevice();
    expect(authResponse.status).toBe(200);

    const deleteShelfResponse = await deleteShelf('non-existent', authResponse.body.AccessToken);
    expect(deleteShelfResponse.status).toBe(401);
    expect(deleteShelfResponse.body.message).toBe(DEVICE_NOT_LINKED);
  });
});

describe('Rename shelf', () => {
  test('Simple', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Create', 'Update'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const createShelfResponse = await createShelf('new-shelf', authResponse.body.AccessToken);
    expect(createShelfResponse.status).toBe(201);

    let getShelfMetadataResponse = await getProsaShelfMetadata(createShelfResponse.text, { jwt: registerResponse.body.jwt_token });
    expect(getShelfMetadataResponse.status).toBe(200);
    expect(getShelfMetadataResponse.body.name).toEqual('new-shelf');

    const renameShelfResponse = await renameShelf(createShelfResponse.text, 'new-name', authResponse.body.AccessToken);
    expect(renameShelfResponse.status).toBe(200);

    getShelfMetadataResponse = await getProsaShelfMetadata(createShelfResponse.text, { jwt: registerResponse.body.jwt_token });
    expect(getShelfMetadataResponse.status).toBe(200);
    expect(getShelfMetadataResponse.body.name).toEqual('new-name');
  });

  test('Non-existent shelf', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Create', 'Update'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const deleteShelfResponse = await renameShelf('non-existent', 'new-name', authResponse.body.AccessToken);
    expect(deleteShelfResponse.status).toBe(404);
  });

  test('Wrong capabilities', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Create'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const createShelfResponse = await createShelf('new-shelf', authResponse.body.AccessToken);
    expect(createShelfResponse.status).toBe(201);

    let getShelfMetadataResponse = await getProsaShelfMetadata(createShelfResponse.text, { jwt: registerResponse.body.jwt_token });
    expect(getShelfMetadataResponse.status).toBe(200);
    expect(getShelfMetadataResponse.body.name).toEqual('new-shelf');

    const renameShelfResponse = await renameShelf(createShelfResponse.text, 'new-name', authResponse.body.AccessToken);
    expect(renameShelfResponse.status).toBe(403);
  });

  test('No auth', async () => {
    const renameShelfResponse = await renameShelf('non-existent', 'new-name');
    expect(renameShelfResponse.status).toBe(401);
    expect(renameShelfResponse.body.message).toBe(UNAUTHENTICATED);
  });

  test('Not linked', async () => {
    const { response: authResponse } = await authDevice();
    expect(authResponse.status).toBe(200);

    const renameShelfResponse = await renameShelf('non-existent', 'new-name', authResponse.body.AccessToken);
    expect(renameShelfResponse.status).toBe(401);
    expect(renameShelfResponse.body.message).toBe(DEVICE_NOT_LINKED);
  });
});

describe('Add book to shelf', () => {
  test('Simple', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadBookResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadBookResponse.status).toBe(200);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Create', 'Update'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const createShelfResponse = await createShelf('new-shelf', authResponse.body.AccessToken);
    expect(createShelfResponse.status).toBe(201);

    const addBookToShelfResponse = await addBooksToShelf(createShelfResponse.text, [uploadBookResponse.text], authResponse.body.AccessToken);
    expect(addBookToShelfResponse.status).toBe(201);

    const listShelfBooksResponse = await listBooksFromShelf(createShelfResponse.text, { jwt: registerResponse.body.jwt_token });
    expect(listShelfBooksResponse.status).toBe(200);
    expect(listShelfBooksResponse.body).toEqual([uploadBookResponse.text]);
  });

  test('Wrong capabilities', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadBookResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadBookResponse.status).toBe(200);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Create'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const createShelfResponse = await createShelf('new-shelf', authResponse.body.AccessToken);
    expect(createShelfResponse.status).toBe(201);

    const addBookToShelfResponse = await addBooksToShelf(createShelfResponse.text, [uploadBookResponse.text], authResponse.body.AccessToken);
    expect(addBookToShelfResponse.status).toBe(403);
  });

  test('No auth', async () => {
    const addBookToShelfResponse = await addBooksToShelf('non-existent', ['non-existent']);
    expect(addBookToShelfResponse.status).toBe(401);
    expect(addBookToShelfResponse.body.message).toBe(UNAUTHENTICATED);
  });

  test('Not linked', async () => {
    const { response: authResponse } = await authDevice();
    expect(authResponse.status).toBe(200);

    const addBookToShelfResponse = await addBooksToShelf('non-existent', ['non-existent'], authResponse.body.AccessToken);
    expect(addBookToShelfResponse.status).toBe(401);
    expect(addBookToShelfResponse.body.message).toBe(DEVICE_NOT_LINKED);
  });
});

describe('Delete book from shelf', () => {
  test('Simple', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadBookResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadBookResponse.status).toBe(200);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Create', 'Update'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const createShelfResponse = await createShelf('new-shelf', authResponse.body.AccessToken);
    expect(createShelfResponse.status).toBe(201);

    const addBookToShelfResponse = await addBooksToShelf(createShelfResponse.text, [uploadBookResponse.text], authResponse.body.AccessToken);
    expect(addBookToShelfResponse.status).toBe(201);

    let listShelfBooksResponse = await listBooksFromShelf(createShelfResponse.text, { jwt: registerResponse.body.jwt_token });
    expect(listShelfBooksResponse.status).toBe(200);
    expect(listShelfBooksResponse.body).toEqual([uploadBookResponse.text]);

    const deleteBookFromShelfResponse = await deleteBooksFromShelf(createShelfResponse.text, [uploadBookResponse.text], authResponse.body.AccessToken);
    expect(deleteBookFromShelfResponse.status).toBe(200);

    listShelfBooksResponse = await listBooksFromShelf(createShelfResponse.text, { jwt: registerResponse.body.jwt_token });
    expect(listShelfBooksResponse.status).toBe(200);
    expect(listShelfBooksResponse.body).toEqual([]);
  });

  test('Non-existent book', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Create', 'Update'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const createShelfResponse = await createShelf('new-shelf', authResponse.body.AccessToken);
    expect(createShelfResponse.status).toBe(201);

    const deleteBookFromShelfResponse = await deleteBooksFromShelf(createShelfResponse.text, ['non-existent'], authResponse.body.AccessToken);
    expect(deleteBookFromShelfResponse.status).toBe(200);
  });

  test('Wrong capabilities', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadBookResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadBookResponse.status).toBe(200);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Create', 'Update'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const createApiKeyResponse2 = await createApiKey(userId, 'Test Key', ['Create', 'Delete'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse2.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    let linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const createShelfResponse = await createShelf('new-shelf', authResponse.body.AccessToken);
    expect(createShelfResponse.status).toBe(201);

    const addBookToShelfResponse = await addBooksToShelf(createShelfResponse.text, [uploadBookResponse.text], authResponse.body.AccessToken);
    expect(addBookToShelfResponse.status).toBe(201);

    let listShelfBooksResponse = await listBooksFromShelf(createShelfResponse.text, { jwt: registerResponse.body.jwt_token });
    expect(listShelfBooksResponse.status).toBe(200);
    expect(listShelfBooksResponse.body).toEqual([uploadBookResponse.text]);

    const unlinkResponse = await unlinkDevice(deviceId, createApiKeyResponse.body.key);
    expect(unlinkResponse.status).toBe(200);

    linkResponse = await linkDevice(deviceId, createApiKeyResponse2.body.key);
    expect(linkResponse.status).toBe(200);

    const deleteBookFromShelfResponse = await deleteBooksFromShelf(createShelfResponse.text, [uploadBookResponse.text], authResponse.body.AccessToken);
    expect(deleteBookFromShelfResponse.status).toBe(403);
  });

  test('No auth', async () => {
    const deleteBookFromShelfResponse = await deleteBooksFromShelf('non-existent', ['non-existent']);
    expect(deleteBookFromShelfResponse.status).toBe(401);
    expect(deleteBookFromShelfResponse.body.message).toBe(UNAUTHENTICATED);
  });

  test('Not linked', async () => {
    const { response: authResponse } = await authDevice();
    expect(authResponse.status).toBe(200);

    const deleteBooksFromShelfResponse = await deleteBooksFromShelf('non-existent', ['non-existent'], authResponse.body.AccessToken);
    expect(deleteBooksFromShelfResponse.status).toBe(401);
    expect(deleteBooksFromShelfResponse.body.message).toBe(DEVICE_NOT_LINKED);
  });
});
