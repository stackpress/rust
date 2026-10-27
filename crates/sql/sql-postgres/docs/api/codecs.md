# Strict values and statement outcomes

The adapter retains the core value set: Null, Bool, Integer(i64), finite Real(f64), Text and
Blob. It does not guess values from strings or add decimal, unsigned, date or UUID variants.
Native metadata determines result decoding, including NULL columns.

## Type matrix

Use explicit casts or typed columns when the server cannot infer a supported parameter type.

| Advertised PostgreSQL type | Owned result | Bind contract |
| --- | --- | --- |
| BOOL | `Bool` | Bool only |
| INT2 / INT4 / INT8 | `Integer(i64)`, exactly widened | Integer checked against prepared width; int2/int4 overflow fails |
| FLOAT4 / FLOAT8 | Finite `Real(f64)`; f32 exactly widened | Finite Real; FLOAT4 narrowing may round and rejects overflow to infinity |
| TEXT / VARCHAR | UTF-8 `Text` | Text; DateTime uses validated canonical UTC text |
| BYTEA | `Blob`, including empty or non-UTF-8 bytes | Blob |
| JSON / JSONB | Validated canonical `Text` | Validated Text for an explicitly prepared JSON target |
| Supported type with SQL NULL | `Null` | Prepared metadata selects the matching typed NULL |
| NUMERIC, date/time/timestamp, UUID, arrays, enums, custom/unknown OIDs | `Decode`, including typed NULL | `Unsupported`; explicitly cast to a supported target |

Results are decoded by their wire type; domains over supported base types decode as that
base type. PostgreSQL 12 RowDescription erases that domain identity. Any advertised custom,
domain or unknown OID is rejected, including NULL; the adapter performs no catalog lookup
to recover erased identity. Server domain constraints remain enforced on writes. This
wire-type boundary is a coordinator decision characterized by the native domain test.

An untyped/unknown prepared parameter is Unsupported; supply a cast such as `$1::INT4`.
The narrowing check happens before execution. NaN and infinities fail explicitly on input
and output. Unsupported NUMERIC/date/UUID output needs an intentional cast, not a lossy
automatic text fallback.

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
`insert_id` is always None; use supported RETURNING rows for generated keys.
Do not assume a previous INSERT's outcome survives an UPDATE, SELECT or DDL statement.

Source: the Rustdoc-included [codec and lease contract](../../CODECS.md).
