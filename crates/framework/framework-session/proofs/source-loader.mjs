// Read-only source loader: TypeScript imports spell .js, while the checkout holds .ts.
import { access } from 'node:fs/promises';
export async function resolve(specifier, context, next) {
  if (specifier.endsWith('.js') && (specifier.startsWith('.') || specifier.startsWith('file:'))) {
    const url = new URL(specifier, context.parentURL);
    try { await access(url); } catch {
      const source = new URL(url.href.replace(/\.js$/, '.ts'));
      try { await access(source); return {url: source.href, shortCircuit: true}; } catch {}
    }
  }
  return next(specifier, context);
}
// The original generated code uses type-only default imports without `import type`.
// Its TypeScript compiler elides those; Node's whitespace-only stripper does not.
// Reuse the upstream installed compiler to reproduce the package's emit semantics.
export async function load(url, context, next) {
  if (url.includes('/packages/stackpress-session/src/') && url.endsWith('.ts')) {
    const {readFile}=await import('node:fs/promises');
    const upstream=url.slice(0,url.indexOf('/packages/stackpress-session/src/'));
    const {default:ts}=await import(upstream+'/node_modules/typescript/lib/typescript.js');
    const source=await readFile(new URL(url),'utf8');
    return {format:'module',source:ts.transpileModule(source,{compilerOptions:{module:ts.ModuleKind.ESNext,target:ts.ScriptTarget.ES2022}}).outputText,shortCircuit:true};
  }
  return next(url,context);
}
