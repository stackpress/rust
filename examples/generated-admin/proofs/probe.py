"""Finite actual HTTP/SQLite bootstrap probe; each request is killed/reaped after 15 seconds."""
import json
import subprocess
import tempfile
from pathlib import Path
from .runtime import ROOT


def request(binary, database, path='/', method='GET', body=None, actor='editor', mount='/', **options):
    """Pass trusted fixture configuration separately from the intercepted browser request."""
    payload = dict(database=str(database), path=path, method=method, body=body,
                   actor=actor, mount=mount, headers={'origin':'https://admin.invalid',
                   'x-admin-csrf':'finite-session-csrf'}, **options)
    result = subprocess.run([str(binary), '--request'],
                            input=json.dumps(payload), capture_output=True, text=True,
                            timeout=15, check=True)
    return json.loads(result.stdout)


def probe(ctx):
    """Create through generated handlers, then read actual server-rendered pages."""
    LOGS = ctx.logs
    binary = LOGS / "admin-web"
    LOGS.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=LOGS) as temporary:
        database=Path(temporary) / 'probe.sqlite'
        assert request(binary,database,initialize=True)['status']==200
        created=request(binary,database,'/api/profile/create','POST',{'input':{'id':'p1','name':'Parent'}})
        assert created['status']==200,created
        page=request(binary,database,'/profile/search')
        assert page['status']==200 and 'Parent' in page['body'],page
        (LOGS / 'first-page.html').write_text(page['body'])
        denied=request(binary,database,'/profile/create',actor='reader')
        assert denied['status']==403,denied
        failed=request(binary,database,'/api/profile/create','POST',{'input':{'id':'bad','name':''}})
        assert failed['status']==400,failed
        unknown=request(binary,database,'/api/profile/create','POST',{'input':{'id':'uncertain','name':'Uncertain'}},fault=True)
        assert unknown['status']==500 and json.loads(unknown['body'])['errors']['outcome']=='unknown',unknown
        assert request(binary,database,'/api/profile/get','POST',{'key':{'id':'uncertain'}})['status']==200
        print(json.dumps({'created':created['status'],'page':page['status'],'denied':denied['status'],'validation':failed['status'],'uncertain':unknown['status']}))

