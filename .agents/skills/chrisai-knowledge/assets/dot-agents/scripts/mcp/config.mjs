import { readFile, realpath } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { z } from 'zod';

export const runtimeDir = path.dirname(fileURLToPath(import.meta.url));
export const SCHEMA = 4;
export const hash = value => createHash('sha256').update(value).digest('hex');
export const posix = value => value.split(path.sep).join('/');
export const inside = (root, file) => {
  const rel = path.relative(root, file);
  return rel === '' || (!rel.startsWith(`..${path.sep}`) && rel !== '..' && !path.isAbsolute(rel));
};

/** Match publisher exclusion settings against a path relative to the KB root. */
export function isExcluded(config, file) {
  const source = `.agents/${file}`;
  return config.exclude.some(prefix => source === prefix.replace(/\/$/, '') ||
    source.startsWith(`${prefix.replace(/\/$/, '')}/`));
}

const schema = z.object({
  schema_version: z.literal(1),
  name: z.string().regex(/^[a-z0-9][a-z0-9-]{0,63}$/),
  project_id: z.string().regex(/^[a-zA-Z0-9][a-zA-Z0-9._-]{0,127}$/),
  project_root: z.string().min(1),
  exclude: z.array(z.string()).default([]),
  max_file_bytes: z.number().int().min(1024).max(50 * 1024 * 1024).default(2097152),
  max_chunk_chars: z.number().int().min(200).max(8000).default(1600),
  retrieval: z.object({
    expand_references: z.boolean().default(true),
    response_budget_tokens: z.number().int().min(1000).max(32000).default(6000),
    max_reference_depth: z.number().int().min(1).max(32).default(8)
  }).strict().default({ expand_references: true, response_budget_tokens: 6000, max_reference_depth: 8 }),
  embeddings: z.object({
    provider: z.enum(['none', 'local', 'remote']).default('local'),
    model: z.string().min(1),
    revision: z.string().min(1),
    endpoint: z.string().default(''),
    api_key_env: z.string().default('CHRISAI_EMBEDDING_API_KEY'),
    allow_remote: z.boolean().default(false),
    allow_download: z.boolean().default(true),
    fallback: z.enum(['keyword', 'error']).default('keyword'),
    batch_size: z.number().int().min(1).max(128).default(16),
    max_input_tokens: z.number().int().min(32).max(8192).default(256),
    remote_tokenizer: z.enum(['utf8_bytes', 'cl100k_base']).default('utf8_bytes'),
    timeout_ms: z.number().int().min(100).max(300000).default(30000)
  }).strict(),
  http: z.object({
    host: z.string().default('127.0.0.1'),
    port: z.number().int().min(0).max(65535).default(3333),
    allowed_hosts: z.array(z.string()).min(1),
    allowed_origins: z.array(z.string()).default([]),
    token_env: z.string().default('CHRISAI_KB_TOKEN')
  }).strict()
}).strict();

export async function loadConfig(filename = path.join(runtimeDir, 'config.json')) {
  const configPath = path.resolve(filename);
  const config = schema.parse(JSON.parse(await readFile(configPath, 'utf8')));
  if (config.project_id === 'replace-with-a-stable-project-id') {
    throw new Error('Set a stable, unique project_id in config.json before indexing.');
  }
  for (const prefix of config.exclude) {
    if (!prefix.startsWith('.agents/') || prefix.includes('..') || prefix.includes('\\') || prefix.includes('*')) {
      throw new Error('exclude entries must be project-relative .agents/ path prefixes, without globs.');
    }
  }
  const root = await realpath(path.resolve(path.dirname(configPath), config.project_root));
  const embeddingFingerprint = hash(JSON.stringify({
    provider: config.embeddings.provider, model: config.embeddings.model,
    revision: config.embeddings.revision, endpoint: config.embeddings.endpoint,
    max_input_tokens: config.embeddings.max_input_tokens,
    remote_tokenizer: config.embeddings.remote_tokenizer,
    implementation: 'transformers-3.8.1/mean-normalized-q8/context-chunks-v3'
  }));
  return { ...config, root, configPath, runtimeDir, dataDir: path.join(runtimeDir, 'data'),
    fingerprint: hash(JSON.stringify({ config, index_schema: SCHEMA })), embeddingFingerprint };
}
