/// Pure inspectable table, query and codec functions.
pub struct [%model%]Store;
impl [%model%]Store {
    /// Build the checked storage definition; installation is always explicit.
    pub fn table() -> DataResult<TableDefinition> {
        let mut table = stackpress_sql::Create::new([%table_literal%]);
[%#@:fields field%]        // Preserve declared nullability and explicit character length in storage.
        let field = stackpress_sql::Field::new(
            [%field.name_literal%], stackpress_sql::SqlType::[%field.kind%]
        )
[%#?:field.not_null%]            .not_null()
[%/?:field.not_null%][%#?:field.has_length%]            .length(stackpress_sql::Length::Size([%field.length%]))
[%/?:field.has_length%]        ;
        table = table.add_field(field);
[%/@:fields%]        table = table.primary_key([[%#@:primary column%][%#!:@first%], [%/!:@first%][%column%][%/@:primary%]]);
[%#@:indexes index%]        // Storage constraints/indexes are authoritative; UI hints cannot replace them.
        table = table.add_index(stackpress_sql::Index::new(
            [%index.name_literal%], [[%#@:index.columns column%][%#!:@first%], [%/!:@first%][%column%][%/@:index.columns%]]
        )[%#?:index.unique%].unique()[%/?:index.unique%]);
[%/@:indexes%][%#@:foreign_keys key%]        // Explicit owning FK; reverse collections have no column.
        let mut foreign = stackpress_sql::ForeignKey::new(
            [%key.name_literal%],
            [[%#@:key.columns column%][%#!:@first%], [%/!:@first%][%column%][%/@:key.columns%]],
            [%key.table_literal%],
            [[%#@:key.foreign_columns column%][%#!:@first%], [%/!:@first%][%column%][%/@:key.foreign_columns%]],
        );
        foreign.on_delete = stackpress_sql::ReferentialAction::Restrict;
        table = table.add_foreign_key(foreign);
[%/@:foreign_keys%][%#?:mysql_target%]        table = table.require_mysql_target();
[%/?:mysql_target%]        Ok(table.definition()?)
    }
}
