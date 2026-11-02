[%#@:depths depth%][%#@:depth.views view%]/// Finite depth-[%depth.value%] [%view.audience%] include result; omission means Unloaded.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct [%view.prefix%]View[%depth.value%] {
    /// Fully owned [%view.row_description%] row.
    pub row: [%#?:view.public%]crate::generated::contracts::public::Public[%model%][%|%]_Stored[%/?:view.public%],
[%#@:view.fields field%]    /// Explicit eager [%field.name_literal%] relation.
    #[serde(rename = [%field.relation_literal%], default)]
    pub [%field.member%]: Loaded<
[%#?:field.multiple%]        Vec<
[%|%][%#?:field.optional%]        Option<
[%/?:field.optional%][%/?:field.multiple%]        super::[%field.store%]::[%view.prefix%]View[%depth.previous%]
[%#?:field.multiple%]        >
[%|%][%#?:field.optional%]        >
[%/?:field.optional%][%/?:field.multiple%]    >,
[%/@:view.fields%]}
[%/@:depth.views%]impl View[%depth.value%] {
    /// Recursively project each loaded node, retaining unloaded/empty/absent distinctions.
    pub fn project(self) -> DataResult<PublicView[%depth.value%]> {
        Ok(PublicView[%depth.value%] {
            row: self.row.project().map_err(stackpress_framework_sql::stores::projection_error)?,
[%#@:depth.projection_fields field%]            [%field.member%]: self.[%field.member%].try_map(|values|
[%#?:field.multiple%]                values.into_iter().map(|value| value.project()).collect::<DataResult<Vec<_>>>()
[%|%][%#?:field.optional%]                values.map(|value| value.project()).transpose()
[%|%]                values.project()
[%/?:field.optional%][%/?:field.multiple%]            )?,
[%/@:depth.projection_fields%]        })
    }
}
[%/@:depths%]