"""Stdlib executable-line coverage of the new launcher/packager, with platform effects mocked."""
import json
from pathlib import Path
import sys
import trace
import unittest


def main():
    """Run the real policy/file tests; never import an installed coverage package."""
    folder=Path(__file__).resolve().parent
    def tests():
        """Discovery imports the two measured modules under the trace observer."""
        suite=unittest.defaultTestLoader.discover(str(folder),pattern='test_review*.py')
        return unittest.TextTestRunner(verbosity=1).run(suite).wasSuccessful()
    observer=trace.Trace(count=True,trace=False,ignoredirs=[sys.base_prefix])
    if not observer.runfunc(tests):raise SystemExit(1)
    counts=observer.results().counts
    measured={}
    for name in ('manual_ios.py','review_assets.py'):
        path=folder/name
        # Match stdlib trace's own executable-line convention, without deleting
        # declarations or failure branches just because this host cannot execute iOS.
        lines=set(trace._find_executable_linenos(str(path)))
        covered={line for line in lines if counts.get((str(path),line),0)>0}
        measured[name]=dict(covered=len(covered),executable=len(lines),percent=100*len(covered)/len(lines),
                            uncovered=sorted(lines-covered))
    Path(sys.argv[1]).write_text(json.dumps(measured,indent=2)+'\n')
    print(json.dumps(measured,indent=2))
    if not all(value['percent']>=70 for value in measured.values()):raise SystemExit('Python development coverage floor')


if __name__=='__main__':main()
