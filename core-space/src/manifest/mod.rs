mod inherit_mode;
mod man_util;
mod table_path;
mod toml_manifest;

use toml_manifest::TomlManifest;

pub use inherit_mode::InheritMode;
pub use table_path::TableDepKind;
pub use table_path::TableDepTarget;
pub use table_path::TablePath;
pub use table_path::build_table_dep;
pub use toml_manifest::Array;
pub use toml_manifest::Doc;
pub use toml_manifest::DocMut;
pub use toml_manifest::InlineTable;
pub use toml_manifest::Item;
pub use toml_manifest::Table;
pub use toml_manifest::Value;

pub use man_util::into_inline_table;
pub use man_util::into_inline_table_item;
pub use man_util::into_inline_table_item_if;
pub use man_util::into_item;

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

    pub fn toml_path(&self) -> &Path {
        &self.toml_path
    }

    /// Returns an immutable reference to the underlying TOML document.
    pub fn data(&self) -> &DocMut {
        &self.toml_manifest.data()
    }

    /// Returns a mutable reference to the underlying TOML document.
    pub fn data_mut(&mut self) -> &mut DocMut {
        self.toml_manifest.data_mut()
    }

    pub fn is_empty(&self) -> bool {
        self.toml_manifest.is_empty()
    }

    pub fn write(&self) -> CargoResult<()> {
        let contents = self.data().to_string();

        std::fs::write(&self.toml_path, contents.as_bytes())
            .context("Failed to write updated Cargo.toml")
    }
}

impl Manifest {
    pub fn is_table_exist(&self, table: &TablePath) -> bool {
        self.toml_manifest.get_table_like(table.as_slice()).is_ok()
    }

