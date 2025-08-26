import { DEVICE_NOT_LINKED, UNAUTHENTICATED, wait } from '../utils/common';
import { authDevice, linkDevice } from '../utils/kobont/devices';
import { addBooksToShelf } from '../utils/kobont/shelves';
import { sync } from '../utils/kobont/sync';
import { deleteBook, uploadBook } from '../utils/prosa/books';
import { createShelf, deleteBookFromShelf, deleteShelf, updateShelf } from '../utils/prosa/shelves';
import { createApiKey, registerUser } from '../utils/prosa/users';

describe('Book syncing', () => {
  test('New book', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadBookResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadBookResponse.status).toBe(200);

    // Wait for cover and metadata to be extracted
    await wait(1);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Create', 'Read'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const syncResponse = await sync(undefined, authResponse.body.AccessToken);
    expect(syncResponse.status).toBe(200);
    expect(syncResponse.body).toHaveLength(1);
    expect(syncResponse.body[0]).toHaveProperty('NewEntitlement');
    expect(syncResponse.body[0].NewEntitlement).toHaveProperty('BookEntitlement');
    expect(syncResponse.body[0].NewEntitlement.BookEntitlement.Id).toEqual(uploadBookResponse.text);
    expect(syncResponse.body[0].NewEntitlement).toHaveProperty('BookMetadata');
    expect(syncResponse.body[0].NewEntitlement.BookMetadata.Title).toEqual("Alice's Adventures in Wonderland");
  });

  test('Deleted book', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadBookResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadBookResponse.status).toBe(200);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Create', 'Read'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    let syncResponse = await sync(undefined, authResponse.body.AccessToken);
    expect(syncResponse.status).toBe(200);
    expect(syncResponse.body).toHaveLength(1);
    expect(syncResponse.body[0]).toHaveProperty('NewEntitlement');

    const deleteBookResponse = await deleteBook(uploadBookResponse.text, { jwt: registerResponse.body.jwt_token });
    expect(deleteBookResponse.status).toBe(204);

    syncResponse = await sync(undefined, authResponse.body.AccessToken);
    expect(syncResponse.status).toBe(200);
    expect(syncResponse.body).toHaveLength(1);
    expect(syncResponse.body[0]).toHaveProperty('NewEntitlement');
    expect(syncResponse.body[0].NewEntitlement).toHaveProperty('BookEntitlement');
    expect(syncResponse.body[0].NewEntitlement.BookEntitlement.Id).toEqual(uploadBookResponse.text);
    expect(syncResponse.body[0].NewEntitlement.BookEntitlement.IsRemoved).toEqual(true);
    expect(syncResponse.body[0].NewEntitlement.BookEntitlement.IsHiddenFromArchive).toEqual(true);
  });
});

