"""Generate the disposable Profile-column consumer without changing the shipped blog schema."""
from pathlib import Path
import sys

BLOG = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(BLOG))
from proof import bounded


def main():
    """Use the public full emitter, the current lock and the same app family in the fixture."""
    original = (BLOG / 'schema.idea').read_bytes()
    source = original.decode()
    header = 'model Profile {\n'
    assert source.count(header) == 1
    source = source.replace(header, header + '  nickname String? @label("Nickname") @searchable @field.string @list.string @view.string\n', 1)
    root = BLOG / '.build/extension'
    root.mkdir(parents=True, exist_ok=True)
    entry = root / 'schema.idea'
    entry.write_text(source)
    command = ['cargo', 'run', '--manifest-path', str(BLOG / 'Cargo.toml'), '-p', 'blog-tool',
               '--locked', '--offline', '--', 'generate-app', '--input', str(entry),
               '--output', str(root / 'generated')]
    try:
        code = bounded(command, 120)
        if code:
            return code
        return bounded(command + ['--check'], 120)
    finally:
        assert (BLOG / 'schema.idea').read_bytes() == original


if __name__ == '__main__':
    raise SystemExit(main())
