//! Any Excel sheet's rows as JSON (`kawari-questdump sheet`), and the list of sheets (`sheets`): for looking at the
//! client's data from the server's game install without writing a reader for each sheet (ArchonEngine
//! ControlPanel's harness serves them). Columns are named from `sheet_columns.json` (EXDSchema's names, through
//! icarus's sources; `sheet_columns.py` writes it); a column it doesn't name is `#<index>`.

use std::collections::HashMap;
use std::sync::OnceLock;

use physis::Language;
use physis::excel::{Field, Row};
use physis::resource::ResourceResolver;
use serde_json::{Value, json};

/// The most rows one call returns.
pub const MAX_ROWS: usize = 500;

fn column_names() -> &'static HashMap<String, Vec<String>> {
    static NAMES: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();
    NAMES.get_or_init(|| {
        serde_json::from_str(include_str!("../sheet_columns.json")).unwrap_or_default()
    })
}

/// A column's name: the schema's, else `#<index>`.
pub fn column_name(sheet: &str, index: usize) -> String {
    column_names()
        .get(sheet)
        .and_then(|names| names.get(index))
        .filter(|name| !name.is_empty())
        .cloned()
        .unwrap_or_else(|| format!("#{index}"))
}

fn field_value(field: &Field) -> Value {
    match field {
        Field::String(text) => json!(text.to_string()),
        Field::Bool(value) => json!(value),
        Field::Int8(value) => json!(value),
        Field::UInt8(value) => json!(value),
        Field::Int16(value) => json!(value),
        Field::UInt16(value) => json!(value),
        Field::Int32(value) => json!(value),
        Field::UInt32(value) => json!(value),
        Field::Float32(value) => json!(value),
        Field::Int64(value) => json!(value),
        Field::UInt64(value) => json!(value),
    }
}

fn row_value(sheet: &str, id: u32, subrow: Option<u16>, row: &Row) -> Value {
    let mut columns = serde_json::Map::new();
    for (index, field) in row.columns.iter().enumerate() {
        columns.insert(column_name(sheet, index), field_value(field));
    }
    let mut value = json!({ "id": id, "columns": columns });
    if let Some(subrow) = subrow {
        value["subrow"] = json!(subrow);
    }
    value
}

/// Which rows: IDs (`1043117`) and ranges (`100-120`), comma-separated.
pub fn parse_rows(spec: &str) -> Result<Vec<(u32, u32)>, String> {
    let mut ranges = Vec::new();
    for part in spec
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        let (low, high) = part.split_once('-').unwrap_or((part, part));
        let low: u32 = low
            .trim()
            .parse()
            .map_err(|_| format!("{part:?} isn't a row or a range"))?;
        let high: u32 = high
            .trim()
            .parse()
            .map_err(|_| format!("{part:?} isn't a row or a range"))?;
        if high < low {
            return Err(format!("{part:?} goes backwards"));
        }
        ranges.push((low, high));
    }
    Ok(ranges)
}

/// A `--where COLUMN=VALUE` filter: the column (by name or `#index`) whose value, as text, is VALUE.
pub fn parse_where(spec: &str) -> Result<(String, String), String> {
    let (column, value) = spec
        .split_once('=')
        .ok_or_else(|| format!("--where {spec:?} isn't COLUMN=VALUE"))?;
    Ok((column.trim().to_string(), value.trim().to_string()))
}

fn matches(sheet: &str, row: &Row, filter: &(String, String)) -> bool {
    row.columns.iter().enumerate().any(|(index, field)| {
        (column_name(sheet, index) == filter.0 || format!("#{index}") == filter.0)
            && match field_value(field) {
                Value::String(text) => text == filter.1,
                other => other.to_string() == filter.1,
            }
    })
}

/// The sheet's rows `rows` selects (all of them if none), those `filter` keeps, up to `MAX_ROWS`.
pub fn dump(
    resolver: &mut ResourceResolver,
    language: Language,
    sheet: &str,
    rows: &[(u32, u32)],
    filter: Option<&(String, String)>,
) -> Result<Value, String> {
    let exh = resolver
        .read_excel_sheet_header(sheet)
        .map_err(|err| format!("no sheet {sheet:?} ({err:?})"))?;
    // Sheets without text are in no language.
    let language = if exh.languages.contains(&language) {
        language
    } else {
        exh.languages.first().copied().unwrap_or(Language::None)
    };
    let data = resolver
        .read_excel_sheet(&exh, sheet, language)
        .map_err(|err| format!("couldn't read {sheet} ({err:?})"))?;
    let columns: Vec<Value> = exh
        .column_definitions
        .iter()
        .enumerate()
        .map(|(index, column)| {
            json!({ "index": index, "name": column_name(sheet, index), "type": format!("{:?}", column.data_type),
                    "offset": column.offset })
        })
        .collect();

    let mut out = Vec::new();
    let mut more = false;
    'pages: for page in &data.pages {
        for entry in &page.entries {
            if !rows.is_empty()
                && !rows
                    .iter()
                    .any(|(low, high)| (*low..=*high).contains(&entry.id))
            {
                continue;
            }
            let has_subrows =
                entry.subrows.len() > 1 || entry.subrows.first().is_some_and(|(id, _)| *id != 0);
            for (subrow, row) in &entry.subrows {
                if filter.is_some_and(|filter| !matches(sheet, row, filter)) {
                    continue;
                }
                if out.len() == MAX_ROWS {
                    more = true;
                    break 'pages;
                }
                out.push(row_value(
                    sheet,
                    entry.id,
                    has_subrows.then_some(*subrow),
                    row,
                ));
            }
        }
    }
    Ok(json!({
        "sheet": sheet,
        "language": format!("{language:?}"),
        "row_count": data.pages.iter().map(|page| page.entries.len()).sum::<usize>(),
        "columns": columns,
        "rows": out,
        "more": more,
    }))
}

/// Every sheet the game lists (its root.exl.)
pub fn list(resolver: &mut ResourceResolver) -> Result<Value, String> {
    let mut names = resolver
        .get_all_sheet_names()
        .map_err(|err| format!("couldn't list the sheets ({err:?})"))?;
    names.sort();
    Ok(json!({ "sheets": names }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_and_ranges() {
        assert_eq!(parse_rows("1043117").unwrap(), vec![(1043117, 1043117)]);
        assert_eq!(parse_rows("1, 5-7").unwrap(), vec![(1, 1), (5, 7)]);
        assert!(parse_rows("7-5").is_err());
        assert!(parse_rows("x").is_err());
        assert_eq!(
            parse_where("Invisibility=1").unwrap(),
            ("Invisibility".into(), "1".into())
        );
    }

    /// ENpcBase's columns are named as EXDSchema names them (what's needed to read Gaius's, 1043117.)
    #[test]
    fn columns_are_named() {
        assert_eq!(column_name("ENpcBase", 105), "Invisibility");
        assert_eq!(column_name("ENpcBase", 1), "Important");
        assert_eq!(column_name("ENpcBase", 2), "ENpcData[0]");
        assert_eq!(column_name("NoSuchSheet", 3), "#3");
    }
}