describe('Shelf syncing', () => {
  test('New shelf', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Create', 'Read'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const createShelfResponse = await createShelf('new-shelf', undefined, { jwt: registerResponse.body.jwt_token });
    expect(createShelfResponse.status).toBe(200);

    const syncResponse = await sync(undefined, authResponse.body.AccessToken);
    expect(syncResponse.status).toBe(200);
    expect(syncResponse.body).toHaveLength(1);
    expect(syncResponse.body[0]).toHaveProperty('NewTag');
    expect(syncResponse.body[0].NewTag).toHaveProperty('Tag');
    expect(syncResponse.body[0].NewTag.Tag.Id).toEqual(createShelfResponse.text);
    expect(syncResponse.body[0].NewTag.Tag.Name).toEqual('new-shelf');
  });

  test('Renamed shelf', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Create', 'Read'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const createShelfResponse = await createShelf('new-shelf', undefined, { jwt: registerResponse.body.jwt_token });
    expect(createShelfResponse.status).toBe(200);

    let syncResponse = await sync(undefined, authResponse.body.AccessToken);
    expect(syncResponse.status).toBe(200);
    expect(syncResponse.body).toHaveLength(1);
    expect(syncResponse.body[0]).toHaveProperty('NewTag');
    expect(syncResponse.body[0].NewTag).toHaveProperty('Tag');
    expect(syncResponse.body[0].NewTag.Tag.Id).toEqual(createShelfResponse.text);
    expect(syncResponse.body[0].NewTag.Tag.Name).toEqual('new-shelf');

    const updateShelfResponse = await updateShelf(createShelfResponse.text, 'new-name', { jwt: registerResponse.body.jwt_token });
    expect(updateShelfResponse.status).toBe(204);

    syncResponse = await sync(undefined, authResponse.body.AccessToken);
    expect(syncResponse.status).toBe(200);
    expect(syncResponse.body).toHaveLength(1);
    expect(syncResponse.body[0]).toHaveProperty('NewTag');
    expect(syncResponse.body[0].NewTag).toHaveProperty('Tag');
    expect(syncResponse.body[0].NewTag.Tag.Id).toEqual(createShelfResponse.text);
    expect(syncResponse.body[0].NewTag.Tag.Name).toEqual('new-name');
  });

  test('Add book to shelf', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadBookResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadBookResponse.status).toBe(200);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Create', 'Read', 'Update'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const createShelfResponse = await createShelf('new-shelf', undefined, { jwt: registerResponse.body.jwt_token });
    expect(createShelfResponse.status).toBe(200);

    const addBookToShelfResponse = await addBooksToShelf(createShelfResponse.text, [uploadBookResponse.text], authResponse.body.AccessToken);
    expect(addBookToShelfResponse.status).toBe(201);

    const syncResponse = await sync(undefined, authResponse.body.AccessToken);
    expect(syncResponse.status).toBe(200);
    expect(syncResponse.body).toHaveLength(2);
    expect(syncResponse.body[1]).toHaveProperty('NewTag');
    expect(syncResponse.body[1].NewTag).toHaveProperty('Tag');
    expect(syncResponse.body[1].NewTag.Tag.Id).toEqual(createShelfResponse.text);
    expect(syncResponse.body[1].NewTag.Tag.Name).toEqual('new-shelf');
    expect(syncResponse.body[1].NewTag.Tag.Items).toHaveLength(1);
    expect(syncResponse.body[1].NewTag.Tag.Items[0].RevisionId).toEqual(uploadBookResponse.text);
  });

  test('Remove book from shelf', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadBookResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadBookResponse.status).toBe(200);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Create', 'Read', 'Update'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const createShelfResponse = await createShelf('new-shelf', undefined, { jwt: registerResponse.body.jwt_token });
    expect(createShelfResponse.status).toBe(200);

    const addBookToShelfResponse = await addBooksToShelf(createShelfResponse.text, [uploadBookResponse.text], authResponse.body.AccessToken);
    expect(addBookToShelfResponse.status).toBe(201);

    let syncResponse = await sync(undefined, authResponse.body.AccessToken);
    expect(syncResponse.status).toBe(200);
    expect(syncResponse.body).toHaveLength(2);
    expect(syncResponse.body[1]).toHaveProperty('NewTag');
    expect(syncResponse.body[1].NewTag).toHaveProperty('Tag');
    expect(syncResponse.body[1].NewTag.Tag.Id).toEqual(createShelfResponse.text);
    expect(syncResponse.body[1].NewTag.Tag.Name).toEqual('new-shelf');
    expect(syncResponse.body[1].NewTag.Tag.Items).toHaveLength(1);
    expect(syncResponse.body[1].NewTag.Tag.Items[0].RevisionId).toEqual(uploadBookResponse.text);

    const removeBookFromShelfResponse = await deleteBookFromShelf(createShelfResponse.text, uploadBookResponse.text, { jwt: registerResponse.body.jwt_token });
    expect(removeBookFromShelfResponse.status).toBe(204);

    syncResponse = await sync(undefined, authResponse.body.AccessToken);
    expect(syncResponse.status).toBe(200);
    expect(syncResponse.body).toHaveLength(2);
    expect(syncResponse.body[1]).toHaveProperty('NewTag');
    expect(syncResponse.body[1].NewTag).toHaveProperty('Tag');
    expect(syncResponse.body[1].NewTag.Tag.Id).toEqual(createShelfResponse.text);
    expect(syncResponse.body[1].NewTag.Tag.Name).toEqual('new-shelf');
    expect(syncResponse.body[1].NewTag.Tag.Items).toHaveLength(0);
  });

  test('Delete shelf', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Create', 'Read', 'Update'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    const createShelfResponse = await createShelf('new-shelf', undefined, { jwt: registerResponse.body.jwt_token });
    expect(createShelfResponse.status).toBe(200);

    let syncResponse = await sync(undefined, authResponse.body.AccessToken);
    expect(syncResponse.status).toBe(200);
    expect(syncResponse.body).toHaveLength(1);
    expect(syncResponse.body[0]).toHaveProperty('NewTag');
    expect(syncResponse.body[0].NewTag).toHaveProperty('Tag');
    expect(syncResponse.body[0].NewTag.Tag.Id).toEqual(createShelfResponse.text);
    expect(syncResponse.body[0].NewTag.Tag.Name).toEqual('new-shelf');
    expect(syncResponse.body[0].NewTag.Tag.Items).toHaveLength(0);

    const deleteShelfResponse = await deleteShelf(createShelfResponse.text, { jwt: registerResponse.body.jwt_token });
    expect(deleteShelfResponse.status).toBe(204);

    syncResponse = await sync(undefined, authResponse.body.AccessToken);
    expect(syncResponse.status).toBe(200);
    expect(syncResponse.body).toHaveLength(1);
    expect(syncResponse.body[0]).toHaveProperty('DeletedTag');
    expect(syncResponse.body[0].DeletedTag).toHaveProperty('Tag');
    expect(syncResponse.body[0].DeletedTag.Tag.Id).toEqual(createShelfResponse.text);
  });
});

describe('Errors', () => {
  test('No auth', async () => {
    const syncResponse = await sync();
    expect(syncResponse.status).toBe(401);
    expect(syncResponse.body.message).toBe(UNAUTHENTICATED);
  });

  test('Not linked', async () => {
    const { response: authResponse } = await authDevice();
    expect(authResponse.status).toBe(200);

    const syncResponse = await sync(undefined, authResponse.body.AccessToken);
    expect(syncResponse.status).toBe(401);
    expect(syncResponse.body.message).toBe(DEVICE_NOT_LINKED);
  });
});
