import path from 'node:path';
import { responseTokens } from './tokens.mjs';

let localPipeline;
let localKey;
let localTokenizer;
let tokenizerKey;

async function tokenizerFor(config) {
  const settings = config.embeddings;
  if (!/^[a-f0-9]{40}$/i.test(settings.revision)) throw new Error('Pin the local embedding model revision to a 40-character commit before indexing.');
  const { AutoTokenizer, env } = await import('@huggingface/transformers');
  const cache = path.join(config.runtimeDir, 'models');
  env.cacheDir = cache;
  env.allowRemoteModels = settings.allow_download;
  const key = `${cache}:${config.embeddingFingerprint}:${settings.allow_download}`;
  if (!localTokenizer || key !== tokenizerKey) {
    tokenizerKey = key;
    localTokenizer = AutoTokenizer.from_pretrained(settings.model, {
      revision: settings.revision, cache_dir: cache, local_files_only: !settings.allow_download
    }).catch(error => { localTokenizer = undefined; throw error; });
  }
  return localTokenizer;
}

export async function embeddingTokenizer(config) {
  const settings = config.embeddings;
  if (settings.provider === 'local') {
    const tokenizer = await tokenizerFor(config);
    return { name: `${settings.model}@${settings.revision}`, limit: Math.min(settings.max_input_tokens, tokenizer.model_max_length),
      count: text => tokenizer.encode(text, { add_special_tokens: true }).length };
  }
  const name = settings.remote_tokenizer || 'utf8_bytes';
  return { name, limit: settings.max_input_tokens,
    count: name === 'cl100k_base' ? responseTokens : text => Buffer.byteLength(text, 'utf8') };
}

export function validateVectors(vectors, count, dimensions) {
  if (!Array.isArray(vectors) || vectors.length !== count) throw new Error('Embedding count mismatch.');
  const size = dimensions ?? vectors[0]?.length;
  if (!size || vectors.some(vector => !Array.isArray(vector) || vector.length !== size ||
      vector.some(value => !Number.isFinite(value)) || !vector.some(value => value !== 0))) {
    throw new Error('Invalid embedding dimensions or non-finite/zero vector.');
  }
  return vectors;
}

export async function embed(texts, config) {
  if (!texts.length) return [];
  const settings = config.embeddings;
  if (settings.provider === 'none') throw new Error('Semantic search is not configured.');
  const tokenizer = await embeddingTokenizer(config);
  if (texts.some(text => tokenizer.count(text) > tokenizer.limit)) {
    throw new Error('Embedding input exceeds the configured token limit; shorten the query or rebuild token-sized chunks.');
  }
  if (settings.provider === 'local') {
    if (!/^[a-f0-9]{40}$/i.test(settings.revision)) {
      throw new Error('Pin the local embedding model revision to a 40-character commit before indexing.');
    }
    const { AutoModel, FeatureExtractionPipeline, env } = await import('@huggingface/transformers');
    env.cacheDir = path.join(config.runtimeDir, 'models');
    env.allowRemoteModels = settings.allow_download;
    const key = `${config.runtimeDir}:${config.embeddingFingerprint}:${settings.allow_download}`;
    if (!localPipeline || localKey !== key) {
      localKey = key;
      const options = { revision: settings.revision, dtype: 'q8', device: 'cpu',
        cache_dir: env.cacheDir, local_files_only: !settings.allow_download };
      // Explicit components avoid pipeline auto-discovery consulting an unpinned
      // remote file list when the operator requested a cached, pinned revision.
      localPipeline = Promise.all([
        tokenizerFor(config),
        AutoModel.from_pretrained(settings.model, options)
      ]).then(([tokenizer, model]) => new FeatureExtractionPipeline({
        task: 'feature-extraction', tokenizer, model
      })).catch(error => { localPipeline = undefined; throw error; });
    }
    const pipe = await localPipeline;
    const result = await pipe(texts, { pooling: 'mean', normalize: true });
    return validateVectors(result.tolist(), texts.length);
  }
  if (!settings.allow_remote) throw new Error('Remote embeddings require allow_remote=true.');
  const endpoint = new URL(settings.endpoint);
  if (endpoint.protocol !== 'https:' && !(endpoint.protocol === 'http:' &&
      ['localhost', '127.0.0.1', '[::1]'].includes(endpoint.hostname))) {
    throw new Error('Embedding endpoint must use HTTPS (HTTP is allowed on loopback only).');
  }
  const key = process.env[settings.api_key_env];
  if (!key) throw new Error(`Set the embedding credential environment variable ${settings.api_key_env}.`);
  const response = await fetch(endpoint, {
    method: 'POST', redirect: 'error', signal: AbortSignal.timeout(settings.timeout_ms),
    headers: { 'content-type': 'application/json', authorization: `Bearer ${key}` },
    body: JSON.stringify({ model: settings.model, input: texts, encoding_format: 'float' })
  });
  if (!response.ok) throw new Error(`Embedding provider returned HTTP ${response.status}.`);
  const result = await response.json();
  if (result.model && result.model !== settings.model) throw new Error('Embedding provider model does not match configuration.');
  if (!Array.isArray(result.data) || result.data.some((item, index, all) =>
    !Number.isInteger(item.index) || item.index < 0 || item.index >= texts.length ||
    all.findIndex(other => other.index === item.index) !== index)) throw new Error('Invalid embedding response indexes.');
  return validateVectors(result.data.sort((a, b) => a.index - b.index).map(item => item.embedding), texts.length);
}

export function cosine(a, b) {
  if (a.length !== b.length) throw new Error('Query/index embedding dimensions do not match.');
  let dot = 0, aa = 0, bb = 0;
  for (let i = 0; i < a.length; i++) { dot += a[i] * b[i]; aa += a[i] ** 2; bb += b[i] ** 2; }
  return dot / (Math.sqrt(aa * bb) || 1);
}
