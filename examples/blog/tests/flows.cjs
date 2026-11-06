// Real authenticated browser flows; no injected cookies or fabricated model payloads.
const assert = require('node:assert/strict');

/** Submit the live form and observe its native redirect before the next renderer selection. */
async function save(page, label = 'Save') {
  const invalid = await page.locator('form').evaluateAll(forms => forms.flatMap(form =>
    Array.from(form.elements).filter(input => input.willValidate && !input.validity.valid)
      .map(input => ({ name: input.name, message: input.validationMessage }))));
  assert.deepEqual(invalid, [], JSON.stringify(invalid));
  const [response] = await Promise.all([
    page.waitForResponse(reply => reply.request().method() === 'POST'),
    page.getByRole('button', { name: label, exact: true }).click()
  ]);
  // Chromium hides redirect response bodies; read diagnostics only for a failed write.
  assert.equal(response.status(), 302,
    response.status() === 302 ? response.headers() : await response.text());
  const location = response.headers().location;
  // The CDP fixture also routes the browser's actual redirect continuation.
  await page.waitForURL(url => url.pathname === new URL(location, url).pathname);
  // Source redirects omit the proof's renderer query, so the destination uses its default.
  await page.waitForFunction(() => !!document.documentElement.dataset.blogReady);
  return location;
}

/** Fetch through the actual browser cookie jar and the existing in-process HTTP route. */
async function json(page, path, token) {
  return page.evaluate(async ({ path, token }) => {
    const response = await fetch(path, { headers: token ? { Authorization: `Bearer ${token}` } : {} });
    // The app's source error view may render an API denial as HTML; retain its actual status.
    const text = await response.text();
    const data = response.headers.get('content-type')?.includes('application/json')
      ? JSON.parse(text) : text;
    return { status: response.status, data };
  }, { path, token });
}

/** Operate the generated relation control, including its real URL lookup and choice callback. */
async function relation(page, name, query, label) {
  const input = page.getByRole('combobox', { name, exact: true });
  await input.fill(query);
  await page.getByRole('option', { name: label, exact: true }).click();
}

/** Sign in, edit generated controls, create a scoped app, and read the actual account export. */
async function run(page, go, article, mode, nickname) {
  await page.locator('input[name=username]').fill('developer');
  await page.locator('input[name=secret]').fill('developer');
  assert.equal(await save(page, 'Submit'), '/');
  const admin = await go(page, '/admin/article/search');
  assert.equal(admin.status(), 200, 'the real source-issued ADMIN cookie must reach /admin');
  // The source search page names its generated table with the model's singular caption.
  assert.equal(await page.locator('table caption').innerText(), 'Article');

  // The stock seed is intentionally denied by the source endpoint's user scope requirement.
  const applications = await json(page, '/admin/application/search?json');
  assert.equal(applications.status, 200);
  const seeded = applications.data.results[0].id;
  assert.equal((await json(page, '/api/profile/search')).status, 401);
  assert.equal((await json(page, '/api/profile/search', seeded)).status, 401);

  await go(page, '/admin/article/update/' + article.id);
  const title = `Browser article ${mode}`;
  const contents = `Browser editor body ${mode}`;
  await page.getByLabel('Title', { exact: true }).fill(title);
  const editor = page.locator('[contenteditable="true"]');
  assert.equal(await editor.count(), 1, 'Article.contents must use the real rich editor');
  await editor.fill(contents);
  await save(page);
  await go(page, '/articles/' + article.slug);
  assert.equal(await page.locator('.article-title').innerText(), title);
  assert.ok((await page.locator('.article-prose').innerText()).includes(contents));

  await go(page, '/admin/category/create');
  const category = `Browser category ${mode}`;
  await page.getByLabel('Name', { exact: true }).fill(category);
  await page.getByLabel('URL Slug', { exact: true }).fill(`browser-category-${mode}`);
  assert.match(await save(page), /^\/admin\/category\/detail\//);

  // This exercises both relation fetches and the formerly broken compound-key redirect.
  await go(page, '/admin/catalog/create');
  await relation(page, 'Category ID', category, category);
  await relation(page, 'Article ID', title, title);
  assert.equal(await save(page), '/admin/catalog/search');

  await go(page, '/admin/application/create');
  await relation(page, 'Profile', 'Developer', 'Developer');
  await page.getByLabel('Name', { exact: true }).fill(`Browser scoped app ${mode}`);
  const scopes = page.getByRole('group', { name: 'Scopes', exact: true });
  await scopes.getByLabel('Add tag', { exact: true }).fill('user');
  await scopes.getByRole('button', { name: 'Add', exact: true }).click();
  // The shared temporal editor requires an explicit offset for an instant, then commits on blur.
  await page.getByLabel('Expires', { exact: true }).fill('2099-01-01T00:00:00Z');
  await page.getByLabel('Expires', { exact: true }).blur();
  const location = await save(page);
  assert.match(location, /^\/admin\/application\/detail\//);
  const token = decodeURIComponent(location.split('/').at(-1));
  const allowed = await json(page, '/api/profile/search', token);
  assert.equal(allowed.status, 200, JSON.stringify(allowed));
  assert.ok(allowed.data.results.some(row => row.id === 'developer'));

  if (nickname) {
    // This project was regenerated after a real SQLite upgrade; the default schema is untouched.
    const value = `Browser nickname ${mode}`;
    await go(page, '/admin/profile/update/developer');
    await page.getByLabel('Nickname', { exact: true }).fill(value);
    await save(page);
    await go(page, '/admin/profile/detail/developer');
    assert.ok(await page.getByText(value, { exact: true }).count());
    const profiles = await json(page, '/api/profile/search', token);
    assert.equal(profiles.data.results.find(row => row.id === 'developer').nickname, value);
  }
  const exported = await page.evaluate(async () => {
    const response = await fetch('/auth/account/security/export?download=1');
    return { status: response.status, type: response.headers.get('content-type'), text: await response.text() };
  });
  assert.equal(exported.status, 200);
  assert.match(exported.type, /text\/csv/);
  assert.ok(exported.text.includes('name'));
  if (nickname) assert.ok(exported.text.includes(`Browser nickname ${mode}`));
  return { title, token, nickname, exported: true };
}

module.exports = { run };
