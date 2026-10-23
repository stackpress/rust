import { mkdir, open, readFile, rename, rm, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { randomUUID } from 'node:crypto';
import { hostname } from 'node:os';
import { setTimeout as delay } from 'node:timers/promises';
import { collectCorpus } from './corpus.mjs';
import { embed, embeddingTokenizer, validateVectors } from './embeddings.mjs';
import { embeddingPrefix, splitText } from './tokens.mjs';
import { referenceGraph } from './references.mjs';
import { hash, SCHEMA } from './config.mjs';

const jsonl = records => records.map(record => JSON.stringify(record)).join('\n') + (records.length ? '\n' : '');
const parseJsonl = text => text.split('\n').filter(Boolean).map(line => JSON.parse(line));

export async function loadSnapshot(config) {
  let pointer;
  try { pointer = JSON.parse(await readFile(path.join(config.dataDir, 'current.json'), 'utf8')); }
  catch (error) { if (error.code === 'ENOENT') return null; throw error; }
  if (!/^[a-f0-9-]{36}$/.test(pointer.generation)) throw new Error('Invalid snapshot pointer.');
  return readGeneration(path.join(config.dataDir, 'generations', pointer.generation));
}

async function readGeneration(dir) {
  const manifest = JSON.parse(await readFile(path.join(dir, 'manifest.json'), 'utf8'));
  if (manifest.schema_version !== SCHEMA) {
    const error = new Error('Unsupported index schema; rebuild the index.');
    if ([1, 2, 3].includes(manifest.schema_version)) error.code = 'OLD_INDEX_VERSION';
    throw error;
  }
  const data = {};
  for (const name of ['documents', 'sections', 'vectors', 'references']) {
    const bytes = await readFile(path.join(dir, `${name}.jsonl`));
    if (hash(bytes) !== manifest.files[`${name}.jsonl`]) throw new Error(`Corrupt ${name} index; rebuild the index.`);
    data[name] = parseJsonl(bytes.toString('utf8'));
  }
  if (new Set(data.documents.map(doc => doc.id)).size !== data.documents.length ||
      new Set(data.sections.map(section => section.id)).size !== data.sections.length) throw new Error('Duplicate index IDs.');
  for (const doc of data.documents) {
    const sections = data.sections.filter(section => section.document_id === doc.id);
    if (hash(doc.text) !== doc.source_hash || sections.map(section => section.text).join('') !== doc.text) {
      throw new Error(`Index failed lossless reconstruction: ${doc.path}`);
    }
  }
  if (data.vectors.length) {
    if (data.sections.some(section => section.embedding_hash !== hash(section.embedding_context + section.text))) {
      throw new Error('Embedding context does not match its cache key; rebuild the index.');
    }
    validateVectors(data.vectors.map(row => row.vector), data.sections.length, manifest.embeddings.dimensions);
    if (data.vectors.some((row, index) => row.id !== data.sections[index].id)) throw new Error('Vector IDs do not match sections.');
  } else if (manifest.embeddings.available && data.sections.length) throw new Error('Missing indexed vectors.');
  const sectionIds = new Set(data.sections.map(section => section.id));
  if (data.references.some(edge => [...edge.source_section_ids, ...edge.target_section_ids].some(id => !sectionIds.has(id)))) {
    throw new Error('Corrupt reference graph; section is missing.');
  }
  return { manifest, ...data };
}

export async function buildIndex(config, { force = false, embedder = embed } = {}) {
  await mkdir(config.dataDir, { recursive: true });
  const lock = path.join(config.dataDir, 'index.lock');
  let handle;
  for (let attempt = 0; attempt < 120; attempt++) {
    try { handle = await open(lock, 'wx'); break; }
    catch (error) {
      if (error.code !== 'EEXIST') throw error;
      if (attempt === 119) throw new Error('Index is locked. If its recorded process has stopped, remove data/index.lock and retry.');
      await delay(250);
    }
  }
  await handle.writeFile(JSON.stringify({ pid: process.pid, host: hostname(), started_at: new Date().toISOString() }));
  const generation = randomUUID();
  const staging = path.join(config.dataDir, `staging-${generation}`);
  const pointerTemp = path.join(config.dataDir, `current-${generation}.tmp`);
  try {
    const corpus = await collectCorpus(config);
    let previous;
    try { previous = await loadSnapshot(config); }
    catch (error) { if (!force && error.code !== 'OLD_INDEX_VERSION') throw error; }
    if (!force && previous?.manifest.config_fingerprint === config.fingerprint &&
        previous.manifest.source_fingerprint === corpus.source_fingerprint) return previous;
    let sections = corpus.documents.flatMap(doc => doc.sections);
    let vectors = [];
    const embeddings = { available: false, provider: config.embeddings.provider,
      model: config.embeddings.model, revision: config.embeddings.revision,
      fingerprint: config.embeddingFingerprint, dimensions: 0, reason: 'not configured' };
    if (config.embeddings.provider !== 'none' && sections.length) {
      try {
        const tokenizer = await embeddingTokenizer(config);
        const rebuilt = corpus.documents.map(doc => {
          const split = doc.sections.flatMap(section => {
            const prefix = embeddingPrefix(doc, section, tokenizer);
            return splitText(section.text, text => tokenizer.count(prefix + text), tokenizer.limit).map(part => {
            const start = section.start + part.start;
            const lineStart = doc.text.slice(0, start).split('\n').length;
            return { ...section, id: `sec-${hash(`${section.logical_section_id}:${start}`).slice(0, 24)}`,
              start, end: section.start + part.end, text: part.text, content_hash: hash(part.text),
              line_start: lineStart, line_end: lineStart + (part.text.match(/\n/g)?.length || 0) - (part.text.endsWith('\n') ? 1 : 0),
              embedding_context: prefix, embedding_hash: hash(prefix + part.text), embedding_tokens: part.tokens };
            });
          });
          return { ...doc, sections: split };
        });
        corpus.documents = rebuilt;
        sections = rebuilt.flatMap(doc => doc.sections);
        embeddings.tokenizer = tokenizer.name;
        embeddings.max_input_tokens = tokenizer.limit;
        const reusable = new Map();
        if (previous?.manifest.embeddings.fingerprint === config.embeddingFingerprint) {
          const oldSections = new Map(previous.sections.map(section => [section.id, section]));
          for (const row of previous.vectors) reusable.set(oldSections.get(row.id)?.embedding_hash, row.vector);
        }
        const missing = sections.filter(section => !reusable.has(section.embedding_hash));
        for (let i = 0; i < missing.length; i += config.embeddings.batch_size) {
          const batch = missing.slice(i, i + config.embeddings.batch_size);
          const result = validateVectors(await embedder(batch.map(section => section.embedding_context + section.text), config), batch.length);
          batch.forEach((section, index) => reusable.set(section.embedding_hash, result[index]));
        }
        vectors = sections.map(section => ({ id: section.id, vector: reusable.get(section.embedding_hash) }));
        validateVectors(vectors.map(row => row.vector), sections.length);
        Object.assign(embeddings, { available: true, dimensions: vectors[0].vector.length, reason: null });
      } catch (error) {
        if (config.embeddings.fallback === 'error') throw error;
        vectors = [];
        embeddings.reason = 'Embedding generation failed; run index --force after restoring the configured provider.';
        console.error(`Semantic indexing unavailable: ${error.message}`);
      }
    }
    if ((await collectCorpus(config)).source_fingerprint !== corpus.source_fingerprint) {
      throw new Error('Sources changed while indexing. Previous snapshot retained; retry.');
    }
    await mkdir(staging, { recursive: true });
    const documents = corpus.documents.map(({ sections, ...doc }) => doc);
    const references = referenceGraph(corpus.documents, corpus.skipped, config);
    const files = {};
    for (const [name, rows] of Object.entries({ documents, sections, vectors, references })) {
      const text = jsonl(rows);
      files[`${name}.jsonl`] = hash(text);
      await writeFile(path.join(staging, `${name}.jsonl`), text);
    }
    const manifest = { schema_version: SCHEMA, generation, project_id: config.project_id,
      indexed_at: new Date().toISOString(), source_fingerprint: corpus.source_fingerprint,
      config_fingerprint: config.fingerprint, embeddings, files,
      documents: documents.length, sections: sections.length,
      attachments: corpus.attachments, skipped: corpus.skipped };
    await writeFile(path.join(staging, 'manifest.json'), JSON.stringify(manifest, null, 2));
    const snapshot = await readGeneration(staging);
    const generations = path.join(config.dataDir, 'generations');
    await mkdir(generations, { recursive: true });
    await rename(staging, path.join(generations, generation));
    await writeFile(pointerTemp, JSON.stringify({ generation }));
    await rename(pointerTemp, path.join(config.dataDir, 'current.json'));
    return snapshot;
  } finally {
    await rm(staging, { recursive: true, force: true });
    await rm(pointerTemp, { force: true });
    await handle.close();
    await rm(lock, { force: true });
  }
}

export async function snapshotStatus(config, snapshot) {
  const source = await collectCorpus(config);
  return { project_id: config.project_id, generation: snapshot?.manifest.generation ?? null,
    indexed_at: snapshot?.manifest.indexed_at ?? null,
    stale: !snapshot || snapshot.manifest.config_fingerprint !== config.fingerprint ||
      snapshot.manifest.source_fingerprint !== source.source_fingerprint,
    documents: snapshot?.documents.length ?? 0, sections: snapshot?.sections.length ?? 0,
    embeddings: snapshot?.manifest.embeddings ?? { available: false },
    skipped: source.skipped, attachments: source.attachments };
}
