import { hash, isExcluded } from './config.mjs';
import { followUp } from './responses.mjs';

/** Describe source links and whether their exact targets are in this response. */
export function resolvedLinks(snapshot, sourceIds, includedIds = []) {
  const sources = new Set(sourceIds), included = new Set(includedIds);
  return snapshot.references.filter(edge => edge.source_section_ids.some(id => sources.has(id))).map(edge => {
    const present = edge.target_section_ids.filter(id => included.has(id));
    const missing = edge.target_section_ids.length - present.length;
    const status = edge.reason ? (edge.reason.startsWith('missing-') ? 'missing' : 'unavailable')
      : missing === 0 ? 'included' : present.length ? 'partial' : 'omitted';
    return { reference_id: edge.id, href: edge.href, source_section_id: edge.source_section_ids.find(id => sources.has(id)),
      source_path: edge.source_path, source_line: edge.source_line, target_path: edge.target_path,
      target_anchor: edge.target_anchor, status, included_section_ids: present,
      target_section_count: edge.target_section_ids.length, omitted_section_count: missing, reason: edge.reason,
      follow_up: edge.reason || !missing ? null : followUp(snapshot, 'fetch_document', {
        document: edge.target_path, ...(edge.target_anchor ? { anchor: edge.target_anchor } : {})
      }) };
  });
}

/** Add bounded reference metadata; link-only retrieval can page the remainder. */
export function attachResolvedLinks(result, snapshot, sourceIds, includedIds, budget) {
  const links = resolvedLinks(snapshot, sourceIds, includedIds);
  Object.assign(result, { resolved_links: [], resolved_links_total: links.length,
    resolved_links_complete: true, links_next: null });
  for (const link of links) {
    result.resolved_links.push(link);
    if (budget.fits()) continue;
    result.resolved_links.pop();
    result.resolved_links_complete = false;
    // A continuation can displace another mapping; point at the first omitted one.
    for (;;) {
      const pending = links[result.resolved_links.length];
      const ownLinks = resolvedLinks(snapshot, [pending.source_section_id]);
      result.links_next = followUp(snapshot, 'related_context', { section_id: pending.source_section_id,
        mode: 'links', offset: ownLinks.findIndex(item => item.reference_id === pending.reference_id) });
      if (budget.fits() || !result.resolved_links.length) break;
      result.resolved_links.pop();
    }
    break;
  }
}

// These are content dependencies, not arbitrary hyperlinks to other documents.
export function referenceGraph(documents, skipped = [], config) {
  const byPath = new Map(documents.map(doc => [doc.path, doc]));
  const omitted = new Map(skipped.map(item => [item.path, item.reason]));
  const edges = [];
  for (const doc of documents) {
    for (const [ordinal, link] of doc.links.entries()) {
      if (!link.target?.startsWith('references/')) continue;
      const linkedFrom = doc.sections.filter(section => link.line >= section.line_start && link.line <= section.line_end);
      const sourceIds = new Set(linkedFrom.map(section => section.logical_section_id));
      const source = doc.sections.filter(section => sourceIds.has(section.logical_section_id));
      const target = byPath.get(link.target);
      const targets = target?.sections.filter(section => !link.anchor || section.anchors.includes(link.anchor)) || [];
      const excluded = isExcluded(config, link.target);
      const reason = !target ? (omitted.get(link.target) || (excluded ? 'excluded-by-config' : 'missing-document'))
        : !targets.length ? 'missing-anchor' : null;
      edges.push({ id: `ref-${hash(`${doc.path}:${link.line}:${ordinal}:${link.href}`).slice(0, 24)}`,
        href: link.href, source_line: link.line,
        source_path: doc.path, source_section_ids: source.map(section => section.id),
        target_path: link.target, target_anchor: link.anchor || '',
        target_section_ids: targets.map(section => section.id), reason });
    }
  }
  return edges;
}

export function expandReferenceIds(snapshot, seed, authority, maxDepth) {
  const byId = new Map(snapshot.sections.map(section => [section.id, section]));
  const edges = snapshot.references;
  const outgoing = new Map(), incoming = new Map();
  for (const edge of edges) {
    for (const id of edge.source_section_ids) {
      if (!outgoing.has(id)) outgoing.set(id, []);
      outgoing.get(id).push(edge);
    }
    for (const id of edge.target_section_ids) {
      if (!incoming.has(id)) incoming.set(id, []);
      incoming.get(id).push(edge);
    }
  }
  const owners = new Set(seed.authorities.filter(item => item.authority === authority).map(item => item.owner));
  const allowed = section => section.authorities.some(item => item.authority === authority && owners.has(item.owner));
  const included = new Map();
  const omissions = new Map();
  const queue = [{ id: seed.id, depth: 0, direction: 'out' }];
  // A hit inside a reference also brings back its actual owning section. Do not
  // follow inbound links from every downstream reference into unrelated owners.
  if (seed.path.startsWith('references/')) queue.push({ id: seed.id, depth: 0, direction: 'in' });
  const seen = new Set();
  for (let cursor = 0; cursor < queue.length; cursor++) {
    const current = queue[cursor];
    const key = `${current.direction}:${current.id}`;
    if (seen.has(key)) continue;
    seen.add(key);
    const section = byId.get(current.id);
    // Include all chunks of a matched heading so a chunk split cannot hide links.
    const siblings = snapshot.sections.filter(item => item.logical_section_id === section.logical_section_id && allowed(item));
    for (const sibling of siblings) {
      if (sibling.id !== seed.id && !included.has(sibling.id)) included.set(sibling.id, { id: sibling.id, via: current.id, relation: 'same-section' });
    }
    for (const edge of (current.direction === 'out' ? outgoing : incoming).get(current.id) || []) {
      if (edge.reason) {
        omissions.set(edge.id, { reference_id: edge.id, target_path: edge.target_path, target_anchor: edge.target_anchor, reason: edge.reason });
        continue;
      }
      const targets = current.direction === 'out' ? edge.target_section_ids : edge.source_section_ids;
      for (const id of targets) {
        const next = byId.get(id);
        if (!allowed(next) || id === seed.id) continue;
        if (current.depth >= maxDepth) {
          if (!included.has(id)) omissions.set(id, { section_id: id, reference_id: edge.id, reason: 'depth-limit' });
          continue;
        }
        omissions.delete(id);
        if (!included.has(id)) included.set(id, { id, via: current.id, reference_id: edge.id,
          relation: current.direction === 'out' ? 'reference' : 'owner' });
        queue.push({ id, depth: current.depth + 1, direction: 'out' });
        if (current.direction === 'in' && next.path.startsWith('references/')) queue.push({ id, depth: current.depth + 1, direction: 'in' });
      }
    }
  }
  return { included: [...included.values()], omissions: [...omissions.values()] };
}
