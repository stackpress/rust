/// Only non-key, non-generated members can be patched.
const MUTABLE_COLUMNS: &[StorageColumn] = &[
[%#@:columns column%]    StorageColumn {
        wire: [%column.wire_literal%],
        column: [%column.column_literal%],
        codec: StorageCodec::[%column.codec%],
        required: [%column.required%],
        nonempty: [%column.nonempty%],
    },
[%/@:columns%]];
