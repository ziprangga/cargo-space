use crate::errors::{CargoResult, error};
use crate::manifest::InheritMode;
use crate::manifest::Item;
use crate::manifest::Manifest;
use crate::manifest::TablePath;

#[derive(Debug, Clone, Default)]
pub enum Modifier {
    AddKey(Change, InheritMode),
    UpdateValue(Change),
    DeleteKey(Change),
    RemoveValueOfArray(Change),
    ReplaceValue(Change, InheritMode),
    InsertItems(Change, InheritMode),

    #[default]
    None,
}

impl Modifier {
    pub fn add_key(change: Change, mode: impl Into<InheritMode>) -> Self {
        Self::AddKey(change, mode.into())
    }

    pub fn update_value(change: Change) -> Self {
        Self::UpdateValue(change)
    }

    pub fn delete_key(change: Change) -> Self {
        Self::DeleteKey(change)
    }

    pub fn remove_value_from_array(change: Change) -> Self {
        Self::RemoveValueOfArray(change)
    }

    pub fn replace_value(change: Change, mode: impl Into<InheritMode>) -> Self {
        Self::ReplaceValue(change, mode.into())
    }

    pub fn insert_items(change: Change, mode: impl Into<InheritMode>) -> Self {
        Self::InsertItems(change, mode.into())
    }
    pub fn apply(&self, manifest: &mut Manifest) -> CargoResult<()> {
        match self {
            Self::AddKey(change, mode) => {
                let item = mode.resolve(change.key().to_string(), change.item().clone())?;

                manifest.add_key(change.path(), change.key(), &item)?;
            }
            Self::UpdateValue(change) => {
                manifest.update_value_of_key(change.path(), change.key(), change.item())?;
            }
            Self::DeleteKey(change) => {
                if let Some(message) =
                    manifest.delete_key_from_table(change.path(), change.key())?
                {
                    println!("{message}");
                }
            }
            Self::RemoveValueOfArray(change) => {
                let value = change
                    .item()
                    .as_str()
                    .ok_or_else(|| error!("Remove value must be a string"))?;

                manifest.remove_value_from_array(change.path(), change.key(), value)?;
            }
            Self::ReplaceValue(change, mode) => {
                let item = mode.resolve(change.key().to_string(), change.item().clone())?;

                manifest.replace_value_of_key(change.path(), change.key(), &item)?;
            }
            Self::InsertItems(change, mode) => {
                let item = mode.resolve_insert(change.item().clone())?;

                manifest.insert_items_in_table(change.path(), &item)?;
            }

            Self::None => {}
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Change {
    path: TablePath,
    key: String,
    item: Item,
}

impl Change {
    pub fn new() -> Self {
        Self {
            path: TablePath::default(),
            key: String::new(),
            item: Item::default(),
        }
    }

    pub fn with_path(mut self, path: TablePath) -> Self {
        self.path = path;
        self
    }

    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = key.into();
        self
    }

    pub fn with_item(mut self, item: Item) -> Self {
        self.item = item;
        self
    }

    pub fn path(&self) -> &TablePath {
        &self.path
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn item(&self) -> &Item {
        &self.item
    }

    pub fn into_parts(self) -> (TablePath, String, Item) {
        (self.path, self.key, self.item)
    }
}
