#!/usr/bin/env python3
"""Compatibility exports for the shared bounded runtime; prefer proof.py for new commands."""
import os
import sys
from proofs.runtime import Context, ROOT, TARGET

_context = Context('format')
LOGS, run = _context.logs, _context.run

if __name__ == '__main__':
    run(sys.argv[1], sys.argv[2:], timeout=min(120,int(os.environ.get('ADMIN_TIMEOUT','120'))) if sys.argv[1].startswith('browser') else int(os.environ.get('ADMIN_TIMEOUT','900')))
