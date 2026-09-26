use crate::errors::CargoResult;
use crate::manifest::InheritMode;
use crate::manifest::Item;
use crate::manifest::Manifest;
use crate::manifest::TablePath;

#[derive(Debug, Clone, Default)]
pub enum Modifier {
    AddKey(TablePath, String, Item, InheritMode),

    UpdateKey(TablePath, String, Item, InheritMode),

    RemoveKey(TablePath, String),

    AddItemsAt(TablePath, Option<String>, Item, InheritMode),

    UpdateItemsAt(TablePath, Option<String>, Item, InheritMode),

    RemoveItemsAt(TablePath, Option<String>, Option<Vec<String>>),

    AddValueToArray(TablePath, String, Item),

    UpdateValueOfArray(TablePath, String, Option<Item>, Item),

    RemoveValueFromArray(TablePath, String, Item),

    #[default]
    None,
}

impl Modifier {
    pub fn add_key(
        table_path: TablePath,
        key: impl Into<String>,
        value: Item,
        mode: impl Into<InheritMode>,
    ) -> Self {
        Self::AddKey(table_path, key.into(), value, mode.into())
    }

    pub fn update_key(
        table_path: TablePath,
        key: impl Into<String>,
        value: Item,
        mode: impl Into<InheritMode>,
    ) -> Self {
        Self::UpdateKey(table_path, key.into(), value, mode.into())
    }

    pub fn remove_key(table_path: TablePath, key: impl Into<String>) -> Self {
        Self::RemoveKey(table_path, key.into())
    }

    pub fn add_items_at(
        table_path: TablePath,
        key: Option<String>,
        value: Item,
        mode: impl Into<InheritMode>,
    ) -> Self {
        Self::AddItemsAt(table_path, key, value, mode.into())
    }

    pub fn update_items_at(
        table_path: TablePath,
        key: Option<String>,
        value: Item,
        mode: impl Into<InheritMode>,
    ) -> Self {
        Self::UpdateItemsAt(table_path, key, value, mode.into())
    }

    pub fn remove_items_at(
        table_path: TablePath,
        key: Option<String>,
        targets: Option<Vec<String>>,
    ) -> Self {
        Self::RemoveItemsAt(table_path, key, targets)
    }

    pub fn add_value_to_array(table_path: TablePath, key: impl Into<String>, value: Item) -> Self {
        Self::AddValueToArray(table_path, key.into(), value)
    }

    pub fn update_value_of_array(
        table_path: TablePath,
        key: impl Into<String>,
        old_value: Option<Item>,
        value: Item,
    ) -> Self {
        Self::UpdateValueOfArray(table_path, key.into(), old_value, value)
    }

    pub fn remove_value_from_array(
        table_path: TablePath,
        key: impl Into<String>,
        value: Item,
    ) -> Self {
        Self::RemoveValueFromArray(table_path, key.into(), value)
    }

    pub fn apply(&self, manifest: &mut Manifest) -> CargoResult<()> {
        match self {
            Self::AddKey(table_path, key, value, mode) => {
                manifest.add_key(table_path, key, value, mode)?;
            }

            Self::UpdateKey(table_path, key, value, mode) => {
                manifest.update_key(table_path, key, value, mode)?;
            }

            Self::RemoveKey(table_path, key) => {
                manifest.remove_key(table_path, key)?;
            }

            Self::AddItemsAt(table_path, key, value, mode) => {
                manifest.add_items_at(table_path, key.as_deref(), value, mode)?;
            }

            Self::UpdateItemsAt(table_path, key, value, mode) => {
                manifest.update_items_at(table_path, key.as_deref(), value, mode)?;
            }

            Self::RemoveItemsAt(table_path, key, targets) => {
                manifest.remove_items_at(table_path, key.as_deref(), targets.as_deref())?;
            }

            Self::AddValueToArray(table_path, key, value) => {
                manifest.add_value_to_array(table_path, key, value)?;
            }

            Self::UpdateValueOfArray(table_path, key, old_value, value) => {
                manifest.update_value_of_array(table_path, key, old_value.as_ref(), value)?;
            }

            Self::RemoveValueFromArray(table_path, key, value) => {
                manifest.remove_value_from_array(table_path, key, value)?;
            }

            Self::None => {}
        }

        Ok(())
    }
}
