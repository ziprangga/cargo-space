mod inherit_mode;
mod table_path;
mod toml_manifest;

pub use inherit_mode::InheritMode;
pub use table_path::TablePath;
pub use toml_manifest::Array;
pub use toml_manifest::Doc;
pub use toml_manifest::DocMut;
pub use toml_manifest::InlineTable;
pub use toml_manifest::Item;
pub use toml_manifest::Table;
use toml_manifest::TomlManifest;
pub use toml_manifest::Value;

use crate::errors::{CargoResult, Context, error};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct Manifest {
    toml_path: PathBuf,
    toml_manifest: TomlManifest,
}

impl Manifest {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            toml_path: path.into(),
            toml_manifest: TomlManifest::new(),
        }
    }

    pub fn from_toml_path(path: &Path) -> CargoResult<Self> {
        let toml_path = path.to_path_buf();
        let toml_manifest = TomlManifest::from_manifest_path(&path)?;

        Ok(Self {
            toml_path,
            toml_manifest,
        })
    }

    /// Returns an immutable reference to the underlying TOML document.
    pub fn data(&self) -> &DocMut {
        &self.toml_manifest.data()
    }

    /// Returns a mutable reference to the underlying TOML document.
    pub fn data_mut(&mut self) -> &mut DocMut {
        self.toml_manifest.data_mut()
    }

    pub fn write(&self) -> CargoResult<()> {
        let contents = self.data().to_string();

        std::fs::write(&self.toml_path, contents.as_bytes())
            .context("Failed to write updated Cargo.toml")
    }
}

impl Manifest {
    pub fn get_item_of_key(&self, table: &TablePath, key: &str) -> CargoResult<&Item> {
        let item = self.toml_manifest.get_table_like(table.as_slice())?;

        item.get(key)
            .ok_or_else(|| error!("`{key}` could not be found"))
    }

    pub fn get_item_of_table(&self, table: &TablePath) -> CargoResult<&Item> {
        self.toml_manifest
            .get_table_like(table.as_slice())
            .with_context(|| format!("Item of `{}` could not be found", table))
    }

    pub fn add_key(&mut self, table: &TablePath, key: &str, value: &Item) -> CargoResult<()> {
        let table = self
            .toml_manifest
            .get_or_create_table_like_mut(table.as_slice())?;

        table[key] = value.into();

        Ok(())
    }

    pub fn replace_value_of_key(
        &mut self,
        table: &TablePath,
        key: &str,
        value: &Item,
    ) -> CargoResult<()> {
        let item = self
            .toml_manifest
            .get_table_like_mut(table.as_slice())?
            .get_mut(key)
            .ok_or_else(|| error!("`{key}` could not be found"))?;

        *item = value.into();

        Ok(())
    }

    pub fn update_value_of_key(
        &mut self,
        table: &TablePath,
        key: &str,
        value: &Item,
    ) -> CargoResult<()> {
        let item = self
            .toml_manifest
            .get_table_like_mut(table.as_slice())?
            .get_mut(key)
            .ok_or_else(|| error!("`{key}` could not be found"))?;

        if let Some(old_value) = item.as_value_mut() {
            if let Some(array) = old_value.as_array_mut() {
                if !array.iter().any(|item| item.as_str() == value.as_str()) {
                    if let Some(value) = value.as_value() {
                        array.push(value);
                    }
                }
            } else if old_value.as_str() != value.as_str() {
                if let Some(value) = value.as_value() {
                    *old_value = value.clone();
                }
            }
        }

        Ok(())
    }

    pub fn delete_key_from_table(
        &mut self,
        table: &TablePath,
        key: &str,
    ) -> CargoResult<Option<String>> {
        let table = self
            .toml_manifest
            .get_table_like_mut(table.as_slice())?
            .as_table_like_mut()
            .ok_or_else(|| error!("Item is not a table"))?;

        let message = if table.remove(key).is_none() {
            Some(format!("`{key}` could not be found"))
        } else {
            None
        };

        Ok(message)
    }

    pub fn insert_items_in_table(&mut self, table: &TablePath, item: &Item) -> CargoResult<()> {
        let source = item
            .as_table_like()
            .ok_or_else(|| error!("Item is not a table"))?;

        let table = self
            .toml_manifest
            .get_or_create_table_like_mut(table.as_slice())?;

        for (key, value) in source.iter() {
            table[key] = value.clone();
        }

        Ok(())
    }

    pub fn remove_value_from_array(
        &mut self,
        table: &TablePath,
        key: &str,
        value: &str,
    ) -> CargoResult<()> {
        let item = self
            .toml_manifest
            .get_table_like_mut(table.as_slice())?
            .get_mut(key)
            .ok_or_else(|| error!("`{key}` could not be found"))?;

        let array = item
            .as_array_mut()
            .ok_or_else(|| error!("`{key}` is not an array"))?;

        let index = array.iter().position(|item| item.as_str() == Some(value));

        if let Some(index) = index {
            array.remove(index);
        }

        Ok(())
    }
}

impl Manifest {
    pub fn is_table_exist(&self, table: &TablePath) -> bool {
        self.toml_manifest.get_table_like(table.as_slice()).is_ok()
    }

    pub fn is_key_exist(&self, table: &TablePath, key: &str) -> bool {
        self.toml_manifest
            .get_table_like(table.as_slice())
            .ok()
            .and_then(|table| table.get(key))
            .is_some()
    }

    pub fn is_key_inline_table(&self, table: &TablePath, key: &str) -> bool {
        self.toml_manifest
            .get_table_like(table.as_slice())
            .ok()
            .and_then(|table| table.get(key))
            .is_some_and(Item::is_inline_table)
    }
}
