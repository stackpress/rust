//! Host-only length guards retain private declaration arguments outside browser contracts.
use super::*;

/// Emit guards only for annotated models; null and unchanged patches retain their semantics.
pub(super) fn emit(model: &Model, text: &mut String) {
    let fields: Vec<_> = model
        .fields
        .iter()
        .filter(|field| field.storage_length.is_some())
        .collect();
    if fields.is_empty() {
        return;
    }
    text.push_str(&format!("impl {}Store {{\n/// Enforce host storage declarations, including redacted private fields.\nfn storage_bounds(value:&_Stored)->DataResult<()> {{\n", model.rust_name));
    for field in &fields {
        let length = field.storage_length.expect("filtered declaration");
        let check = format!(
            "if text.chars().count()>{length}usize {{return Err(DataError::new(\"declared storage length exceeded\"));}}\n"
        );
        if field.required {
            text.push_str(&format!("let text=&value.{};\n{check}", field.rust_name));
        } else {
            text.push_str(&format!(
                "if value.{}.as_ref().is_some_and(|text|text.chars().count()>{length}usize) {{return Err(DataError::new(\"declared storage length exceeded\"));}}\n",
                field.rust_name
            ));
        }
    }
    if !fields.iter().any(|field| !field.key && !field.generated) {
        text.push_str("Ok(())\n}\n}\n");
        return;
    }
    text.push_str("Ok(())\n}\n/// Check supplied encoded patch values before any write is submitted.\nfn patch_bounds(values:&[(String,stackpress_sql::SqlValue)])->DataResult<()> {\nfor (column,value) in values {match column.as_str() {\n");
    for field in fields.iter().filter(|field| !field.key && !field.generated) {
        let length = field.storage_length.expect("filtered declaration");
        text.push_str(&format!("{:?}=>{{if let stackpress_sql::SqlValue::Text(text)=value && text.chars().count()>{length}usize {{return Err(DataError::new(\"declared storage length exceeded\"));}}}},\n", field.column));
    }
    text.push_str("_=>{}\n}}\nOk(())\n}\n}\n");
}
