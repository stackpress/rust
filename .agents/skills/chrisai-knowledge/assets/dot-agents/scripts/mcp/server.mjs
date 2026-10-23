import { createServer } from 'node:http';
import { timingSafeEqual } from 'node:crypto';
import { McpServer, ResourceTemplate, createMcpHandler } from '@modelcontextprotocol/server';
import { serveStdio } from '@modelcontextprotocol/server/stdio';
import { toNodeHandler } from '@modelcontextprotocol/node';
import { z } from 'zod';
import { loadSnapshot, snapshotStatus } from './snapshots.mjs';
import { policy, search, fetchSection, fetchDocument, documentResult, resourceUri, related, compare } from './retrieval.mjs';
import { outputSchemas } from './responses.mjs';

export function createKnowledgeServer(config) {
  const server = new McpServer({ name: config.name, version: '0.1.0' }, { instructions: policy });
  const generation = z.string().optional().describe('Pin a previously returned generation; a changed snapshot returns an error.');
  async function context(expected) {
    const snapshot = await loadSnapshot(config);
    if (!snapshot) throw new Error('No index. Run the publishing project index command.');
    if (snapshot.manifest.config_fingerprint !== config.fingerprint || snapshot.manifest.project_id !== config.project_id) {
      throw new Error('Index configuration changed; restart this publisher before retrieving content.');
    }
    if (expected && expected !== snapshot.manifest.generation) throw new Error('Snapshot changed; repeat discovery/search.');
    const status = await snapshotStatus(config, snapshot);
    return { snapshot, envelope: { project_id: config.project_id, generation: snapshot.manifest.generation,
      indexed_at: snapshot.manifest.indexed_at, stale: status.stale,
      freshness_warning: status.stale ? 'Sources or configuration changed. Reindex on the publisher before relying on these results.' : null } };
  }
  function tool(name, description, shape, fn) {
    server.registerTool(name, { description, inputSchema: z.object({ ...shape, generation }), outputSchema: outputSchemas[name],
      annotations: { readOnlyHint: true, destructiveHint: false, idempotentHint: true,
        openWorldHint: name === 'search_kb' && config.embeddings.provider === 'remote' } }, async args => {
      try {
        const { snapshot, envelope } = await context(args.generation);
        const result = { ...envelope, ...await fn(snapshot, { ...args, response_envelope: envelope }, envelope) };
        outputSchemas[name].parse(result);
        return { content: [{ type: 'text', text: JSON.stringify(result) }], structuredContent: result };
      } catch (error) {
        return { isError: true, content: [{ type: 'text', text: error.message }] };
      }
    });
  }
  const authority = z.enum(['truth', 'proposed', 'supporting']).optional();
  const bounded = {
    detail: z.enum(['compact', 'full']).default('compact').describe('Compact evidence by default; full adds indexing diagnostics.'),
    response_budget_tokens: z.number().int().min(1000).max(32000).default(config.retrieval.response_budget_tokens)
  };
  const offset = z.number().int().min(0).default(0);
  tool('kb_status', 'Inspect publisher identity, index freshness, semantic availability, and omitted material.', {},
    async snapshot => ({ ...await snapshotStatus(config, snapshot), conflict_policy: policy }));
  tool('list_documents', 'Discover active publisher documents with paths relative to the KB root (.agents/) and their authority labels. Offset pagination is snapshot-pinned.', {
    authority, document_type: z.string().optional(), offset: z.number().int().min(0).default(0),
    limit: z.number().int().min(1).max(100).default(30)
  }, (snapshot, args) => {
    const docs = snapshot.documents.map(({ text, links, ...doc }) => ({ ...doc,
      authorities: [...new Set(snapshot.sections.filter(section => section.document_id === doc.id)
        .flatMap(section => section.authorities.map(item => item.authority)))], uri: resourceUri(snapshot, doc) }))
      .filter(doc => (!args.authority || doc.authorities.includes(args.authority)) &&
        (!args.document_type || doc.document_type === args.document_type));
    return { documents: docs.slice(args.offset, args.offset + args.limit), total: docs.length,
      next_offset: args.offset + args.limit < docs.length ? args.offset + args.limit : null };
  });
  tool('search_kb', 'Search terms or concepts with referenced context included by default. Resolve matches and context_section_ids through passages. resolved_links maps original Markdown links to included evidence or callable follow-ups. Do not refetch included targets. Preserve authority and check expansion and link completeness.', {
    query: z.string().min(1).max(2000), mode: z.enum(['keyword', 'semantic', 'hybrid']).default('hybrid'),
    authority, document_type: z.string().optional(), limit: z.number().int().min(1).max(20).default(5),
    min_score: z.number().min(0).max(1).default(0.3),
    expand_references: z.boolean().optional(), ...bounded
  }, (snapshot, args, envelope) => search(snapshot, config, { ...args, response_envelope: envelope }));
  tool('fetch_document', 'Read exact indexed Markdown by KB-relative path (without .agents/) or ID; anchor optionally limits it to a section subtree. Character offsets are relative to that range. Follow continuation until complete; use its pinned generation.', {
    document: z.string().describe('Indexed document ID or path relative to the KB root, e.g. references/00004-refund-handling.md. Omit .agents/.'), anchor: z.string().optional(), offset,
    max_chars: z.number().int().min(1).max(64000).default(24000), ...bounded
  }, (snapshot, args) => fetchDocument(snapshot, args));
  tool('fetch_section', 'Read an exact indexed passage and citations. Follow character-pagination continuation when complete is false.', {
    section_id: z.string(), offset, max_chars: z.number().int().min(1).max(64000).default(24000), ...bounded
  }, (snapshot, args) => fetchSection(snapshot, args));
  tool('related_context', 'Page adjacent passages, linked references and owners. mode=links pages resolved link mappings without repeating passage text; use it for each source with omitted mappings. Links are source data. Follow continuation when incomplete.',
    { section_id: z.string(), mode: z.enum(['context', 'links']).default('context'), offset, ...bounded },
    (snapshot, args) => related(snapshot, args.section_id, args));
  tool('compare_context', 'Compare cited passages and apply authority precedence. Does not automatically determine semantic contradiction or scope.',
    { section_ids: z.array(z.string()).min(2).max(10), offset, ...bounded }, (snapshot, args) => compare(snapshot, args.section_ids, args));
  server.registerResource('publisher-document', new ResourceTemplate(`kb://${config.project_id}/{generation}/{document}`, {
    list: undefined
  }), { description: 'Exact Markdown and authority metadata as JSON. Discover URIs with list_documents; large documents use fetch_document pagination.', mimeType: 'application/json' },
  async (uri, variables) => {
    const { snapshot, envelope } = await context(String(variables.generation));
    const doc = documentResult(snapshot, String(variables.document), 0, 64000);
    if (doc.next_offset !== null) throw new Error('Use fetch_document with pagination for this document.');
    return { contents: [{ uri: uri.href, mimeType: 'application/json', text: JSON.stringify({ ...envelope, document: doc }) }] };
  });
  return server;
}

