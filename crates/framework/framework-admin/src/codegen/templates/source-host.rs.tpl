//! Generated source host catalog; the application selects concrete route/event registrars.
/// Native callbacks traverse this catalog once in source order, with admin registered last.
pub const MODELS:&[(&str,&str)]=&[
[%#@:models model%]    ([%model.module_literal%],[%model.name_literal%]),
[%/@:models%]];

/// Only source-authored labels/icons are published; no native model or codec metadata.
pub fn presentation()->stackpress_framework_schema::__private::serde_json::Value {
    stackpress_framework_schema::__private::serde_json::json!({
[%#@:models model%]        [%model.module_literal%]:{"label":[%model.label_literal%],"plural":[%model.plural_literal%],"icon":[%model.icon_literal%]},
[%/@:models%]    })
}
