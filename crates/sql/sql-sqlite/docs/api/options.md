# SqliteOptions API

SqliteOptions is owned, serializable configuration for one native file pool. It implements
Clone and Debug; cloning copies options, not a database. Source: [options.rs](../../src/options.rs).

Contents: [Fields](#fields-and-serialization) · [1. File](#1-sqliteoptionsfile) ·
[2. Creation](#2-create_if_missing) · [3. Read-only](#3-read_only) · [4. Validate](#4-validate).

## Fields and serialization

All seven fields are public. Builder setters consume and return Self; assign other budgets
directly before validating or opening. There is no Default implementation.

| Field | Type | `file()` default | Validation |
| --- | --- | --- | --- |
| `path` | `PathBuf` | Caller-supplied | Nonempty UTF-8; no controls, exact `:memory:` or leading `file:`. |
| `create_if_missing` | `bool` | `false` | Cannot combine with read-only. |
| `read_only` | `bool` | `false` | Cannot combine with creation. |
| `max_connections` | `u32` | `4` | `1..=128` |
| `acquire_timeout_ms` | `u64` | `5000` | `1..=300_000` |
| `busy_timeout_ms` | `u64` | `1000` | `1..=300_000` |
| `max_in_flight` | `usize` | `128` | `1..=4096`, including held transactions. |

Serialize/Deserialize use field names as written and reject unknown fields. Only
`read_only` has a missing-field default (`false`); deserialization is not a substitute for
`validate()`. Path validation does not check existence, canonicalize, create parent
directories or enforce a filesystem sandbox. Relative paths use process cwd.

## 1. SqliteOptions::file

`file(path: impl Into<PathBuf>) -> Self`

Choose an explicit existing file with the defaults above. Construction performs no IO and
does not validate until you call validate or open.

```rust
use stackpress_sql_sqlite::SqliteOptions;

fn example() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let options = SqliteOptions::file(root.path().join("profiles.sqlite"));
    assert_eq!(options.max_connections, 4);
    assert!(!options.create_if_missing);
    options.validate()?;
    Ok(())
}
```

The temporary path need not exist for validation to succeed. Opening it without creation
would fail natively.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `impl Into<PathBuf>` | Application-selected filename, not a SQLite URI. |

**Returns** Owned options with no database connection.

## 2. create_if_missing

`create_if_missing(self, create: bool) -> Self`

Opt into file creation for explicit installation. This does not install tables or seed rows.

```rust
use stackpress_sql_sqlite::SqliteOptions;

fn example() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let options = SqliteOptions::file(root.path().join("new.sqlite")).create_if_missing(true);
    options.validate()?;
    assert!(options.create_if_missing);
    Ok(())
}
```

Creation still requires an existing parent directory and native permission at open time.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `create` | `bool` | Whether native open may create a missing file; default false. |

**Returns** Self for chaining. Conflicts are rejected by validation, not by the setter.

## 3. read_only

`read_only(self, read_only: bool) -> Self`

Open an existing file without permitting writes or changing its journal mode.

```rust
use stackpress_sql_sqlite::SqliteOptions;

fn example() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let options = SqliteOptions::file(root.path().join("existing.sqlite")).read_only(true);
    options.validate()?;
    assert!(options.read_only);
    assert!(!options.create_if_missing);
    Ok(())
}
```

Validation checks configuration only. Open later verifies that a readable file exists;
read-only does not itself verify application schema.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `read_only` | `bool` | Read-only native open; default false. |

**Returns** Self for chaining. `read_only(true).create_if_missing(true)` is invalid.

## 4. validate

`validate(&self) -> stackpress_sql::Result<()>`

Reject invalid paths, conflicting flags and budgets outside the allowed ranges before
opening a worker. `SqliteDatabase::open` always calls this method too.

```rust
use stackpress_sql::ErrorKind;
use stackpress_sql_sqlite::SqliteOptions;

fn example() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let mut options = SqliteOptions::file(root.path().join("bounded.sqlite"));
    options.max_connections = 1;
    options.max_in_flight = 8;
    options.validate()?;
    options.acquire_timeout_ms = 0;
    assert_eq!(
        options.validate().unwrap_err().kind(),
        ErrorKind::Configuration
    );
    Ok(())
}
```

The invalid timeout returns the generic configuration message without disclosing the path.

**Returns** Unit on valid configuration, or SqlError with `ErrorKind::Configuration`.
No existence, permission, schema or native-open guarantee follows from success.

Related: [SqliteDatabase::open](database.md#1-sqlitedatabaseopen),
[file and pool concepts](../concepts.md#file-policy-comes-from-the-application),
[server configuration](../guides/server-service.md).
