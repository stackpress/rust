//! Pure scalar predicates, literal containment and deterministic temporal/null ordering.
use super::{Exposure, QueryField, ReadQuery, StoreSpec, Visibility, invalid, validation};
use crate::{
    DataError, DataErrorKind, DataResult, StorageCodec, StorageColumn, codec_error, encode_value,
};
use serde_json::Value;
use stackpress_framework_schema::{NullOrder, Predicate, SortDirection, query_budget};
use stackpress_sql::{
    Compare, DialectId, Expr, JsonPath, JsonScalar, Order, Render, Select, Selector, Sort,
    SqlValue, TrustedSql,
};
use std::collections::BTreeSet;

/// Normalize a checked canonical UTC value to a fixed-width comparison key, without changing storage.
fn datetime_key(value: &str) -> DataResult<String> {
    let utc = value.strip_suffix('Z').ok_or_else(invalid)?;
    let (base, fraction) = utc.split_once('.').unwrap_or((utc, ""));
    Ok(format!("{base}.{fraction:0<9}Z"))
}
/// Use a narrowly audited constant expression over one checked identifier for exact nanosecond order.
/// Canonical UTC storage remains compatible with existing values and foreign keys. Every digit
/// is preserved; SQLite's millisecond strftime conversion is deliberately not used.
pub(super) fn ordered(field: &QueryField, dialect: DialectId) -> DataResult<Expr> {
    // Canonical UTC fractions need padding for lexical order; other scalar codecs sort natively.
    if !matches!(field.scalar, StorageCodec::Datetime) {
        return Ok(Expr::column(field.storage.column));
    }
    if dialect != DialectId::Sqlite {
        return Err(
            DataError::new("datetime ordering awaits this dialect's execution proof")
                .with_kind(DataErrorKind::Unsupported),
        );
    }
    // Check the catalog identifier with the SQL core before placing it in the audited expression.
    Select::new([field.storage.column])
        .from("checked_identifier")
        .render(dialect)?;
    let column = format!("\"{}\"", field.storage.column.replace('"', "\"\""));
    let sql = format!(
        "CASE WHEN instr({column}, '.') = 0 THEN substr({column}, 1, length({column}) - 1) || '.000000000Z' ELSE substr({column}, 1, instr({column}, '.')) || substr(substr({column}, instr({column}, '.') + 1, length({column}) - instr({column}, '.') - 1) || '000000000', 1, 9) || 'Z' END"
    );
    Ok(Expr::Trusted(dialect, TrustedSql::audited(sql), Vec::new()))
}
/// Compile scalar null/comparison/pattern or JSON membership with exact codec checks.
fn predicate(
    field: &QueryField,
    predicate: &Predicate<Value>,
    dialect: DialectId,
) -> DataResult<Expr> {
    let column = field.storage.column;
    // NULL tests are explicit and valid only for nullable columns, including nullable lists.
    if let Predicate::IsNull(is_null) = predicate {
        if field.storage.required {
            return Err(invalid());
        }
        return Ok(Expr::IsNull(Box::new(Expr::column(column)), !*is_null));
    }
    // Operator/type eligibility precedes encoding so a malformed shape never reaches native SQL.
    let (value, comparison) = match predicate {
        Predicate::Eq(value) if !field.multiple => (value, Some(Compare::Eq)),
        Predicate::Ne(value) if !field.multiple => (value, Some(Compare::Ne)),
        Predicate::Ge(value)
            if !field.multiple
                && !matches!(field.scalar, StorageCodec::Boolean | StorageCodec::Json) =>
        {
            (value, Some(Compare::Ge))
        }
        Predicate::Le(value)
            if !field.multiple
                && !matches!(field.scalar, StorageCodec::Boolean | StorageCodec::Json) =>
        {
            (value, Some(Compare::Le))
        }
        Predicate::Like(value)
            if !field.multiple
                && field.choices == "[]"
                && matches!(field.scalar, StorageCodec::Text) =>
        {
            (value, None)
        }
        Predicate::Has(value) | Predicate::Hasnt(value) if field.multiple => (value, None),
        _ => return Err(invalid()),
    };
    if value.is_null() || value.as_str().is_some_and(|value| value.len() > 4096) {
        return Err(invalid());
    }
    // Enum values must belong to the actual declared scalar choices, even on trusted APIs.
    let choices: Vec<Value> = serde_json::from_str(field.choices).map_err(codec_error)?;
    if !choices.is_empty() && !choices.contains(value) {
        return Err(invalid());
    }
    // Lists use their element codec here; storage for the whole list remains JSON.
    let mut scalar = encode_value(
        value,
        &StorageColumn {
            codec: field.scalar,
            required: true,
            nonempty: false,
            ..field.storage
        },
    )?;
    // Comparison binds use the same fixed precision as the checked column expression.
    if let Some(comparison) = comparison {
        if matches!(field.scalar, StorageCodec::Datetime) {
            scalar = SqlValue::Text(datetime_key(value.as_str().ok_or_else(invalid)?)?);
        }
        return Ok(ordered(field, dialect)?.compare(comparison, Expr::value(scalar)));
    }
    if let Predicate::Like(_) = predicate {
        return Ok(Expr::Like(
            Box::new(Expr::column(column)),
            value.as_str().ok_or_else(invalid)?.into(),
            '\\',
        ));
    }
    // JSON membership accepts scalar values only and keeps temporal spelling canonical.
    let scalar = match scalar {
        SqlValue::Text(encoded) => JsonScalar::String(
            if matches!(
                field.scalar,
                StorageCodec::Date | StorageCodec::Time | StorageCodec::Datetime
            ) {
                match field.scalar {
                    StorageCodec::Date => stackpress_framework_schema::DateValue::parse(&encoded)
                        .map(|value| value.as_str().to_owned()),
                    StorageCodec::Time => stackpress_framework_schema::TimeValue::parse(&encoded)
                        .map(|value| value.as_str().to_owned()),
                    _ => stackpress_framework_schema::DatetimeValue::parse(&encoded)
                        .map(|value| value.as_str().to_owned()),
                }
                .map_err(validation)?
            } else {
                encoded
            },
        ),
        SqlValue::Integer(value) => JsonScalar::Integer(value),
        SqlValue::Real(value) => JsonScalar::Real(value),
        SqlValue::Bool(value) => JsonScalar::Bool(value),
        _ => return Err(invalid()),
    };
    let contains = Expr::json_contains(column, JsonPath::default(), scalar);
    Ok(if matches!(predicate, Predicate::Hasnt(_)) {
        Expr::Not(Box::new(contains))
    } else {
        contains
    })
}
/// Compile filters before ordering, keeping literal containment distinct from LIKE semantics.
pub(super) fn filtered(
    spec: &StoreSpec,
    query: &ReadQuery,
    dialect: DialectId,
    exposure: Exposure,
    visibility: Visibility,
    count: bool,
) -> DataResult<Select> {
    query_budget(
        query.q.as_deref(),
        query.filters.len(),
        query.sort.len(),
        &query.includes,
        query.skip,
        query.take,
    )
    .map_err(validation)?;
    // A count shares predicates/active policy while omitting sorting, paging and relations.
    let base = if count {
        Select::new([Selector::expression(Expr::Trusted(
            dialect,
            TrustedSql::audited("COUNT(*)"),
            Vec::new(),
        ))
        .alias("total")])
        .from(spec.table)
    } else {
        spec.base()
    };
    let mut select = spec.visible(base, visibility)?;
    if let Some(q) = query.q.as_ref().filter(|q| !q.is_empty()) {
        // The initial execution proof is SQLite. Network q waits for its dialect integration.
        if dialect != DialectId::Sqlite {
            return Err(
                DataError::new("literal containment is not verified for this dialect")
                    .with_kind(DataErrorKind::Unsupported),
            );
        }
        let mut expressions = Vec::new();
        for field in spec.fields.iter().filter(|field| {
            field.searchable && (exposure == Exposure::Trusted || field.public_search)
        }) {
            // Validate/quote the generated identifier using the core renderer before this
            // narrowly audited constant-function expression. q remains a bind, including %/_/quotes.
            Select::new([field.storage.column])
                .from(spec.table)
                .render(dialect)?;
            let quoted = format!("\"{}\"", field.storage.column.replace('"', "\"\""));
            expressions.push(Expr::Trusted(
                dialect,
                TrustedSql::audited(format!("instr({quoted}, ?) > 0")),
                vec![q.clone().into()],
            ));
        }
        if expressions.is_empty() {
            return Err(invalid());
        }
        select = select.and_where(Expr::any(expressions));
    }
    // Public eligibility is checked at the immutable field catalog, never from a UI role alone.
    for filter in &query.filters {
        let field = spec
            .fields
            .iter()
            .find(|field| {
                field.storage.wire == filter.field
                    && (exposure == Exposure::Trusted || field.public_filter)
            })
            .ok_or_else(invalid)?;
        select = select.and_where(predicate(field, &filter.predicate, dialect)?);
    }
    Ok(select)
}
/// Build a pure deterministic base page; every requested sort has explicit NULL ordering.
pub fn select(
    spec: &StoreSpec,
    query: &ReadQuery,
    dialect: DialectId,
    exposure: Exposure,
    visibility: Visibility,
) -> DataResult<Select> {
    let mut select = filtered(spec, query, dialect, exposure, visibility, false)?;
    let mut sorted = BTreeSet::new();
    for sort in &query.sort {
        let field = spec
            .fields
            .iter()
            .find(|field| {
                field.storage.wire == sort.field
                    && field.sortable
                    && (exposure == Exposure::Trusted || field.public_sort)
            })
            .ok_or_else(invalid)?;
        if !sorted.insert(field.storage.wire) {
            return Err(invalid());
        }
        // Sorting IS NULL first establishes portable null order without interpolated SQL.
        select = select.sort(Sort {
            expression: Expr::IsNull(Box::new(Expr::column(field.storage.column)), false),
            order: if sort.nulls == NullOrder::First {
                Order::Desc
            } else {
                Order::Asc
            },
        });
        select = select.sort(Sort {
            expression: ordered(field, dialect)?,
            order: if sort.direction == SortDirection::Asc {
                Order::Asc
            } else {
                Order::Desc
            },
        });
    }
    // A complete identity tie breaker makes equal sort values deterministic across pages.
    for key in spec.key.iter().filter(|key| !sorted.contains(**key)) {
        let field = spec
            .fields
            .iter()
            .find(|field| field.storage.wire == *key)
            .ok_or_else(invalid)?;
        select = select.sort(Sort {
            expression: ordered(field, dialect)?,
            order: Order::Asc,
        });
    }
    Ok(select.limit(u64::from(query.take)).offset(query.skip))
}
