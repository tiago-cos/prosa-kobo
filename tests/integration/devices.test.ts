import { INVALID_TOKEN, randomString } from "../utils/common";
import { authDevice, authRefreshDevice, DEVICE_NOT_FOUND, getLinkedDevices, getUnlinkedDevices, INVALID_API_KEY, linkDevice, MISSING_API_KEY, unlinkDevice } from "../utils/devices";

describe("Device auth", () => {
    test.concurrent("Simple", async () => {
        const { response, userKey } = await authDevice();
        expect(response.status).toBe(200);
        expect(response.body.UserKey).toEqual(userKey);

        const refreshResponse = await authRefreshDevice(response.body.RefreshToken);
        expect(refreshResponse.status).toBe(200);
    });

    test.concurrent("Invalid refresh token", async () => {
        const refreshResponse = await authRefreshDevice("invalid");
        expect(refreshResponse.status).toBe(401);
        expect(refreshResponse.body.message).toBe(INVALID_TOKEN);
    });
});

describe("Device linking", () => {
    test.concurrent("Complete", async () => {
        const apiKey = randomString(16);

        let linkedResponse = await getLinkedDevices(apiKey);
        expect(linkedResponse.status).toBe(200);
        expect(linkedResponse.body).toEqual([]);

        const { response: authResponse, deviceId } = await authDevice();
        expect(authResponse.status).toBe(200);

        let unlinkedResponse = await getUnlinkedDevices();
        expect(unlinkedResponse.status).toBe(200);
        expect(unlinkedResponse.body).toEqual(expect.arrayContaining([expect.objectContaining({ device_id: deviceId })]));

        const linkResponse = await linkDevice(deviceId, apiKey);
        expect(linkResponse.status).toBe(200);

        linkedResponse = await getLinkedDevices(apiKey);
        expect(linkedResponse.status).toBe(200);
        expect(linkedResponse.body).toEqual([deviceId]);

        unlinkedResponse = await getUnlinkedDevices();
        expect(unlinkedResponse.status).toBe(200);
        expect(unlinkedResponse.body).not.toEqual(expect.arrayContaining([expect.objectContaining({ device_id: deviceId })]));

        const unlinkResponse = await unlinkDevice(deviceId, apiKey);
        expect(unlinkResponse.status).toBe(200);

        linkedResponse = await getLinkedDevices(apiKey);
        expect(linkedResponse.status).toBe(200);
        expect(linkedResponse.body).toEqual([]);

        unlinkedResponse = await getUnlinkedDevices();
        expect(unlinkedResponse.status).toBe(200);
        expect(unlinkedResponse.body).toEqual(expect.arrayContaining([expect.objectContaining({ device_id: deviceId })]));
    });

    test.concurrent("Get linked devices without api key", async () => {
        let linkedResponse = await getLinkedDevices();
        expect(linkedResponse.status).toBe(400);
        expect(linkedResponse.body.message).toBe(MISSING_API_KEY);
    });

    test.concurrent("Get linked devices invalid api key", async () => {
        let linkedResponse = await getLinkedDevices("this is an invalid key");
        expect(linkedResponse.status).toBe(400);
        expect(linkedResponse.body.message).toBe(INVALID_API_KEY);
    });

    test.concurrent("Link non-existent device", async () => {
        let linkResponse = await linkDevice("non-existent", "dummyKey");
        expect(linkResponse.status).toBe(404);
        expect(linkResponse.body.message).toBe(DEVICE_NOT_FOUND);
    });

    test.concurrent("Link device invalid key", async () => {
        let linkResponse = await linkDevice("non-existent", "this is as invalid key");
        expect(linkResponse.status).toBe(400);
        expect(linkResponse.body.message).toBe(INVALID_API_KEY);
    });

    test.concurrent("Unlink device non-existent device", async () => {
        let unlinkResponse = await unlinkDevice("non-existent", "dummyKey");
        expect(unlinkResponse.status).toBe(404);
        expect(unlinkResponse.body.message).toBe(DEVICE_NOT_FOUND);
    });

    test.concurrent("Unlink device invalid key", async () => {
        let unlinkResponse = await unlinkDevice("non-existent", "this is as invalid key");
        expect(unlinkResponse.status).toBe(400);
        expect(unlinkResponse.body.message).toBe(INVALID_API_KEY);
    });
});