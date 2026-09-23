use super::InlineTable;
use super::Item;
use super::Value;
use crate::errors::CargoResult;
use crate::errors::error;

pub fn into_inline_table(item: Item) -> CargoResult<InlineTable> {
    match item {
        Item::Value(Value::InlineTable(table)) => Ok(table),
        Item::Table(table) => Ok(table.into_inline_table()),
        _ => Err(error!("item is not a table or inline table")),
    }
}

pub fn into_inline_table_item(item: Item) -> CargoResult<Item> {
    Ok(into_inline_table(item)?.into())
}

pub fn into_inline_table_item_if(condition: bool, item: Item) -> CargoResult<Item> {
    if condition {
        Ok(into_inline_table(item)?.into())
    } else {
        Ok(item)
    }
}

pub fn into_item(value: impl Into<String>) -> Item {
    Item::Value(Value::from(value.into()))
}
