# Terminal and argument API

Supplied argv parsing, with operator settings separate from capability data. Source:
[args.rs](../../src/args.rs). Terminal/TerminalOptions are Debug; options are Default.

```rust
use stackpress_cli::{ColorMode, Terminal, TerminalOptions};
use stackpress_lib::ParseLimits;
use std::ffi::OsString;
let argv = ["-v", "emit", "search", "--active", "false", "--limit", "0"].map(String::from);
let terminal = Terminal::parse(&argv, ParseLimits::default())?;
```

Each following Rust snippet is independent, using this setup inside an async function returning
`std::result::Result<(), Box<dyn std::error::Error>>`. Named helpers remain ordinary functions.

Contents: [1. Options and errors](#1-options-and-errors) · [2. Parsing](#2-parsing) · [3. Access and
lookup](#3-access-and-lookup)

## 1. Options and errors

TerminalOptions has public fields `verbose: bool` false, `force: bool` false, `bootstrap:
Option<PathBuf>` None, `brand: String` empty, `color: ColorMode` Auto. force carries intent, not
destructive-operation authorization. ArgumentError implements Debug/Display/std::error::Error:
Usage(&'static str), NonUtf8(usize), Data(stackpress_lib::Error). Display never echoes target
payload; Data retains its original source. From<Error> creates Data.

All parser functions exclude the executable name. The full argv counts toward max_bytes/max_entries
before target parsing. Events must be nonblank and contain no controls; host handling separately
enforces its 8,192-byte selector limit. Target data uses [Lib argv
grammar](../../../../crates/lib/docs/api/parsers.md).

## 2. Parsing

Only -v/--verbose, -f/--force, -b/--bootstrap and --bootstrap=path are operators. Short groups such
as -vf, -vfb file.json, -bfile.json, and -vfbfile.json are accepted; b ends its group and consumes
its suffix or the next token as path. A separate path cannot be empty or begin with a dash. Repeated
bootstrap selection keeps the last path. `--` may end operators immediately before literal emit.
After the event every token belongs to the target; no file is read by parsing.

### 2.1. parse

`Terminal::parse(args: &[String], limits: ParseLimits) -> Result<Self, ArgumentError>`

Parse UTF-8 argv with neutral application defaults.

```rust
let parsed = Terminal::parse(&argv, ParseLimits::default())?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `args` | `&[String]` | Operator options, emit, event, then target tokens. |
| `limits` | `ParseLimits` | Full argv and parsed Nest budgets; zero is a ceiling. |

**Returns** A checked immutable Terminal or Usage/Data error for operator syntax, event, budget, or
target grammar. No process argument/file/environment read occurs.

### 2.2. parse_os

`Terminal::parse_os(args: &[OsString], limits: ParseLimits) -> Result<Self, ArgumentError>`

Reject invalid UTF-8 instead of replacing it.

```rust
let args = [OsString::from("emit"), OsString::from("search")];
let parsed = Terminal::parse_os(&args, ParseLimits::default())?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `args` | `&[OsString]` | OS tokens excluding executable; bounded before UTF-8 copies. |
| `limits` | `ParseLimits` | Same budgets as parse. |

**Returns** Terminal or first NonUtf8(index), budget/Data, or ordinary parse error. Index is
zero-based in supplied argv.

### 2.3. parse_with_options

`Terminal::parse_with_options(args: &[String], limits: ParseLimits, options: TerminalOptions) -> Result<Self, ArgumentError>`

Start from app defaults and apply explicit operator overrides.

```rust
let options = TerminalOptions {
    brand: "[app]".into(),
    color: ColorMode::Never,
    ..Default::default()
};
let parsed = Terminal::parse_with_options(&argv, ParseLimits::default(), options)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `args` | `&[String]` | Same supplied token grammar. |
| `limits` | `ParseLimits` | Same budgets. |
| `options` | `TerminalOptions` | Consumed defaults; supplied operator flags may set/replace fields. |

**Returns** Terminal or the same parse errors. There is no argv color/brand flag; those are
application defaults.

## 3. Access and lookup

Terminal retains original target tokens plus eagerly parsed immutable data. Accessors confer no
mutation authority and do not reparse argv.

### 3.1. command

`command(&self) -> &str`

Borrow the selected event.

```rust
assert_eq!(terminal.command(), "search");
```

**Returns** Borrowed checked selector text.

### 3.2. args

`args(&self) -> &[String]`

Borrow original target token boundaries.

```rust
assert_eq!(terminal.args()[0], "--active");
```

**Returns** Borrowed target-only argv, never joined/re-tokenized.

### 3.3. data

`data(&self) -> &Nest`

Borrow eagerly parsed target fields.

```rust
assert_eq!(terminal.data().path_as::<bool>("active")?, Some(false));
```

**Returns** Borrowed Nest with its owning limits.

### 3.4. options

`options(&self) -> &TerminalOptions`

Inspect operator/application metadata.

```rust
assert!(terminal.options().verbose);
```

**Returns** Borrowed options separate from capability data.

### 3.5. control

`control(&self) -> Control`

Create human formatting policy from brand/color.

```rust
let control = terminal.control();
```

**Returns** A new Control with a normalized owned label, without selecting a stream or acquiring
input.

### 3.6. expect

`expect<T: DeserializeOwned>(&self, aliases: &[&str]) -> Result<Option<T>, ArgumentError>`

Decode the first present literal alias, including false/zero/empty/null.

```rust
assert_eq!(terminal.expect::<bool>(&["active", "a"])?, Some(false));
assert_eq!(terminal.expect::<u32>(&["limit"])?, Some(0));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `aliases` | `&[&str]` | Literal key priority; an empty list yields None. |

**Returns** Some(T) for the first present successfully decoded value, None if all absent, or Data
error on that first value's wire/type conversion. No fallback after a type error. For present null,
T = Option<U> yields Some(None). Defaults belong at the call site, e.g. ?.unwrap_or(default).

Related: [argument guide](../guides/arguments.md), [execution](execution.md).
