# Strict values and statement outcomes

The adapter retains the core value set: Null, Bool, Integer(i64), finite Real(f64), Text and
Blob. It does not guess values from strings or add decimal, unsigned, date or UUID variants.
Native metadata determines result decoding, including NULL columns.

## Type matrix

Use explicit casts or typed columns when the server cannot infer a supported parameter type.

| Advertised MySQL type | Owned result | Bind contract |
| --- | --- | --- |
| Signed integer families | `Integer(i64)`, exactly widened | Integer uses native signed i64 |
| Unsigned integer families | Integer through `i64::MAX`; larger values fail `Decode` | No unsigned SqlValue |
| BOOL / TINYINT(1) | Integer; Row bool conversion accepts only 0/1, rejecting 2 | Bool has a native typed bind |
| FLOAT / DOUBLE | Finite `Real(f64)`; f32 exactly widened | Finite Real uses native f64 |
| UTF-8 CHAR / VARCHAR / TEXT families | `Text` | Text; DateTime uses validated canonical UTC text |
| BINARY / VARBINARY / BLOB families | `Blob`, including empty or non-UTF-8 bytes | Blob; wire charset/type flags distinguish text |
| JSON | Validated canonical `Text` | Text for an explicit JSON cast/column; server validates input |
| Supported type with SQL NULL | `Null` | Native NULL bit; casts/columns supply the target |
| DECIMAL, date/time/datetime/timestamp, ENUM/SET, geometry, other unsupported types | `Decode`, including typed NULL | Explicitly cast output to a supported wire type |

MySQL BOOL is a TINYINT alias, so native rows remain Integer. Use `Row::try_get::<bool>`
only for exact 0/1 values; 2 is not truthy. Unsigned i64 overflow is rejected even if the
native server accepts the value. NaN/infinities are invalid core inputs; strict MySQL can
reject nonfinite arithmetic before any output is decoded.

A binary text collation such as utf8mb4_bin remains Text. SQLx's display name can
say VARBINARY/BLOB for its BINARY flag; the public string compatibility check uses
the wire charset to distinguish such text from actual binary data. Unsupported
ENUM/SET types remain rejected before this check.

A bare Blob parameter result can advertise text metadata. Use `CAST(? AS BINARY)` or a
binary column when a Blob result is required. UUID() may produce supported text; that is
not a native UUID codec. Unsupported output requires an explicit supported cast.

The DateTime schema/rendering convention uses validated UTC text in VARCHAR(30), with no
implicit timezone conversion; it does not enable a native date/time codec.

## JSON is validated canonical text

SQL NULL becomes `SqlValue::Null`. JSON null becomes `SqlValue::Text("null".into())`.
Ordinary TEXT is never inferred to be JSON.

Native JSON is parsed, compacted and recursively key-sorted into Text, with normalized string
escaping and raw numeric wire tokens preserved without an f64 conversion. Duplicate object
keys visible on the wire are rejected. Traversal is limited to 128 levels. The server may
already have normalized JSON numbers or duplicate keys before sending them; original input
spelling cannot be reconstructed.

This deterministic encoding is not an RFC 8785 numeric-normalization promise. JSON wire
input must fit the remaining byte budget before parsing, and serialization is separately
bounded. See [retained payload limits](errors-and-limits.md).

## Rows and outcomes

Column order and duplicate names are preserved; ambiguous name lookup fails. Invalid UTF-8
and nonfinite output fail explicitly. A decode or limit failure returns no partial result.

Native preparation on the same lease checks missing/extra bind counts, including casts;
a literal question mark in a string is not a parameter. Values remain separate native binds.

`QueryResult`'s `affected` field comes from this statement's completion.
`insert_id` is None for a native zero ID; positive IDs are checked into i64. Overflow
fails decoding after the write may have succeeded. No cached/sticky last-ID query is used.
Do not assume a previous INSERT's outcome survives an UPDATE, SELECT or DDL statement.

Source: the Rustdoc-included [codec and lease contract](../../CODECS.md).
