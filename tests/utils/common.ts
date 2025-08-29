export const MIDDLEWARE_URL = requiredEnv('MIDDLEWARE_URL');
export const PROSA_URL = requiredEnv('PROSA_URL');
export const BOOK_DIR = 'books/';

export const INVALID_TOKEN = 'Invalid token';
export const UNAUTHENTICATED = 'No authentication was provided.';
export const DEVICE_NOT_LINKED = 'Device is recognized, but is unauthenticated.';

export function randomString(length: number) {
  let result = '';
  const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  const charactersLength = characters.length;
  for (let i = 0; i < length; i++) {
    result += characters.charAt(Math.floor(Math.random() * charactersLength));
  }
  return result;
}

export function wait(seconds: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, seconds * 1000));
}

function requiredEnv(name: string): string {
  const value = process.env[name];
  if (!value) {
    console.error(`Missing required environment variable: ${name}`);
    process.exit(1);
  }
  return value;
}
