#!/usr/bin/env python3
"""Check that every release candidate's crate name is still unclaimed on crates.io.

Run from the repository root right before publishing:

    python3 tools/release/check_names.py

Each name is looked up read-only through the public crates.io API. A 404 means
the name is free. The script exits non-zero if any name is taken or could not be
checked. It sends no credentials or personal details, and it waits one second
between requests to respect the crates.io crawler policy.
"""
import sys
import time
import urllib.error
import urllib.request

import common

API = "https://crates.io/api/v1/crates/"
# crates.io rejects requests without a User-Agent; this one identifies only the tool.
USER_AGENT = "stackpress-release-name-check"


def status(name):
    """Return the HTTP status crates.io gives for this crate name."""
    request = urllib.request.Request(API + name, headers={"User-Agent": USER_AGENT})
    try:
        with urllib.request.urlopen(request, timeout=15) as response:
            return response.status
    except urllib.error.HTTPError as error:
        return error.code


def main():
    """Check every candidate name and report free, taken or unknown."""
    taken, unknown = [], []
    names = [name for _, name in common.candidates()]
    for index, name in enumerate(names):
        # Respect crates.io's one-request-per-second guidance after the first lookup.
        if index:
            time.sleep(1)
        try:
            code = status(name)
        except (urllib.error.URLError, TimeoutError) as error:
            code = None
            reason = str(error)
        if code == 404:
            print(f"free   {name}")
        elif code == 200:
            print(f"TAKEN  {name}")
            taken.append(name)
        else:
            print(f"check  {name} ({'HTTP ' + str(code) if code else reason})")
            unknown.append(name)
    print(f"\n{len(names)} names: {len(names) - len(taken) - len(unknown)} free, "
          f"{len(taken)} taken, {len(unknown)} not checked")
    return 1 if taken or unknown else 0


if __name__ == "__main__":
    sys.exit(main())
