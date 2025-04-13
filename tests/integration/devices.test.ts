import { INVALID_TOKEN } from "../utils/common";
import { authDevice, authRefreshDevice } from "../utils/devices";

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