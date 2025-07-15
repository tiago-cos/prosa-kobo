import request from "supertest";
import { randomString, SERVER_URL } from "./common";
import { createHash } from "crypto";

export const DEVICE_NOT_FOUND = "The requested device does not exist or is not accessible.";
export const DEVICE_ALREADY_LINKED = "This device is already linked.";
export const INVALID_API_KEY = "The provided api key is invalid.";
export const MISSING_API_KEY = "The api key must be provided.";

async function generateDeviceAuthRequest(device_id: string, user_key: string) {
    const request = {
        AffiliateName: "Kobo",
        AppVersion: "1.0.1",
        ClientKey: "client-key",
        DeviceId: device_id,
        PlatformId: "unimportant",
        SerialNumber: "30241001",
        UserKey: user_key,
    };

    return request;
}

function generateDeviceId(deviceId: string, userKey: string): string {
    const hash = createHash('sha256')
        .update(deviceId + userKey)
        .digest();
    return hash.toString('base64');
}

async function generateDeviceAuthRefreshRequest(refresh_token: string) {
    const request = {
        AppVersion: "1.0.1",
        ClientKey: "client-key",
        PlatformId: "unimportant",
        RefreshToken: refresh_token,
    };

    return request;
}

export async function getUnlinkedDevices() {
    let req = request(SERVER_URL).get("/devices/unlinked");
    return req.send();
}

export async function getLinkedDevices(api_key?: string) {
    let req = request(SERVER_URL).get("/devices/linked");

    if (api_key) req = req.query({ api_key: api_key });

    return req.send();
}

export async function linkDevice(device_id: string, api_key: string) {
    let req = request(SERVER_URL).post("/devices/link");

    return req.send({device_id: device_id, api_key: api_key});
}

export async function unlinkDevice(device_id: string, api_key: string) {
    let req = request(SERVER_URL).post("/devices/unlink");

    return req.send({device_id: device_id, api_key: api_key});
}


export async function authDevice(deviceId?: string, userKey?: string) {
    let req = request(SERVER_URL).post("/v1/auth/device");

    if (!deviceId)
        deviceId = randomString(16);
    if (!userKey)
        userKey = randomString(16);

    const response = await req.send(await generateDeviceAuthRequest(deviceId, userKey));

    deviceId = generateDeviceId(deviceId, userKey);

    return {response, deviceId, userKey};
}

export async function authRefreshDevice(refreshToken: string) {
    let req = request(SERVER_URL).post("/v1/auth/refresh");
    return req.send(await generateDeviceAuthRefreshRequest(refreshToken));
}