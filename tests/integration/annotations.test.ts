import { DEVICE_NOT_LINKED, UNAUTHENTICATED } from '../utils/common';
import { addAnnotation, annotationRequest, checkForChanges, deleteAnnotation, getAnnotations, updateAnnotation } from '../utils/kobont/annotations';
import { authDevice, linkDevice } from '../utils/kobont/devices';
import { sync } from '../utils/kobont/sync';
import { addAnnotation as addProsaAnnotation, ALICE_NOTE, listAnnotations as listProsaAnnotations } from '../utils/prosa/annotations';
import { uploadBook } from '../utils/prosa/books';
import { createApiKey, registerUser } from '../utils/prosa/users';

describe('Check for changes', () => {
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

    let checkForChangesResponse = await checkForChanges([{ ContentId: uploadResponse.text, etag: 'etag' }]);
    expect(checkForChangesResponse.status).toBe(200);
    expect(checkForChangesResponse.body).toEqual([]);

    const addAnnotationResponse = await addProsaAnnotation(uploadResponse.text, ALICE_NOTE, { jwt: registerResponse.body.jwt_token });
    expect(addAnnotationResponse.status).toBe(200);

    // Won't detect changes until sync
    checkForChangesResponse = await checkForChanges([{ ContentId: uploadResponse.text, etag: 'etag' }]);
    expect(checkForChangesResponse.status).toBe(200);
    expect(checkForChangesResponse.body).toEqual([]);

    const syncResponse = await sync(undefined, authResponse.body.AccessToken);
    expect(syncResponse.status).toBe(200);

    // Should have been detected now
    checkForChangesResponse = await checkForChanges([
      { ContentId: uploadResponse.text, etag: 'etag' },
      { ContentId: 'non-existent', etag: 'etag2' }
    ]);
    expect(checkForChangesResponse.status).toBe(200);
    expect(checkForChangesResponse.body).toEqual([uploadResponse.text]);
  });
});

describe('Get annotations', () => {
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

    let getAnnotationsResponse = await getAnnotations(uploadResponse.text, authResponse.body.AccessToken);
    expect(getAnnotationsResponse.status).toBe(200);
    expect(getAnnotationsResponse.body).toEqual({ annotations: [], nextPageOffsetToken: null });

    const addAnnotationResponse = await addProsaAnnotation(uploadResponse.text, ALICE_NOTE, { jwt: registerResponse.body.jwt_token });
    expect(addAnnotationResponse.status).toBe(200);

    getAnnotationsResponse = await getAnnotations(uploadResponse.text, authResponse.body.AccessToken);
    expect(getAnnotationsResponse.status).toBe(200);
    expect(getAnnotationsResponse.body.annotations).toHaveLength(1);
    expect(getAnnotationsResponse.body.annotations[0].noteText).toEqual(ALICE_NOTE.note);
    expect(getAnnotationsResponse.headers['ETag']).not.toBeNull();
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

    let getAnnotationsResponse = await getAnnotations('non-existent', authResponse.body.AccessToken);
    expect(getAnnotationsResponse.status).toBe(404);
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

    let getAnnotationsResponse = await getAnnotations(uploadResponse.text, authResponse.body.AccessToken);
    expect(getAnnotationsResponse.status).toBe(403);
  });

  test('No auth', async () => {
    let getAnnotationsResponse = await getAnnotations('non-existent');
    expect(getAnnotationsResponse.status).toBe(401);
    expect(getAnnotationsResponse.body.message).toBe(UNAUTHENTICATED);
  });

  test('Not linked', async () => {
    const { response: authResponse } = await authDevice();
    expect(authResponse.status).toBe(200);

    const getAnnotationsResponse = await getAnnotations('non-existent', authResponse.body.AccessToken);
    expect(getAnnotationsResponse.status).toBe(401);
    expect(getAnnotationsResponse.body.message).toBe(DEVICE_NOT_LINKED);
  });
});