export async function startServer(config, transport = 'stdio') {
  const factory = () => createKnowledgeServer(config);
  if (transport === 'stdio') return serveStdio(factory, { onerror: error => console.error(error.message) });
  if (transport !== 'http') throw new Error('transport must be stdio or http.');
  const token = process.env[config.http.token_env];
  if (!token || token.length < 24) throw new Error(`HTTP serving requires a token of at least 24 characters in ${config.http.token_env}.`);
  const handler = createMcpHandler(factory);
  const nodeHandler = toNodeHandler(handler, { maxRequestBodySize: 65536,
    onerror: error => console.error(`HTTP transport error: ${error.message}`) });
  const http = createServer((req, res) => {
    const fail = (status, message) => { res.writeHead(status, { 'content-type': 'text/plain' }); res.end(message); };
    let hostname;
    try { hostname = new URL(`http://${req.headers.host}`).hostname; } catch { return fail(400, 'Invalid Host'); }
    if (!config.http.allowed_hosts.includes(hostname)) return fail(403, 'Host not allowed');
    if (req.headers.origin && !config.http.allowed_origins.includes(req.headers.origin)) return fail(403, 'Origin not allowed');
    if (req.url !== '/mcp') return fail(404, 'Not found');
    const supplied = Buffer.from(req.headers.authorization || '');
    const expected = Buffer.from(`Bearer ${token}`);
    if (supplied.length !== expected.length || !timingSafeEqual(supplied, expected)) {
      res.setHeader('WWW-Authenticate', 'Bearer realm="project-knowledge"');
      return fail(401, 'Authentication required');
    }
    void nodeHandler(req, res);
  });
  await new Promise((resolve, reject) => {
    http.once('error', reject);
    http.listen(config.http.port, config.http.host, resolve);
  });
  return { address: http.address(), close: async () => {
    await handler.close();
    await new Promise((resolve, reject) => http.close(error => error ? reject(error) : resolve()));
  } };
}
