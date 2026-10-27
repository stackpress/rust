"""Read one field from the existing CLI JSON envelope; it grants no execution authority."""

import json
from pathlib import Path
import sys


def main():
    """Print a required scalar result so the reviewed hash can be passed unchanged to the tool."""
    document = json.loads(Path(sys.argv[1]).read_text())
    value = document["results"][sys.argv[2]]
    if not isinstance(value, str) or not value:
        raise ValueError("expected a nonempty CLI result field")
    print(value)


if __name__ == "__main__":
    main()