    pub fn is_table_inline_table(&self, table: &TablePath) -> bool {
        self.toml_manifest
            .get_table_like(table.as_slice())
            .ok()
            .is_some_and(|table| table.is_inline_table())
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

impl Manifest {
    pub fn get_array(&self, table: &TablePath, key: &str) -> CargoResult<&Array> {
        let item = self.toml_manifest.get_table_like(table.as_slice())?;

        item.get(key)
            .and_then(Item::as_array)
            .ok_or_else(|| error!("`{key}` is not an array"))
    }

    pub fn add_value_to_array(
        &mut self,
        table: &TablePath,
        key: &str,
        value: &Item,
    ) -> CargoResult<()> {
        let table = self
            .toml_manifest
            .get_or_create_table_like_mut(table.as_slice())?;

        let value = value
            .as_value()
            .ok_or_else(|| error!("Value is not a value"))?;

        if let Some(item) = table.get_mut(key) {
            let array = item
                .as_array_mut()
                .ok_or_else(|| error!("`{key}` is not an array"))?;

            array.push(value.clone());
        } else {
            let mut array = Array::new();
            array.push(value.clone());
            table[key] = array.into();
        }

        Ok(())
    }

    pub fn update_value_of_array(
        &mut self,
        table: &TablePath,
        key: &str,
        old_value: Option<&Item>,
        value: &Item,
    ) -> CargoResult<()> {
        let base_table = self.toml_manifest.get_table_like_mut(table.as_slice())?;

        let target_item = base_table
            .get_mut(key)
            .ok_or_else(|| error!("`{key}` could not be found"))?;

        let target_array = target_item
            .as_array_mut()
            .ok_or_else(|| error!("`{key}` is not an array"))?;

        let source_value = value
            .as_value()
            .ok_or_else(|| error!("`{key}` value is not a value"))?;

        if let Some(old_value) = old_value.and_then(Item::as_value) {
            if let Some(old_str) = old_value.as_str() {
                if let Some(existing) = target_array
                    .iter_mut()
                    .find(|item| item.as_str() == Some(old_str))
                {
                    *existing = source_value.clone();
                    return Ok(());
                }
            }
        }

        target_array.push(source_value.clone());

        Ok(())
    }

    pub fn remove_value_from_array(
        &mut self,
        table: &TablePath,
        key: &str,
        value: &Item,
    ) -> CargoResult<()> {
        let table = self.toml_manifest.get_table_like_mut(table.as_slice())?;

        let item = table
            .get_mut(key)
            .ok_or_else(|| error!("`{key}` could not be found"))?;

        let array = item
            .as_array_mut()
            .ok_or_else(|| error!("`{key}` is not an array"))?;

        let value = value
            .as_value()
            .and_then(|value| value.as_str())
            .ok_or_else(|| error!("Value is not a string"))?;

        let index = array.iter().position(|item| item.as_str() == Some(value));

        if let Some(idx) = index {
            array.remove(idx);
        }

        Ok(())
    }

    // ===============================================
    pub fn get_key(&self, table: &TablePath, key: &str) -> CargoResult<&Item> {
        let item = self.toml_manifest.get_table_like(table.as_slice())?;

        item.get(key)
            .ok_or_else(|| error!("`{key}` could not be found"))
    }

    pub fn add_key(
        &mut self,
        table: &TablePath,
        key: &str,
        value: &Item,
        inherit_mode: &InheritMode,
    ) -> CargoResult<()> {
        let table = self
            .toml_manifest
            .get_or_create_table_like_mut(table.as_slice())?;

        let source_item = inherit_mode.resolve_key(key.to_string(), value.clone())?;

        table[key] = source_item;

        Ok(())
    }

    pub fn update_key(
        &mut self,
        table: &TablePath,
        key: &str,
        value: &Item,
        inherit_mode: &InheritMode,
    ) -> CargoResult<()> {
        let base_table = self.toml_manifest.get_table_like_mut(table.as_slice())?;

        let source_item = if base_table.is_inline_table() {
            value.clone()
        } else {
            inherit_mode.resolve_key(key.to_string(), value.clone())?
        };

        let target_item = base_table
            .get_mut(key)
            .ok_or_else(|| error!("`{key}` could not be found"))?;

        if let Some(old_table) = target_item.as_table_mut() {
            if let Some(new_table) = source_item.as_table() {
                // Keep the old layout decoration (headers, bracket formatting, spacing, comments)
                let old_decor = old_table.decor().clone();

                // Keep its original file position index so it doesn't shift to the bottom
                let old_position = old_table.position();

                // Overwrite the table content
                *old_table = new_table.clone();

                // Re-apply the formatting layout anchors
                *old_table.decor_mut() = old_decor;
                old_table.set_position(old_position);
            }
        } else if let Some(old_value) = target_item.as_value_mut() {
            match old_value {
                Value::Array(old_array) => {
                    if let Some(new_array) = source_item.as_value().and_then(Value::as_array) {
                        let old_decor = old_array.decor().clone();

                        old_array.clear();
                        old_array.extend(new_array.iter().cloned());

                        *old_array.decor_mut() = old_decor;
                    }
                }
                Value::InlineTable(old_inline_table) => {
                    if let Some(mut new_value) = source_item.as_value().cloned() {
                        let old_decor = old_inline_table.decor().clone();
                        *new_value.decor_mut() = old_decor;
                        *old_value = new_value;
                    }
                }
                Value::String(val_string) => {
                    if source_item.as_value().and_then(|v| v.as_str()) != Some(val_string.value()) {
                        if let Some(mut new_value) = source_item.as_value().cloned() {
                            *new_value.decor_mut() = val_string.decor().clone();
                            *old_value = new_value;
                        }
                    }
                }
                Value::Integer(val_int) => {
                    if source_item.as_value().and_then(|v| v.as_integer()) != Some(*val_int.value())
                    {
                        if let Some(mut new_value) = source_item.as_value().cloned() {
                            *new_value.decor_mut() = val_int.decor().clone();
                            *old_value = new_value;
                        }
                    }
                }
                Value::Float(val_float) => {
                    if source_item.as_value().and_then(|v| v.as_float()) != Some(*val_float.value())
                    {
                        if let Some(mut new_value) = source_item.as_value().cloned() {
                            *new_value.decor_mut() = val_float.decor().clone();
                            *old_value = new_value;
                        }
                    }
                }
                Value::Boolean(val_bool) => {
                    if source_item.as_value().and_then(|v| v.as_bool()) != Some(*val_bool.value()) {
                        if let Some(mut new_value) = source_item.as_value().cloned() {
                            *new_value.decor_mut() = val_bool.decor().clone();
                            *old_value = new_value;
                        }
                    }
                }
                Value::Datetime(val_datetime) => {
                    if source_item.as_value().and_then(|v| v.as_datetime())
                        != Some(val_datetime.value())
                    {
                        if let Some(mut new_value) = source_item.as_value().cloned() {
                            *new_value.decor_mut() = val_datetime.decor().clone();
                            *old_value = new_value;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn remove_key(&mut self, table: &TablePath, key: &str) -> CargoResult<()> {
        let table = self.toml_manifest.get_table_like_mut(table.as_slice())?;

        if let Some(target_table) = table.as_table_like_mut() {
            target_table.remove(key);
        }

        Ok(())
    }

    // ===============================================
    pub fn get_items_at(&self, table: &TablePath) -> CargoResult<&Item> {
        let item = self
            .toml_manifest
            .get_table_like(table.as_slice())
            .with_context(|| format!("Item of `{}` could not be found", table))?;

        Ok(item)
    }

    pub fn add_items_at(
        &mut self,
        table: &TablePath,
        value: &Item,
        inherit_mode: &InheritMode,
    ) -> CargoResult<()> {
        let base_table = self
            .toml_manifest
            .get_or_create_table_like_mut(table.as_slice())?;

        let source_item = inherit_mode.resolve_items(value.clone())?;

        let table_like_source = source_item
            .as_table_like()
            .ok_or_else(|| error!("Incoming value is not a table-like type"))?;

        if let Some(old_value) = base_table.as_value_mut() {
            if let Some(inline_table) = old_value.as_inline_table_mut() {
                // Handle Inline Table: Merge inside the curly braces `{ ... }`
                for (key, value) in table_like_source.iter() {
                    if inline_table.get(key).is_none() {
                        if let Some(value) = value.as_value() {
                            inline_table.insert(key, value.clone());
                        }
                    }
                }
            }
        } else if let Some(target_table) = base_table.as_table_mut() {
            // Handle Standard Table: Merge fields cleanly
            for (key, value) in table_like_source.iter() {
                if target_table.get(key).is_none() {
                    target_table.insert(key, value.clone());
                }
            }
        } else {
            return Err(error!("Item is not a table"));
        }

        Ok(())
    }

    pub fn update_items_at(
        &mut self,
        table: &TablePath,
        value: &Item,
        inherit_mode: &InheritMode,
    ) -> CargoResult<()> {
        let base_table = self.toml_manifest.get_table_like_mut(table.as_slice())?;

        let source_item = inherit_mode.resolve_items(value.clone())?;

        let table_like_source = source_item
            .as_table_like()
            .ok_or_else(|| error!("Incoming value is not a table-like type"))?;

        if let Some(old_value) = base_table.as_value_mut() {
            if let Some(inline_table) = old_value.as_inline_table_mut() {
                for (key, value) in table_like_source.iter() {
                    if let Some(new_value) = value.as_value() {
                        inline_table.insert(key, new_value.clone());
                    }
                }
            }
        } else if let Some(target_table) = base_table.as_table_mut() {
            for (key, value) in table_like_source.iter() {
                target_table.insert(key, value.clone());
            }
        } else {
            return Err(error!("Item is not a table"));
        }

        Ok(())
    }

    pub fn remove_items_at(
        &mut self,
        table: &TablePath,
        targets: Option<&[String]>,
    ) -> CargoResult<()> {
        let base_table = self.toml_manifest.get_table_like_mut(table.as_slice())?;

        if let Some(old_value) = base_table.as_value_mut() {
            if let Some(inline_table) = old_value.as_inline_table_mut() {
                match targets {
                    Some(targets) => {
                        for key in targets {
                            inline_table.remove(key);
                        }
                    }
                    None => {
                        inline_table.clear();
                    }
                }
            }
        } else if let Some(target_table) = base_table.as_table_mut() {
            match targets {
                Some(targets) => {
                    for key in targets {
                        target_table.remove(key);
                    }
                }
                None => {
                    target_table.clear();
                }
            }
        } else {
            return Err(error!("Item is not a table"));
        }

        Ok(())
    }
}
