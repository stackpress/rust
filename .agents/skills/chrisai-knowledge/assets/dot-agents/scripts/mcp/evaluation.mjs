import { performance } from 'node:perf_hooks';
import { responseTokens } from './tokens.mjs';

/** Normalize evidence returned by search, section fetches, or document pages. */
export function responsePassages(result) {
  if (result.passages) return result.passages;
  if (result.passage) return [result.passage];
  if (!result.document) return [];
  const doc = result.document;
  return doc.sections.map(section => ({ ...section, path: doc.path,
    text: doc.text.slice(section.start_offset, section.end_offset) }));
}

/** Record a real MCP call and one-copy payload cost, including failures. */
export async function measuredCall(callTool, name, args) {
  const start = performance.now();
  const raw = await callTool({ name, arguments: args });
  if (raw.isError) throw new Error(raw.content?.find(item => item.type === 'text')?.text || `${name} failed`);
  const result = raw.structuredContent ?? raw;
  return { tool: name, arguments: args, latency_ms: Math.round((performance.now() - start) * 100) / 100,
    tokens: responseTokens(JSON.stringify(result)), result };
}

/** Evaluate known evidence against actual search output without inventing an answer. */
export async function evaluateRetrieval(cases, callTool) {
  validateCases(cases);
  const results = [];
  let generation;
  for (const fixture of cases) {
    const call = await measuredCall(callTool, 'search_kb', { ...fixture.search, ...(generation ? { generation } : {}) });
    generation ??= call.result.generation;
    const passages = responsePassages(call.result);
    const checks = (fixture.expected_evidence ?? []).map(expected => ({ ...expected,
      found: passages.some(p => p.path === expected.path && (!expected.contains || p.text.includes(expected.contains))
        && (!expected.authority || p.authorities.some(a => a.authority === expected.authority))) }));
    const forbidden = passages.filter(p => (fixture.forbidden_paths ?? []).some(prefix => p.path.startsWith(prefix)));
    const noMatch = fixture.expected_no_match === undefined || call.result.no_match === fixture.expected_no_match;
    const incomplete = fixture.expected_incomplete === undefined || call.result.expansion.complete === !fixture.expected_incomplete;
    const mode = fixture.expected_mode === undefined || call.result.effective_mode === fixture.expected_mode;
    const passed = checks.every(c => c.found) && !forbidden.length && noMatch && incomplete && mode && !call.result.stale;
    results.push({ id: fixture.id, passed, evidence: checks, forbidden_paths: forbidden.map(p => p.path),
      no_match_check: noMatch, completeness_check: incomplete, effective_mode_check: mode, calls: [call] });
  }
  return { evaluated_at: new Date().toISOString(), generation, kind: 'retrieval', passed: results.every(r => r.passed),
    cases: results, metrics: metrics(results), answer_quality: 'not-evaluated' };
}

/** Export only questions and retrieved material; keep answer keys away from the consumer. */
export function consumerQuestions(cases, report) {
  return cases.map(fixture => ({ id: fixture.id, question: fixture.question,
    claim_keys: [...new Set((fixture.expected_claims ?? []).map(c => c.key))],
    initial_result: report.cases.find(result => result.id === fixture.id).calls[0].result }));
}

/** Check structured facts and exact citations; natural-language reasoning still needs review. */
export function evaluateAnswers(cases, answers, report, extraCalls = {}) {
  validateCases(cases);
  if (!Array.isArray(answers) || new Set(answers.map(a => a.id)).size !== answers.length) throw new Error('Answers need unique case IDs.');
  if (answers.some(a => !cases.some(c => c.id === a.id))) throw new Error('Unknown answer case ID.');
  const results = cases.map(fixture => {
    const answer = answers.find(a => a.id === fixture.id);
    const calls = [...report.cases.find(c => c.id === fixture.id).calls, ...(extraCalls[fixture.id] ?? [])];
    const evidence = calls.flatMap(call => responsePassages(call.result));
    const claims = Array.isArray(answer?.claims) ? answer.claims : [];
    const correct = (fixture.expected_claims ?? []).map(expected => ({ key: expected.key, expected: expected.value,
      correct: claims.some(claim => claim.key === expected.key && JSON.stringify(claim.value) === JSON.stringify(expected.value)
        && claim.authority === expected.authority) }));
    const citations = claims.map(claim => ({ key: claim.key, valid: Boolean(claim.citations?.length) && claim.citations.every(cite =>
      typeof cite.quote === 'string' && cite.quote.trim().length > 0 && evidence.some(p => p.id === cite.section_id
        && p.path === cite.path && p.text.includes(cite.quote) && p.authorities.some(a => a.authority === claim.authority))) }));
    const flags = Boolean(answer && typeof answer.answer === 'string' && answer.answer.trim())
      && answer.cannot_answer === (fixture.cannot_answer ?? false) && answer.needs_review === (fixture.needs_review ?? false)
      && (fixture.required_limitations ?? []).every(value => answer.limitations?.includes(value));
    const generations = calls.every(call => call.result.generation === report.generation && !call.result.stale);
    return { id: fixture.id, passed: Boolean(answer) && flags && generations && correct.every(c => c.correct) && citations.every(c => c.valid),
      facts: correct, citations, answer_flags_correct: flags, consistent_fresh_generation: generations,
      answer: answer?.answer ?? null, calls };
  });
  return { evaluated_at: new Date().toISOString(), generation: report.generation, kind: 'consumer-answer-checks',
    automated_checks_passed: results.every(r => r.passed), prose_review_required: true,
    note: 'Checks structured expected facts, authority, and exact quotes in observed tool output. Does not establish semantic entailment or prose correctness.',
    cases: results, metrics: metrics(results) };
}

/** Reject empty or ambiguous fixtures so an empty run cannot report success. */
function validateCases(cases) {
  if (!Array.isArray(cases) || !cases.length || cases.some(c => !c.id || !c.question || !c.search?.query)
    || new Set(cases.map(c => c.id)).size !== cases.length) throw new Error('Evaluation needs nonempty cases with unique IDs, questions, and search queries.');
}

/** Compare latency and response tokens separately from answer quality. */
function metrics(cases) {
  const calls = cases.flatMap(result => result.calls), times = calls.map(c => c.latency_ms).sort((a, b) => a - b);
  const expected = cases.flatMap(c => c.evidence ?? []);
  return { cases: cases.length, passed_cases: cases.filter(c => c.passed).length, tool_calls: calls.length,
    response_tokens: calls.reduce((sum, c) => sum + c.tokens, 0),
    total_tool_latency_ms: Math.round(calls.reduce((sum, c) => sum + c.latency_ms, 0) * 100) / 100,
    p95_tool_latency_ms: times[Math.max(0, Math.ceil(times.length * 0.95) - 1)] ?? 0,
    evidence_coverage: expected.length ? expected.filter(c => c.found).length / expected.length : null };
}
