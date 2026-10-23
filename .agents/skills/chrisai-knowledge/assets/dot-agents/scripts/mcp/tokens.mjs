import { Tiktoken } from 'js-tiktoken/lite';
import cl100k from 'js-tiktoken/ranks/cl100k_base';

const encoder = new Tiktoken(cl100k);
// Treat source text resembling special tokens as ordinary source text.
export const responseTokens = text => encoder.encode(text, [], []).length;

/** Keep contextual metadata within a quarter of the embedding input limit. */
export function embeddingPrefix(document, section, tokenizer) {
  const context = [...new Set([document.title, section.heading].filter(Boolean))].join(' > ');
  const ceiling = Math.min(64, Math.floor(tokenizer.limit / 4));
  let prefix = '';
  for (const character of context) {
    if (tokenizer.count(`${prefix}${character}\n\n`) > ceiling) break;
    prefix += character;
  }
  return prefix ? `${prefix}\n\n` : '';
}

export function splitText(text, count, limit) {
  const parts = [];
  const offsets = [0];
  for (const character of text) offsets.push(offsets.at(-1) + character.length);
  let start = 0;
  while (start < text.length) {
    let end = text.length;
    if (count(text.slice(start, end)) > limit) {
      let low = offsets.indexOf(start) + 1, high = offsets.length - 1;
      end = start;
      while (low <= high) {
        const mid = Math.floor((low + high) / 2);
        if (count(text.slice(start, offsets[mid])) <= limit) { end = offsets[mid]; low = mid + 1; }
        else high = mid - 1;
      }
      // Prefer paragraph/word boundaries, but never discard whitespace or bytes.
      const prefix = text.slice(start, end);
      const breaks = [...prefix.matchAll(/\s+/gu)];
      const boundary = breaks.at(-1);
      const candidate = boundary ? start + boundary.index + boundary[0].length : end;
      if (candidate > start + (end - start) / 2 && count(text.slice(start, candidate)) <= limit) end = candidate;
    }
    if (end <= start || count(text.slice(start, end)) > limit) throw new Error('Cannot split text within the embedding token limit.');
    parts.push({ start, end, text: text.slice(start, end), tokens: count(text.slice(start, end)) });
    start = end;
  }
  return parts;
}
