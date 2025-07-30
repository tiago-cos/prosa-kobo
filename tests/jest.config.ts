/** @type {import('ts-jest').JestConfigWithTsJest} */

require('dotenv').config({ path: 'config/.env.local' });
require('dotenv').config({ path: 'config/.env' });

module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  testMatch: ['**/*.test.ts'],
  globalSetup: './jest.setup.ts'
};
