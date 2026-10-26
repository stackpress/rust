// Inert script text is admitted by natively tested Rust before hydration starts.
import init, { start, corpus_json } from './gallery.js';
const page = document.querySelector('#stackpress-page')?.textContent;
const replay = document.querySelector('#stackpress-resources')?.textContent;
try {
  await init();
  window.galleryCorpus = corpus_json();
  window.disposeGallery = start(page || '', replay || '');
  window.galleryReady = true;
} catch (error) {
  document.querySelector('#app').textContent = 'Refresh to continue.';
  throw error;
}
