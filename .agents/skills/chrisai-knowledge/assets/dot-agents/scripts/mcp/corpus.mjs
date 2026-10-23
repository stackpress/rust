import { lstat, readFile, readdir, realpath } from 'node:fs/promises';
import path from 'node:path';
import MarkdownIt from 'markdown-it';
import { parse as parseYaml } from 'yaml';
import { hash, inside, isExcluded, posix } from './config.mjs';

const markdown = new MarkdownIt({ html: false });
const inactive = new Set(['archived', 'retired', 'superseded', 'rejected', 'deleted']);
const excludedDirs = new Set(['archives', 'archive', 'retired', 'trash', 'node_modules', '.git']);
const slug = text => text.toLowerCase().replace(/[^\p{L}\p{N}\s_-]/gu, '').trim().replace(/\s+/g, '-');
const linesOf = text => text.match(/[^\n]*\n|[^\n]+$/g) || [];

function kind(file) {
  const category = file.split('/')[0];
  return ({ context: 'context', specs: 'specification', references: 'reference',
    resources: 'evidence', workflows: 'workflow' })[category] || 'workspace-document';
}

function parseDocument(file, text, config) {
  const lines = linesOf(text);
  let meta = {};
  let parseText = text;
  if (/^---\r?\n/.test(text)) {
    const end = lines.findIndex((line, i) => i > 0 && /^(---|\.\.\.)\s*$/.test(line));
    if (end < 0) throw new Error(`Unclosed frontmatter: ${file}`);
    meta = parseYaml(lines.slice(1, end).join('')) || {};
    if (typeof meta !== 'object' || Array.isArray(meta)) throw new Error(`Invalid frontmatter: ${file}`);
    parseText = lines.map((line, i) => i <= end ? '\n' : line).join('');
  }
  const tokens = markdown.parse(parseText, {});
  const prose = tokens.filter(token => token.type === 'inline').map(token => token.content).join('\n');
  const lifecycle = String(meta.status || meta.lifecycle ||
    prose.match(/^\s*(?:[-*]\s+)?(?:\*\*)?(?:Status|Lifecycle)(?:\*\*)?:\s*(?:\*\*)?([^\r\n*]+)/im)?.[1] || 'unspecified').trim().toLowerCase();
  const documentType = kind(file);
  const document = { id: `doc-${hash(file).slice(0, 24)}`, path: file,
    title: String(meta.title || file), document_type: documentType, lifecycle,
    source_hash: hash(text), text, links: [], sections: [],
    active: !inactive.has(lifecycle), tags: Array.isArray(meta.tags) ? meta.tags.map(String) : [] };
  const headings = [];
  const counts = new Map();
  const usedAnchors = new Set();
  const stack = [];
  for (let i = 0; i < tokens.length; i++) {
    const token = tokens[i];
    if (token.type === 'heading_open') {
      const title = tokens[i + 1].content;
      const base = slug(title) || 'section';
      let n = counts.get(base) || 0;
      while (usedAnchors.has(n ? `${base}-${n}` : base)) n++;
      counts.set(base, n + 1);
      const anchor = n ? `${base}-${n}` : base;
      usedAnchors.add(anchor);
      const level = Number(token.tag.slice(1));
      while (stack.length && stack.at(-1).level >= level) stack.pop();
      stack.push({ level, title, anchor });
      headings.push({ line: token.map[0], headings: stack.map(x => x.title),
        anchors: stack.map(x => x.anchor), anchor });
      if (document.title === file) document.title = title;
    }
    if (token.type === 'inline') {
      let sourceLine = (token.map?.[0] || 0) + 1;
      for (const child of token.children || []) {
        const href = child.type === 'link_open' ? child.attrGet('href')
          : child.type === 'image' ? child.attrGet('src') : null;
        if (href) document.links.push({ href, line: sourceLine });
        if (child.type === 'softbreak' || child.type === 'hardbreak') sourceLine++;
      }
    }
  }
  const offsets = [0];
  for (const line of lines) offsets.push(offsets.at(-1) + line.length);
  const boundaries = [{ line: 0, headings: [], anchors: [], anchor: '' }, ...headings]
    .filter((entry, index, all) => index === all.length - 1 || entry.line !== all[index + 1].line);
  for (let i = 0; i < boundaries.length; i++) {
    const entry = boundaries[i];
    let start = offsets[entry.line] || 0;
    const end = offsets[boundaries[i + 1]?.line ?? lines.length] ?? text.length;
    let part = 0;
    while (start < end) {
      let finish = Math.min(end, start + config.max_chunk_chars);
      if (finish < end) {
        const newline = text.lastIndexOf('\n', finish - 1);
        if (newline >= start) finish = newline + 1;
        // Do not split a surrogate pair in a long single line.
        if (/[\uD800-\uDBFF]/.test(text[finish - 1])) finish--;
      }
      const lineStart = text.slice(0, start).split('\n').length;
      const slice = text.slice(start, finish);
      const lineEnd = lineStart + (slice.match(/\n/g)?.length || 0) - (slice.endsWith('\n') ? 1 : 0);
      const searchable = tokens.some(token => ['paragraph_open', 'fence', 'code_block', 'table_open', 'html_block'].includes(token.type)
        && token.map && token.map[0] < lineEnd && token.map[1] >= lineStart);
      document.sections.push({ id: `sec-${hash(`${file}#${entry.anchor}:${part++}`).slice(0, 24)}`,
        logical_section_id: `heading-${hash(`${file}#${entry.anchor}`).slice(0, 24)}`,
        document_id: document.id, path: file, heading: entry.headings.join(' > '),
        anchor: entry.anchor, anchors: entry.anchors, start, end: finish,
        line_start: lineStart, line_end: lineEnd,
        text: slice, content_hash: hash(slice), searchable, authorities: [] });
      start = finish;
    }
  }
  return document;
}

