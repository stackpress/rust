// Run the official patcher after hydration. Its warnings are caught internally, so
// forward them to xtask as a rebuild request rather than accepting a stale page.
const applyBlogPatch = () => {
  if (document.readyState === "loading" || !document.documentElement.dataset.blogReady) {
    if (Date.now() - receivedAt < 30000) {
      setTimeout(applyBlogPatch, 25);
      return;
    }
    ws.send(JSON.stringify({ reload: true, revision: msg.revision }));
    return;
  }
  let failed = Boolean(ws.blogPatchFailed);
  const warn = console.warn;
  try {
    const patches = JSON.parse(msg.view);
    const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_COMMENT);
    const markers = new Set();
    while (walker.nextNode()) markers.add(walker.currentNode.textContent);
    // CSR creates no view markers; changed inactive/missing views also take the safe rebuild.
    if (!patches.every(([id]) => markers.has(`hot-reload|${id}|open`) && markers.has(`hot-reload|${id}|close`))) {
      failed = true;
    }
    if (!failed) {
      console.warn = (...args) => { failed = true; warn.apply(console, args); };
      patch(msg.view);
    }
  } catch (error) {
    failed = true;
    warn.call(console, "Blog hot patch failed; requesting a full rebuild", error);
  } finally {
    console.warn = warn;
  }
  if (failed) {
    ws.blogPatchFailed = true;
    ws.send(JSON.stringify({ reload: true, revision: msg.revision }));
  }
};
const receivedAt = Date.now();
applyBlogPatch();
