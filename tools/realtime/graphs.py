#!/usr/bin/env python3
"""Prove normal-edge feature isolation without resolving or editing any lock."""
import subprocess
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]

def graph(package, features='', target=None):
    """Read exactly the selected normal dependency graph from the frozen root lock."""
    command=['cargo','tree','--locked','--offline','--no-default-features','-p',package,'-e','normal','--prefix','none']
    if features: command += ['--features',features]
    if target: command += ['--target',target]
    result=subprocess.check_output(command,cwd=ROOT,text=True)
    return {line.split()[0] for line in result.splitlines() if line.strip()}

def main():
    frozen=(ROOT/'Cargo.lock').read_bytes()
    for package,feature in [('stackpress',''),('stackpress-server','http'),('stackpress-server','http-sse'),('stackpress-server-http',''),('stackpress-server-http','sse')]:
        names=graph(package,feature)
        assert not names & {'tungstenite','tokio-tungstenite'},(package,feature,names)
        if package=='stackpress-server': assert not names & {'tokio','hyper'},names
        print('PASS disabled transport graph',package,feature)
    for features in ['websocket','sse,websocket','tls,sse,websocket']:
        assert {'tungstenite','tokio-tungstenite'} <= graph('stackpress-server-http',features)
    for package,features in [('stackpress-leptos','hydrate,client-sse,client-websocket'),('stackpress-leptos','csr,client-sse,client-websocket'),('stackpress','leptos-hydrate,leptos-sse,leptos-websocket')]:
        names=graph(package,features,'wasm32-unknown-unknown')
        assert not names & {'tokio','hyper','tungstenite','tokio-tungstenite','stackpress-server-http'},names
        print('PASS browser graph',package,features)
    assert (ROOT/'Cargo.lock').read_bytes()==frozen
if __name__=='__main__':main()
