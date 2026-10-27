#!/usr/bin/env python3
"""Scoped post-refactoring checks for the explicitly selected debug review host."""
import argparse
from pathlib import Path
import review


def main():
    """Run meaningful tests followed by warning-clean lint/docs and final formatting."""
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--directory',type=Path,required=True)
    args=parser.parse_args();ctx=review.context(args.directory)
    scope=['-p',review.PACKAGE,'--features','review-preview','--offline','--locked']
    ctx.run('round-3-tests',['cargo','test',*scope])
    ctx.run('round-3-clippy',['cargo','clippy',*scope,'--all-targets','--','-D','warnings'])
    ctx.run('round-3-rustdoc',['cargo','doc',*scope,'--no-deps'],extra={'RUSTDOCFLAGS':'-D warnings'})
    ctx.run('round-3-fmt',['cargo','fmt','--all','--','--check'])


if __name__=='__main__':main()
