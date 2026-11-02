//! Explicit generated [%model%] store; every operation borrows its executor.
use crate::generated::contracts::[%module%]::{[%model%] as _Stored, Create[%model%] as _Create};
use crate::generated::contracts::public::{[%model%]Key as Key, Patch[%model%] as Patch};
use stackpress_framework_sql::{DataError, DataErrorKind, DataResult, DataSession, StorageColumn, StorageCodec};
use stackpress_framework_sql::stores::{Exposure,[%#?:has_relations%]Loaded,[%/?:has_relations%]Page, ReadQuery, RelationLimits, StoreSpec, Visibility};
use stackpress_sql::{Insert, Row, Select, TableDefinition};
const _: () = assert!(stackpress_framework_sql::CONTRACT_REVISION == [%revision%]);
/// Shared normalized schema identity.
pub const FINGERPRINT: &str = [%fingerprint_literal%];
/// Exact ordered stored codecs.
pub const COLUMNS: &[StorageColumn] = &[
[%#@:columns column%]    StorageColumn {
        wire: [%column.wire_literal%],
        column: [%column.column_literal%],
        codec: StorageCodec::[%column.codec%],
        required: [%column.required%],
        nonempty: [%column.nonempty%],
    },
[%/@:columns%]];
