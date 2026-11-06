#!/usr/bin/env python3
"""Finite HTTP smoke of the coordinator-owned listener; never start or stop a server."""
import argparse
from html.parser import HTMLParser
from pathlib import Path
import sys
# Executing this named proof must not shadow Python's standard-library http package.
sys.path = [item for item in sys.path if Path(item).resolve() != Path(__file__).resolve().parent]
import http.cookiejar
import json
import urllib.error
import urllib.parse
import urllib.request

BLOG = Path(__file__).resolve().parents[1]


class Document(HTMLParser):
    """Extract inert native seeds and original hidden form fields without executing scripts."""
    def __init__(self, text):
        super().__init__()
        self.fields, self.seeds, self.seed = {}, {}, None
        self.feed(text)

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == 'input' and attrs.get('type') == 'hidden':
            self.fields[attrs.get('name', '')] = attrs.get('value', '')
        if tag == 'script' and attrs.get('type') == 'application/json':
            self.seed = attrs.get('id')
            self.seeds[self.seed] = ''

    def handle_data(self, text):
        if self.seed:
            self.seeds[self.seed] += text

    def handle_endtag(self, tag):
        if tag == 'script':
            self.seed = None


class Redirect(urllib.request.HTTPRedirectHandler):
    """Inspect the real source redirect before choosing the next finite GET."""
    def redirect_request(self, *args, **kwargs):
        return None


def main():
    """Verify real assets/pages/cookies/sign-in and a write on the explicitly owned smoke DB."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('origin')
    parser.add_argument('--public-only', action='store_true', help='prove public reads independently of the enabled sign-in regression')
    args = parser.parse_args()
    origin = args.origin.rstrip('/')
    jar = http.cookiejar.CookieJar()
    client = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(jar), Redirect())
    calls = []

    def request(path, fields=None, headers=None):
        payload = urllib.parse.urlencode(fields).encode() if fields is not None else None
        req = urllib.request.Request(origin + path, data=payload, headers=headers or {})
        try:
            response = client.open(req, timeout=10)
        except urllib.error.HTTPError as error:
            response = error
        with response:
            body = response.read()
            calls.append({'method': req.get_method(), 'path': path, 'status': response.code,
                          'bytes': len(body), 'type': response.headers.get('Content-Type')})
            # Retain partial evidence if a later real HTTP assertion exposes a framework gap.
            (BLOG / '.build/http-smoke.json').write_text(json.dumps({'origin': origin, 'calls': calls, 'passed': False}, indent=2))
            return response.code, response.headers, body.decode('utf-8', errors='replace')

    status, _, home = request('/')
    assert status == 200 and 'Knowledge Console' in home
    seed = json.loads(Document(home).seeds['stackpress-page'])
    rows = seed['data']['data']
    assert len(rows) == 2 and all(row['status'] == 'PUBLISHED' for row in rows)
    article = rows[0]
    assert request('/articles/' + article['slug'])[0] == 200
    assert request('/articles/absent-managed-smoke')[0] == 404
    assert request('/articles/absent-managed-smoke.js')[0] == 404
    assert json.loads(request('/?json')[2])['code'] == 200
    for path, content in [('/styles/global.css', 'knowledge-hero'),
                          ('/styles/article.css', 'article-prose'),
                          ('/assets/blog.js', '__blogDispose'),
                          ('/assets/hydrate/blog.js', 'WebAssembly'),
                          ('/assets/csr/blog.js', 'WebAssembly')]:
        status, _, body = request(path)
        assert status == 200 and content in body, path
    status, _, signin = request('/auth/signin/username')
    assert status == 200
    if args.public_only:
        assert request('/api/profile/search')[0] == 401
        (BLOG / '.build/http-public-smoke.json').write_text(json.dumps({'origin': origin, 'calls': calls, 'passed': True}, indent=2))
        print(f'Public HTTP smoke passed: {len(calls)} finite requests; authenticated flows remain separate; listener untouched.')
        return
    fields = Document(signin).fields
    fields.update(username='developer', secret='developer')
    status, headers, _ = request('/auth/signin/username', fields)
    assert status in (302, 303), (status, headers)
    assert any(cookie.name == 'session' and cookie.path == '/' for cookie in jar)
    status, _, admin = request('/admin/article/update/' + article['id'])
    assert status == 200 and 'api-management-seed' in admin
    fields = Document(admin).fields
    fields['title'] = 'Managed smoke article'
    status, headers, body = request('/admin/article/update/' + article['id'], fields)
    assert status in (302, 303), (status, body[:400])
    status, _, updated = request('/articles/' + article['slug'])
    assert status == 200 and 'Managed smoke article' in updated
    status, headers, exported = request('/auth/account/security/export?download=1')
    assert status == 200 and 'text/csv' in headers.get('Content-Type', '') and 'name' in exported
    status, _, api = request('/api/profile/search')
    assert status == 401, (status, api)
    output = BLOG / '.build/http-smoke.json'
    output.write_text(json.dumps({'origin': origin, 'calls': calls, 'passed': True}, indent=2))
    print(f'HTTP smoke passed: {len(calls)} finite requests; sign-in/admin write/export; listener untouched.')


if __name__ == '__main__':
    main()