describe('Patch annotations', () => {
  test('Add annotation', async () => {
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

    let getAnnotationsResponse = await getAnnotations(uploadResponse.text, authResponse.body.AccessToken);
    expect(getAnnotationsResponse.status).toBe(200);
    expect(getAnnotationsResponse.body).toEqual({ annotations: [], nextPageOffsetToken: null });

    const addAnnotationRequest: annotationRequest = {
      startChar: 7,
      startTag: 'kobo.74.1',
      endChar: 4,
      endTag: 'kobo.74.2',
      source: 'OEBPS/229714655232534212_11-h-10.htm.xhtml',
      note: 'I loved this part!'
    };

    const addAnnotationResponse = await addAnnotation(uploadResponse.text, addAnnotationRequest, authResponse.body.AccessToken);
    expect(addAnnotationResponse.status).toBe(204);

    getAnnotationsResponse = await getAnnotations(uploadResponse.text, authResponse.body.AccessToken);
    expect(getAnnotationsResponse.status).toBe(200);
    expect(getAnnotationsResponse.body.annotations).toHaveLength(1);
    expect(getAnnotationsResponse.body.annotations[0].noteText).toEqual(ALICE_NOTE.note);
    expect(getAnnotationsResponse.headers['ETag']).not.toBeNull();
  });

  test('Update annotation', async () => {
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

    let getAnnotationsResponse = await getAnnotations(uploadResponse.text, authResponse.body.AccessToken);
    expect(getAnnotationsResponse.status).toBe(200);
    expect(getAnnotationsResponse.body).toEqual({ annotations: [], nextPageOffsetToken: null });

    let annotationRequest: annotationRequest = {
      startChar: 7,
      startTag: 'kobo.74.1',
      endChar: 4,
      endTag: 'kobo.74.2',
      source: 'OEBPS/229714655232534212_11-h-10.htm.xhtml',
      note: 'I loved this part!'
    };

    let addAnnotationResponse = await addAnnotation(uploadResponse.text, annotationRequest, authResponse.body.AccessToken);
    expect(addAnnotationResponse.status).toBe(204);

    const listAnnotationsResponse = await listProsaAnnotations(uploadResponse.text, { jwt: registerResponse.body.jwt_token });
    expect(listAnnotationsResponse.status).toBe(200);

    const annotationId = listAnnotationsResponse.body[0];
    annotationRequest.note = 'I hated this part!';

    const updateAnnotationResponse = await updateAnnotation(uploadResponse.text, annotationId, annotationRequest, authResponse.body.AccessToken);
    expect(updateAnnotationResponse.status).toBe(204);

    getAnnotationsResponse = await getAnnotations(uploadResponse.text, authResponse.body.AccessToken);
    expect(getAnnotationsResponse.status).toBe(200);
    expect(getAnnotationsResponse.body.annotations).toHaveLength(1);
    expect(getAnnotationsResponse.body.annotations[0].noteText).toEqual('I hated this part!');
    expect(getAnnotationsResponse.headers['ETag']).not.toBeNull();
  });

  test('Delete annotation', async () => {
    const { response: registerResponse } = await registerUser();
    expect(registerResponse.status).toBe(200);
    const userId = registerResponse.body.user_id;

    const uploadResponse = await uploadBook(userId, 'Alices_Adventures_in_Wonderland.epub', { jwt: registerResponse.body.jwt_token });
    expect(uploadResponse.status).toBe(200);

    const createApiKeyResponse = await createApiKey(userId, 'Test Key', ['Read', 'Update', 'Delete'], undefined, { jwt: registerResponse.body.jwt_token });
    expect(createApiKeyResponse.status).toBe(200);

    const { response: authResponse, deviceId } = await authDevice();
    expect(authResponse.status).toBe(200);

    const linkResponse = await linkDevice(deviceId, createApiKeyResponse.body.key);
    expect(linkResponse.status).toBe(200);

    let getAnnotationsResponse = await getAnnotations(uploadResponse.text, authResponse.body.AccessToken);
    expect(getAnnotationsResponse.status).toBe(200);
    expect(getAnnotationsResponse.body).toEqual({ annotations: [], nextPageOffsetToken: null });

    let annotationRequest: annotationRequest = {
      startChar: 7,
      startTag: 'kobo.74.1',
      endChar: 4,
      endTag: 'kobo.74.2',
      source: 'OEBPS/229714655232534212_11-h-10.htm.xhtml',
      note: 'I loved this part!'
    };

    let addAnnotationResponse = await addAnnotation(uploadResponse.text, annotationRequest, authResponse.body.AccessToken);
    expect(addAnnotationResponse.status).toBe(204);

    const listAnnotationsResponse = await listProsaAnnotations(uploadResponse.text, { jwt: registerResponse.body.jwt_token });
    expect(listAnnotationsResponse.status).toBe(200);
    expect(listAnnotationsResponse.body).toHaveLength(1);

    const annotationId = listAnnotationsResponse.body[0];
    const deleteAnnotationResponse = await deleteAnnotation(uploadResponse.text, annotationId, authResponse.body.AccessToken);
    expect(deleteAnnotationResponse.status).toBe(204);

    getAnnotationsResponse = await getAnnotations(uploadResponse.text, authResponse.body.AccessToken);
    expect(getAnnotationsResponse.status).toBe(200);
    expect(getAnnotationsResponse.body.annotations).toHaveLength(0);
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

    const addAnnotationRequest: annotationRequest = {
      startChar: 7,
      startTag: 'kobo.74.1',
      endChar: 4,
      endTag: 'kobo.74.2',
      source: 'OEBPS/229714655232534212_11-h-10.htm.xhtml',
      note: 'I loved this part!'
    };

    const addAnnotationResponse = await addAnnotation('non-existent', addAnnotationRequest, authResponse.body.AccessToken);
    expect(addAnnotationResponse.status).toBe(404);
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

    const addAnnotationRequest: annotationRequest = {
      startChar: 7,
      startTag: 'kobo.74.1',
      endChar: 4,
      endTag: 'kobo.74.2',
      source: 'OEBPS/229714655232534212_11-h-10.htm.xhtml',
      note: 'I loved this part!'
    };

    const addAnnotationResponse = await addAnnotation(uploadResponse.text, addAnnotationRequest, authResponse.body.AccessToken);
    expect(addAnnotationResponse.status).toBe(403);
  });

  test('No auth', async () => {
    const addAnnotationRequest: annotationRequest = {
      startChar: 7,
      startTag: 'kobo.74.1',
      endChar: 4,
      endTag: 'kobo.74.2',
      source: 'OEBPS/229714655232534212_11-h-10.htm.xhtml',
      note: 'I loved this part!'
    };

    const addAnnotationResponse = await addAnnotation('non-existent', addAnnotationRequest);
    expect(addAnnotationResponse.status).toBe(401);
    expect(addAnnotationResponse.body.message).toBe(UNAUTHENTICATED);
  });

  test('Not linked', async () => {
    const { response: authResponse } = await authDevice();
    expect(authResponse.status).toBe(200);

    const addAnnotationRequest: annotationRequest = {
      startChar: 7,
      startTag: 'kobo.74.1',
      endChar: 4,
      endTag: 'kobo.74.2',
      source: 'OEBPS/229714655232534212_11-h-10.htm.xhtml',
      note: 'I loved this part!'
    };

    const addAnnotationResponse = await addAnnotation('non-existent', addAnnotationRequest, authResponse.body.AccessToken);
    expect(addAnnotationResponse.status).toBe(401);
    expect(addAnnotationResponse.body.message).toBe(DEVICE_NOT_LINKED);
  });
});
