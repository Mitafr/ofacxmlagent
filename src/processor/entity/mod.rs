use sea_orm::{DbErr, QueryResult, TryGetable};

pub mod name;
pub mod sdn;

/// Extract a QueryResult field as Vec
///
/// * `query_result` - QueryResult of a successful SQL query.
/// * `alias` - alias of QueryReqult where data is stored.
/// * `record_field` - mutable record where data will be append
pub fn extract_query_field_as_vec<T>(query_result: &QueryResult, alias: &str, record_field: &mut Vec<T>) -> Result<(), DbErr>
where
    T: TryGetable + PartialEq,
{
    if let Some(result) = query_result.try_get::<Option<T>>("", alias)? {
        if !record_field.contains(&result) {
            record_field.push(result);
        }
    }
    Ok(())
}

/// Extract a QueryResult field
///
/// * `query_result` - QueryResult of a successful SQL query.
/// * `alias` - alias of QueryReqult where data is stored.
/// * `record_field` - mutable record where data will be append
pub fn extract_query_field<T>(query_result: &QueryResult, alias: &str, record_field: &mut T) -> Result<(), DbErr>
where
    T: TryGetable + PartialEq,
{
    if let Some(result) = query_result.try_get::<Option<T>>("", alias)? {
        *record_field = result;
    }
    Ok(())
}

/// Extract a QueryResult field as Vec
///
/// * `query_result` - QueryResult of a successful SQL query.
/// * `alias` - alias of QueryReqult where data is stored.
/// * `record_field` - mutable record where data will be append
pub fn extract_field_as_vec<T>(field: Option<T>, record_field: &mut Vec<T>) -> Result<(), DbErr>
where
    T: TryGetable + PartialEq,
{
    if let Some(result) = field {
        if !record_field.contains(&result) {
            record_field.push(result);
        }
    }
    Ok(())
}

/// Extract a QueryResult field
///
/// * `query_result` - QueryResult of a successful SQL query.
/// * `alias` - alias of QueryReqult where data is stored.
/// * `record_field` - mutable record where data will be append
pub fn extract_field<T>(field: Option<T>, record_field: &mut T) -> Result<(), DbErr>
where
    T: TryGetable + PartialEq,
{
    if let Some(result) = field {
        *record_field = result;
    }
    Ok(())
}
