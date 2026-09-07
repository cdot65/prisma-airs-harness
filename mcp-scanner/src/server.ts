import { timingSafeEqual } from 'node:crypto';
import express from 'express';
import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { StreamableHTTPServerTransport } from '@modelcontextprotocol/sdk/server/streamableHttp.js';
import { z } from 'zod';
import { ListResourcesRequestSchema, ListResourceTemplatesRequestSchema } from '@modelcontextprotocol/sdk/types.js';

export interface ScanInput {
  prompt?: string;
  response?: string;
  appName: string;
  signal: AbortSignal;
}

export interface ScanVerdict {
  action: string;
  scan_id: string;
  category?: string;
  error?: boolean;
  timeout?: boolean;
}

export function createApp(options: {
  gatewayKey: string;
  profile: string;
  scan: (input: ScanInput) => Promise<ScanVerdict>;
}) {
  if (options.gatewayKey.length < 32 || !options.profile.trim()) {
    throw new Error('A private gateway key and fixed scanner profile are required');
  }
  const expected = Buffer.from(options.gatewayKey);
  const app = express();
  app.disable('x-powered-by');
  app.get('/health', (_req, res) => res.json({ status: 'healthy' }));
  app.use('/terminal-scanner/mcp', (req, res, next) => {
    const supplied = req.header('x-airs-terminal-mcp-key');
    const received = Buffer.from(supplied ?? '');
    if (received.length !== expected.length || !timingSafeEqual(received, expected)) {
      res.status(401).json({ error: 'Unauthorized' });
      return;
    }
    next();
  });
  app.use(express.json({ limit: '1mb' }));
  app.all('/terminal-scanner/mcp', async (req, res) => {
    // Each HTTP request owns its SDK instance. No in-memory MCP sessions or
    // cross-replica session restoration are required by this fixed tool catalog.
    const server = new McpServer({ name: 'airs-harness-scanner', version: '0.1.0-alpha.8' });
    // Tool discovery and resource discovery are separate protocol operations.
    // This scanner has no resources; advertise an honest empty inventory.
    server.server.registerCapabilities({ resources: {} });
    server.server.setRequestHandler(ListResourcesRequestSchema, async () => ({ resources: [] }));
    server.server.setRequestHandler(ListResourceTemplatesRequestSchema, async () => ({ resourceTemplates: [] }));
    server.registerTool('pan_inline_scan', {
      title: 'Scan content with Prisma AIRS',
      description: 'Scan prompt or response text using the configured Prisma AIRS security profile. Returns the actual action and scan ID. Credentials remain on the remote server.',
      annotations: { readOnlyHint: true, destructiveHint: false, idempotentHint: true, openWorldHint: true },
      inputSchema: {
        scan_request: z.object({
          prompt: z.string().min(1).max(200_000).optional(),
          response: z.string().min(1).max(200_000).optional(),
          profile: z.literal(options.profile).optional(),
          app_name: z.string().min(1).max(100).optional(),
        }).strict().refine(value => Boolean(value.prompt || value.response), 'Provide prompt or response text'),
      },
    }, async ({ scan_request }, extra) => {
      try {
        const result = await options.scan({
          prompt: scan_request.prompt,
          response: scan_request.response,
          appName: scan_request.app_name ?? 'prisma-airs-harness',
          signal: extra.signal,
        });
        if (result.error || result.timeout || !result.scan_id || !['allow', 'block'].includes(result.action)) {
          throw new Error('Scanner did not return a complete verdict');
        }
        const results = {
          action: result.action,
          scan_id: result.scan_id,
          profile_name: options.profile,
          category: result.category,
        };
        return {
          content: [{ type: 'text' as const, text: JSON.stringify({ results }) }],
          structuredContent: { results },
        };
      } catch {
        // Never echo SDK errors: upstream errors can contain request details.
        return { isError: true, content: [{ type: 'text' as const, text: 'Prisma AIRS scan unavailable or incomplete; no allow verdict was produced.' }] };
      }
    });
    const transport = new StreamableHTTPServerTransport({
      sessionIdGenerator: undefined,
      enableJsonResponse: true,
    });
    res.on('close', () => { void transport.close(); void server.close(); });
    try {
      await server.connect(transport);
      await transport.handleRequest(req, res, req.body);
    } catch {
      if (!res.headersSent) res.status(500).json({ error: 'MCP request failed' });
    }
  });
  app.use((_error: unknown, _req: express.Request, res: express.Response, _next: express.NextFunction) => {
    if (!res.headersSent) res.status(400).json({ error: 'Invalid or oversized request' });
  });
  return app;
}
