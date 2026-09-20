use super::InlineTable;
use super::Item;
use crate::errors::CargoResult;
use crate::errors::error;

pub fn to_inline_table(item: &Item) -> CargoResult<InlineTable> {
    if let Some(table) = item.as_inline_table() {
        return Ok(table.clone());
    }

    if let Some(table) = item.as_table() {
        let mut inline_table = InlineTable::new();

        for (key, value) in table.iter() {
            if let Some(value) = value.as_value() {
                inline_table.insert(key, value.clone());
            } else {
                return Err(error!("table contains non-value item"));
            }
        }

        return Ok(inline_table);
    }

    Err(error!("item is not a table or inline table"))
}

pub fn to_inline_table_item(item: &Item) -> CargoResult<Item> {
    Ok(to_inline_table(item)?.into())
}
