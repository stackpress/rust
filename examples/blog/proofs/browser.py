#!/usr/bin/env python3
"""Run finite cached Playwright against in-process HTTP routing, never the managed server."""
import argparse
import os
from pathlib import Path
import sys

BLOG = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(BLOG))
from proof import bounded


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--public-only', action='store_true')
    parser.add_argument('--mode', choices=['hydrate', 'csr', 'both'], default='both')
    parser.add_argument('--matrix', action='store_true')
    args = parser.parse_args()
    os.environ['BLOG_PLAYWRIGHT'] = '/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright'
    modes = ['hydrate', 'csr'] if args.mode == 'both' else [args.mode]
    sizes = [(1280, 1), (375, 1), (375, 2)] if args.matrix else [(1280, 1)]
    for mode in modes:
        for width, zoom in sizes:
            os.environ.update(BLOG_BROWSER_MODE=mode, BLOG_BROWSER_WIDTH=str(width), BLOG_BROWSER_ZOOM=str(zoom))
            command = ['node', str(BLOG / 'tests/browser.cjs')]
            if args.public_only:
                command.append('--public-only')
            if bounded(command, 120):
                return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
