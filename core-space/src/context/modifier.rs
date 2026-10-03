use crate::errors::CargoResult;
use crate::manifest::InheritMode;
use crate::manifest::Item;
use crate::manifest::Manifest;
use crate::manifest::TablePath;
use crate::manifest::Value;

#[derive(Debug, Clone, Default)]
pub enum Modifier {
    AddKey(TablePath, String, Item, InheritMode),

    UpdateKey(TablePath, String, Item, InheritMode),

    RemoveKey(TablePath, String),

    AddItemsAt(TablePath, Item, InheritMode),

    UpdateItemsAt(TablePath, Item, InheritMode),

    RemoveItemsAt(TablePath, Option<Vec<String>>),

    AddValueToArray(TablePath, String, Value),

    UpdateValueOfArray(TablePath, String, Option<Value>, Value),

    RemoveValueFromArray(TablePath, String, Value),

    #[default]
    None,
}

impl Modifier {
    pub fn add_key(
        table_path: TablePath,
        key: impl Into<String>,
        item: Item,
        mode: impl Into<InheritMode>,
    ) -> Self {
        Self::AddKey(table_path, key.into(), item, mode.into())
    }

    pub fn update_key(
        table_path: TablePath,
        key: impl Into<String>,
        item: Item,
        mode: impl Into<InheritMode>,
    ) -> Self {
        Self::UpdateKey(table_path, key.into(), item, mode.into())
    }

    pub fn remove_key(table_path: TablePath, key: impl Into<String>) -> Self {
        Self::RemoveKey(table_path, key.into())
    }

    pub fn add_items_at(
        table_path: TablePath,
        items_table: Item,
        mode: impl Into<InheritMode>,
    ) -> Self {
        Self::AddItemsAt(table_path, items_table, mode.into())
    }

    pub fn update_items_at(
        table_path: TablePath,
        items_table: Item,
        mode: impl Into<InheritMode>,
    ) -> Self {
        Self::UpdateItemsAt(table_path, items_table, mode.into())
    }

    pub fn remove_items_at(table_path: TablePath, targets: Option<Vec<String>>) -> Self {
        Self::RemoveItemsAt(table_path, targets)
    }

    pub fn add_value_to_array(table_path: TablePath, key: impl Into<String>, value: Value) -> Self {
        Self::AddValueToArray(table_path, key.into(), value)
    }

    pub fn update_value_of_array(
        table_path: TablePath,
        key: impl Into<String>,
        old_value: Option<Value>,
        new_value: Value,
    ) -> Self {
        Self::UpdateValueOfArray(table_path, key.into(), old_value, new_value)
    }

    pub fn remove_value_from_array(
        table_path: TablePath,
        key: impl Into<String>,
        target_value: Value,
    ) -> Self {
        Self::RemoveValueFromArray(table_path, key.into(), target_value)
    }

    pub fn apply(&self, manifest: &mut Manifest) -> CargoResult<()> {
        match self {
            Self::AddKey(table_path, key, item, mode) => {
                manifest.add_key(table_path, key, item, mode)?;
            }

            Self::UpdateKey(table_path, key, item, mode) => {
                manifest.update_key(table_path, key, item, mode)?;
            }

            Self::RemoveKey(table_path, key) => {
                manifest.remove_key(table_path, key)?;
            }

            Self::AddItemsAt(table_path, items_table, mode) => {
                manifest.add_items_at(table_path, items_table, mode)?;
            }

            Self::UpdateItemsAt(table_path, items_table, mode) => {
                manifest.update_items_at(table_path, items_table, mode)?;
            }

            Self::RemoveItemsAt(table_path, targets) => {
                manifest.remove_items_at(table_path, targets.as_deref())?;
            }

            Self::AddValueToArray(table_path, key, value) => {
                manifest.add_value_to_array(table_path, key, value)?;
            }

            Self::UpdateValueOfArray(table_path, key, old_value, new_value) => {
                manifest.update_value_of_array(table_path, key, old_value.as_ref(), new_value)?;
            }

            Self::RemoveValueFromArray(table_path, key, target_value) => {
                manifest.remove_value_from_array(table_path, key, target_value)?;
            }

            Self::None => {}
        }

        Ok(())
    }
}
