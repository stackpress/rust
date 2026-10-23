#!/usr/bin/env node
import { parseArgs } from 'node:util';
import { loadConfig } from './config.mjs';
import { buildIndex, loadSnapshot, snapshotStatus } from './snapshots.mjs';
import { startServer } from './server.mjs';

async function main() {
  const { values, positionals } = parseArgs({ allowPositionals: true, options: {
    config: { type: 'string' }, transport: { type: 'string', default: 'stdio' },
    force: { type: 'boolean', default: false }, watch: { type: 'boolean', default: false },
    help: { type: 'boolean', default: false }
  } });
  if (values.help) {
    console.log('node cli.mjs index|serve|status [--config path] [--force] [--transport stdio|http] [--watch]');
    return;
  }
  const command = positionals[0];
  if (positionals.length !== 1 || !['index', 'serve', 'status'].includes(command)) throw new Error('Choose index, serve, or status. Use --help for syntax.');
  const config = await loadConfig(values.config);
  if (command === 'status') {
    console.log(JSON.stringify(await snapshotStatus(config, await loadSnapshot(config)), null, 2));
    return;
  }
  const snapshot = await buildIndex(config, { force: values.force });
  if (command === 'index') { console.log(JSON.stringify(snapshot.manifest, null, 2)); return; }
  const handle = await startServer(config, values.transport);
  console.error(`${config.name}: ${values.transport} ready; generation ${snapshot.manifest.generation}`);
  let updating = false;
  const timer = values.watch ? setInterval(async () => {
    if (updating) return;
    updating = true;
    try { await buildIndex(config); }
    catch (error) { console.error(`Reindex failed; prior snapshot retained: ${error.message}`); }
    finally { updating = false; }
  }, 3000) : null;
  timer?.unref();
  const close = async () => { if (timer) clearInterval(timer); await handle.close(); };
  // Let Node dispose native inference resources naturally after closing I/O.
  const shutdown = () => void close().catch(error => { console.error(error.message); process.exitCode = 1; });
  process.once('SIGINT', shutdown);
  process.once('SIGTERM', shutdown);
}

main().catch(error => { console.error(error.message); process.exitCode = 1; });
