import { init, Scanner, Content } from '@cdot65/prisma-airs-sdk';
import { createApp } from './server.js';

const apiKey = process.env.AIRS_SCANNER_API_KEY;
const gatewayKey = process.env.AIRS_MCP_GATEWAY_KEY;
const profile = process.env.AIRS_SCANNER_PROFILE;
if (!apiKey || !gatewayKey || !profile) throw new Error('Missing required server configuration');
init({ apiKey, apiEndpoint: 'https://service.api.aisecurity.paloaltonetworks.com', numRetries: 0 });
const scanner = new Scanner();
const app = createApp({
  gatewayKey,
  profile,
  scan: input => scanner.syncScan(
    { profile_name: profile },
    new Content({ prompt: input.prompt, response: input.response }),
    {
      numRetries: 0,
      timeoutMs: 10_000,
      signal: input.signal,
      metadata: { app_name: input.appName, app_user: 'workspace-api-key' },
    },
  ),
});
const server = app.listen(8080, '0.0.0.0', () => console.log('AIRS Terminal scanner listening on port 8080'));
server.requestTimeout = 15_000;
server.headersTimeout = 10_000;
process.on('SIGTERM', () => {
  server.close(() => process.exit(0));
  setTimeout(() => process.exit(1), 15_000).unref();
});
