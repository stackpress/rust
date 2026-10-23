import { z } from 'zod';
import { responseTokens } from './tokens.mjs';

/** Measure one complete response, including the server's freshness envelope. */
export function budgetResponse(result, options = {}) {
  const max = options.response_budget_tokens ?? 6000;
  result.response_schema_version = 3;
  result.token_budget = { encoding: 'cl100k_base', max, used: 0 };
  const size = () => responseTokens(JSON.stringify({ ...options.response_envelope, ...result }));
  return {
    fits: (ceiling = max) => size() + 32 <= ceiling,
    finish: () => {
      for (let i = 0; i < 8; i++) {
        const used = size();
        if (used === result.token_budget.used) break;
        result.token_budget.used = used;
      }
      if (size() > max) throw new Error('Response metadata exceeds the token budget; increase response_budget_tokens or use detail="compact".');
      return result;
    }
  };
}

/** Return a callable, snapshot-pinned continuation without implicit session state. */
export function followUp(snapshot, tool, args) {
  return { tool, arguments: { ...args, generation: snapshot.manifest.generation } };
}

/** Avoid splitting a Unicode surrogate pair at character pagination boundaries. */
export function safeEnd(text, end) {
  if (end > 0 && end < text.length && /[\uD800-\uDBFF]/.test(text[end - 1]) && /[\uDC00-\uDFFF]/.test(text[end])) return end - 1;
  return end;
}

const count = z.number().int().nonnegative();
const authority = z.enum(['truth', 'proposed', 'supporting']);
const action = z.object({ tool: z.string(), arguments: z.record(z.string(), z.unknown()) });
const citation = z.object({ uri: z.string(), path: z.string(), line_start: count, line_end: count,
  source_hash: z.string().optional() });
const ownership = z.object({ authority, owner: z.string().nullable() });
const passage = z.object({ id: z.string(), document_id: z.string(), path: z.string(), heading: z.string(),
  title: z.string(), anchor: z.string(), line_start: count, line_end: count, text: z.string(),
  authorities: z.array(ownership), document_type: z.string(), lifecycle: z.string(), citation }).passthrough();
const link = z.object({ reference_id: z.string(), href: z.string(), source_section_id: z.string(),
  source_path: z.string(), source_line: count, target_path: z.string(), target_anchor: z.string(),
  status: z.enum(['included', 'partial', 'omitted', 'missing', 'unavailable']),
  included_section_ids: z.array(z.string()), target_section_count: count, omitted_section_count: count,
  reason: z.string().nullable(), follow_up: action.nullable() });
const links = { resolved_links: z.array(link), resolved_links_total: count,
  resolved_links_complete: z.boolean(), links_next: action.nullable() };
const envelope = { project_id: z.string(), generation: z.string(), indexed_at: z.string(), stale: z.boolean(),
  freshness_warning: z.string().nullable() };
const bounded = { ...envelope, response_schema_version: z.literal(3),
  token_budget: z.object({ encoding: z.literal('cl100k_base'), max: count, used: count }) };
const page = { offset: count, next_offset: count.nullable(), total: count, complete: z.boolean(),
  continuation: action.nullable() };
const match = z.object({ section_id: z.string(), score: z.number(), exact_match: z.boolean(),
  context_section_ids: z.array(z.string()), context_complete: z.boolean().nullable() });
const group = z.object({ total: count, matches: z.array(match) }).optional();

// Typed output contracts describe both compact results and optional diagnostics.
export const outputSchemas = {
  search_kb: z.object({ ...bounded, ...links, query: z.string(), requested_mode: z.string(),
    effective_mode: z.string(), warning: z.string().nullable(), detail: z.enum(['compact', 'full']),
    groups: z.object({ truth: group, proposed: group, supporting: group }),
    passages: z.array(passage), expansion: z.object({ enabled: z.boolean(), complete: z.boolean().nullable(),
      omitted: z.array(z.object({ reason: z.string() }).passthrough()), omitted_count: count, omissions_truncated: z.boolean() }),
    conflict_policy: z.string(), conflict_detection: z.string(), no_match: z.boolean() }),
  fetch_section: z.object({ ...bounded, ...links, passage, offset: count, next_offset: count.nullable(),
    total_chars: count, complete: z.boolean(), continuation: action.nullable() }),
  fetch_document: z.object({ ...bounded, ...links, document: z.object({ id: z.string(), path: z.string(),
    title: z.string(), document_type: z.string(), lifecycle: z.string(), uri: z.string(), text: z.string(),
    offset: count, next_offset: count.nullable(), total_chars: count, range_start: count, range_end: count,
    sections: z.array(z.object({ id: z.string(), heading: z.string(), authorities: z.array(ownership),
      start_offset: count, end_offset: count }).passthrough())
  }).passthrough(), complete: z.boolean(), continuation: action.nullable() }),
  related_context: z.object({ ...bounded, ...links, ...page, mode: z.enum(['context', 'links']),
    passages: z.array(passage), relations: z.array(z.object({ section_id: z.string(), relation: z.string() })) }),
  compare_context: z.object({ ...bounded, ...links, ...page, passages: z.array(passage),
    truth_candidates: z.array(z.string()), resolution: z.string(), conflict_detection: z.string(), conflict_policy: z.string() }),
  kb_status: z.object({ ...envelope, documents: count, sections: count }).passthrough(),
  list_documents: z.object({ ...envelope, documents: z.array(z.object({ id: z.string(), path: z.string(),
    title: z.string(), uri: z.string(), authorities: z.array(authority) }).passthrough()), total: count,
    next_offset: count.nullable() })
};
