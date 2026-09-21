use crate::errors::{CargoResult, error};
use crate::manifest::InheritMode;
use crate::manifest::Item;
use crate::manifest::Manifest;
use crate::manifest::TablePath;

#[derive(Debug, Clone, Default)]
pub enum Modifier {
    AddKey(Change, InheritMode),

    UpdateKey(Change, InheritMode),

    RemoveKeyOrValue(Change),

    ReplaceItemsAt(Change, InheritMode),

    InsertItemsAt(Change, InheritMode),

    #[default]
    None,
}

impl Modifier {
    pub fn add_key(change: Change, mode: impl Into<InheritMode>) -> Self {
        Self::AddKey(change, mode.into())
    }

    pub fn update_key(change: Change, mode: impl Into<InheritMode>) -> Self {
        Self::UpdateKey(change, mode.into())
    }

    pub fn remove_key_or_value(change: Change) -> Self {
        Self::RemoveKeyOrValue(change)
    }

    pub fn replace_items_at(change: Change, mode: impl Into<InheritMode>) -> Self {
        Self::ReplaceItemsAt(change, mode.into())
    }

    pub fn insert_items_at(change: Change, mode: impl Into<InheritMode>) -> Self {
        Self::InsertItemsAt(change, mode.into())
    }

    pub fn apply(&self, manifest: &mut Manifest) -> CargoResult<()> {
        match self {
            Self::AddKey(change, mode) => {
                let key = change
                    .key()
                    .ok_or_else(|| error!("Add key requires a key"))?;

                let item = change
                    .item()
                    .ok_or_else(|| error!("Add key requires an item"))?;

                let item_resolve = mode.resolve_key(key.to_string(), item.clone())?;

                manifest.add_key(change.path(), key, &item_resolve)?;
            }

            Self::UpdateKey(change, mode) => {
                let key = change
                    .key()
                    .ok_or_else(|| error!("Update key requires a key"))?;

                let item = change
                    .item()
                    .ok_or_else(|| error!("Update key requires an item"))?;

                let item_resolve = if manifest.is_table_inline_table(change.path()) {
                    item.clone()
                } else {
                    mode.resolve_key(key.to_string(), item.clone())?
                };

                manifest.update_key(change.path(), key, &item_resolve)?;
            }

            Self::RemoveKeyOrValue(change) => {
                let key = change
                    .key()
                    .ok_or_else(|| error!("Remove key or value requires a key"))?;

                if let Some(message) =
                    manifest.remove_key_or_value(change.path(), key, change.item())?
                {
                    println!("{message}");
                }
            }

            Self::ReplaceItemsAt(change, mode) => {
                let item = change
                    .item()
                    .ok_or_else(|| error!("Replace items requires an item"))?;

                let item_resolve = if manifest.is_table_inline_table(change.path()) {
                    item.clone()
                } else if let Some(key) = change.key() {
                    mode.resolve_key(key.to_string(), item.clone())?
                } else {
                    mode.resolve_items(item.clone())?
                };

                manifest.replace_items_at(change.path(), change.key(), &item_resolve)?;
            }

            Self::InsertItemsAt(change, mode) => {
                let item = change
                    .item()
                    .ok_or_else(|| error!("Insert items requires an item"))?;

                let item_resolve = if manifest.is_table_inline_table(change.path()) {
                    item.clone()
                } else if let Some(key) = change.key() {
                    mode.resolve_key(key.to_string(), item.clone())?
                } else {
                    mode.resolve_items(item.clone())?
                };

                manifest.insert_items_at(change.path(), change.key(), &item_resolve)?;
            }

            Self::None => {}
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Change {
    path: TablePath,
    key: Option<String>,
    item: Option<Item>,
}

impl Change {
    pub fn new() -> Self {
        Self {
            path: TablePath::default(),
            key: None,
            item: None,
        }
    }

    pub fn with_path(mut self, path: TablePath) -> Self {
        self.path = path;
        self
    }

    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn with_item(mut self, item: Item) -> Self {
        self.item = Some(item);
        self
    }

    pub fn path(&self) -> &TablePath {
        &self.path
    }

    pub fn key(&self) -> Option<&str> {
        self.key.as_deref()
    }

    pub fn item(&self) -> Option<&Item> {
        self.item.as_ref()
    }

    pub fn into_parts(self) -> (TablePath, Option<String>, Option<Item>) {
        (self.path, self.key, self.item)
    }
}