export async function collectCorpus(config) {
  const agents = path.join(config.root, '.agents');
  if ((await lstat(agents)).isSymbolicLink()) throw new Error('.agents must not be a symlink.');
  const documents = [];
  const resources = new Map();
  const skipped = [];
  const blocked = file => file.startsWith('skills/') || file.startsWith('scripts/') || isExcluded(config, file);
  async function walk(dir) {
    for (const entry of (await readdir(dir, { withFileTypes: true })).sort((a, b) => a.name.localeCompare(b.name))) {
      const full = path.join(dir, entry.name);
      const file = posix(path.relative(agents, full));
      if (entry.isSymbolicLink()) { skipped.push({ path: file, reason: 'symlink' }); continue; }
      if (entry.name.startsWith('.') || excludedDirs.has(entry.name.toLowerCase()) || blocked(`${file}/`)) continue;
      if (entry.isDirectory()) { await walk(full); continue; }
      if (!entry.isFile()) continue;
      if (file.startsWith('resources/')) { resources.set(file, full); continue; }
      if (/\.md$/i.test(file) && !blocked(file)) documents.push(await readDocument(file, full));
    }
  }
  async function readDocument(file, full) {
    if (!inside(agents, await realpath(full))) throw new Error(`Source escaped .agents: ${file}`);
    if ((await lstat(full)).size > config.max_file_bytes) throw new Error(`Source exceeds max_file_bytes: ${file}`);
    const bytes = await readFile(full);
    const text = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(bytes);
    return parseDocument(file, text, config);
  }
  await walk(agents);
  const activeDocs = documents.filter(doc => {
    if (doc.active) return true;
    skipped.push({ path: doc.path, reason: `lifecycle:${doc.lifecycle}` });
    return false;
  });
  // Linked resource markdown participates as evidence; binary attachments are metadata only.
  const attachments = new Map();
  const seenResources = new Set();
  for (let i = 0; i < activeDocs.length; i++) {
    const doc = activeDocs[i];
    for (const link of doc.links) {
      if (/^[a-zA-Z][\w+.-]*:/.test(link.href) || link.href.startsWith('//')) continue;
      let target;
      try {
        const [filePart, anchor = ''] = link.href.split('#');
        const full = filePart ? path.resolve(agents, path.dirname(doc.path), decodeURIComponent(filePart))
          : path.join(agents, doc.path);
        if (!inside(agents, full)) continue;
        target = posix(path.relative(agents, full));
        link.target = target;
        link.anchor = decodeURIComponent(anchor);
      } catch { continue; }
      if (resources.has(target) && !seenResources.has(target) && !blocked(target)) {
        seenResources.add(target);
        if (/\.md$/i.test(target)) {
          const resource = await readDocument(target, resources.get(target));
          if (resource.active) activeDocs.push(resource);
          else skipped.push({ path: target, reason: `lifecycle:${resource.lifecycle}` });
        } else {
          const stats = await lstat(resources.get(target));
          attachments.set(target, { path: target, bytes: stats.size, indexed_text: false });
        }
      }
    }
  }
  activeDocs.sort((a, b) => a.path.localeCompare(b.path));
  const byPath = new Map(activeDocs.map(doc => [doc.path, doc]));
  // Only context is truth. Reference ownership propagates through references, never through evidence.
  const queue = [];
  for (const doc of activeDocs) {
    if (doc.document_type === 'reference') continue;
    const state = { authority: doc.document_type === 'context' ? 'truth'
      : ['evidence', 'workflow', 'workspace-document'].includes(doc.document_type) ? 'supporting' : 'proposed', owner: doc.path };
    for (const section of doc.sections) { section.authorities.push(state); queue.push({ doc, section, state }); }
  }
  while (queue.length) {
    const { doc, section, state } = queue.shift();
    for (const link of doc.links.filter(link => link.line >= section.line_start && link.line <= section.line_end)) {
      const target = byPath.get(link.target);
      if (target?.document_type !== 'reference') continue;
      for (const next of target.sections.filter(item => !link.anchor || item.anchors.includes(link.anchor))) {
        if (next.authorities.some(item => item.authority === state.authority && item.owner === state.owner)) continue;
        next.authorities.push(state);
        queue.push({ doc: target, section: next, state });
      }
    }
  }
  for (const doc of activeDocs) {
    for (const section of doc.sections) {
      if (!section.authorities.length) section.authorities.push({ authority: 'proposed', owner: null });
      section.authorities.sort((a, b) => `${a.authority}:${a.owner}`.localeCompare(`${b.authority}:${b.owner}`));
    }
  }
  return { documents: activeDocs, attachments: [...attachments.values()], skipped,
    source_fingerprint: hash(JSON.stringify(activeDocs.map(doc => [doc.path, doc.source_hash]).concat(
      [...attachments.values()].map(item => [item.path, item.bytes])))) };
}
