mod inherit_mode;
mod man_util;
mod table_path;
mod toml_manifest;

#[cfg(test)]
mod tests;

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

    pub fn from_toml_path(manifest_path: &Path) -> CargoResult<Self> {
        let toml_path = manifest_path.to_path_buf();
        let content = std::fs::read_to_string(manifest_path)
            .with_context(|| format!("failed to read {}", manifest_path.display()))?;

        let data = content
            .parse::<DocMut>()
            .with_context(|| format!("failed to parse {}", manifest_path.display()))?;

        let toml_manifest = TomlManifest::new().with_data(data);

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

// impl Manifest {
//     pub fn is_table_exist(&self, table: &TablePath) -> bool {
//         self.toml_manifest.get_table_like(table.as_slice()).is_ok()
//     }

//     pub fn is_table_inline_table(&self, table: &TablePath) -> bool {
//         self.toml_manifest
//             .get_table_like(table.as_slice())
//             .ok()
//             .is_some_and(|table| table.is_inline_table())
//     }

//     pub fn is_key_exist(&self, table: &TablePath, key: &str) -> bool {
//         self.toml_manifest
//             .get_table_like(table.as_slice())
//             .ok()
//             .and_then(|table| table.get(key))
//             .is_some()
//     }

//     pub fn is_key_inline_table(&self, table: &TablePath, key: &str) -> bool {
//         self.toml_manifest
//             .get_table_like(table.as_slice())
//             .ok()
//             .and_then(|table| table.get(key))
//             .is_some_and(Item::is_inline_table)
//     }
// }

impl Manifest {
    pub fn get_array(&self, table: &TablePath, key: &str) -> CargoResult<&Array> {
        let item = self.toml_manifest.get_table_like(table.as_slice())?;

        item.get(key)
            .and_then(Item::as_array)
            .ok_or_else(|| error!("`{key}` is not an array"))
    }
    // pub fn get_array(&self, table: &TablePath, key: &str) -> Option<&Array> {
    //     let item = match self.toml_manifest.get_table_like(table.as_slice()) {
    //         Ok(item) => item,
    //         Err(err) => {
    //             // `err` is the original low-level error.
    //             eprintln!("{err}");
    //             return None;
    //         }
    //     };

    //     item.get(key).and_then(Item::as_array)
    // }

    pub fn add_value_to_array(
        &mut self,
        table: &TablePath,
        key: &str,
        value: &Item,
    ) -> CargoResult<()> {
        let base_table = self
            .toml_manifest
            .get_or_create_table_like_mut(table.as_slice())?;

        let value = value
            .as_value()
            .ok_or_else(|| error!("Value is not a value"))?;

        let target_item_exist = base_table.get(key).is_some();

        if target_item_exist {
            let array = base_table
                .get_mut(key)
                .and_then(Item::as_array_mut)
                .ok_or_else(|| error!("`{key}` is not an array"))?;

            let trailing_comma = array.trailing_comma();

            if trailing_comma {
                let trailing = array.trailing().clone();

                let mut new_value = value.clone();

                new_value.decor_mut().set_prefix(trailing);
                array.push_formatted(new_value);

                array.set_trailing("\n");
            } else {
                array.push_formatted(value.clone());
            }
        } else {
            let mut array = Array::new();
            array.push(value.clone());
            base_table[key] = array.into();
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

        let trailing_comma = target_array.trailing_comma();

        if let Some(old_value) = old_value.and_then(Item::as_value) {
            let index = target_array
                .iter()
                .position(|item| item.as_str() == old_value.as_str());

            if let Some(idx) = index {
                target_array.replace(idx, source_value.clone());
            }
        } else {
            if trailing_comma {
                let trailing = target_array.trailing().clone();

                let mut new_value = source_value.clone();

                new_value.decor_mut().set_prefix(trailing);
                target_array.push_formatted(new_value);

                target_array.set_trailing("\n");
            } else {
                target_array.push_formatted(source_value.clone());
            }
        }

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

        let target_array = item
            .as_array_mut()
            .ok_or_else(|| error!("`{key}` is not an array"))?;

        let target_value = value
            .as_value()
            .and_then(|value| value.as_str())
            .ok_or_else(|| error!("Value is not a string"))?;

        let index = target_array
            .iter()
            .position(|item| item.as_str() == Some(target_value));

        if let Some(idx) = index {
            if let Some(v) = target_array.get_mut(idx + 1) {
                if v.decor().prefix().is_some() {
                    v.decor_mut().set_prefix("");
                }
            }

            target_array.remove(idx);
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
        let base_table = self
            .toml_manifest
            .get_or_create_table_like_mut(table.as_slice())?;

        if base_table.get(key).is_some() {
            return Ok(());
        }

        let source_item = inherit_mode.resolve_key(key.to_string(), value.clone())?;

        if !inherit_mode.is_none() {
            let source_table = source_item
                .as_table()
                .ok_or_else(|| error!("Resolved item is not table-like"))?;

            let target_table = base_table
                .as_table_mut()
                .ok_or_else(|| error!("Target item is not table-like"))?;

            for (key, value) in source_table.iter() {
                target_table.insert(key, value.clone());
            }
        } else {
            base_table[key] = source_item;
        }

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

        let source_item = inherit_mode.resolve_key(key.to_string(), value.clone())?;

        let target_item = base_table
            .get_mut(key)
            .ok_or_else(|| error!("`{key}` could not be found"))?;

        let decor = if let Some(table) = target_item.as_table() {
            table.decor().clone()
        } else if let Some(value) = target_item.as_value() {
            value.decor().clone()
        } else {
            Default::default()
        };

        if !inherit_mode.is_none() {
            let source_table = source_item
                .as_table()
                .ok_or_else(|| error!("Resolved item is not table-like"))?;

            let resolved_item = source_table
                .get(key)
                .ok_or_else(|| error!("Resolved key `{key}` could not be found"))?
                .clone();

            *target_item = resolved_item;
        } else {
            *target_item = source_item;
        }

        match target_item {
            Item::Value(value) => *value.decor_mut() = decor,
            Item::Table(table) => {
                if inherit_mode.is_full() {
                    let workspace_val = table
                        .get_mut("workspace")
                        .ok_or_else(|| error!("`workspace` key could not be found"))?
                        .as_value_mut()
                        .ok_or_else(|| error!("`workspace` key is not a value type"))?;

                    *workspace_val.decor_mut() = decor;
                } else {
                    *table.decor_mut() = decor
                }
            }
            _ => {}
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
                        if let Some(old_value) = inline_table.get(key) {
                            let decor = old_value.decor().clone();
                            let mut new_value = new_value.clone();
                            *new_value.decor_mut() = decor;
                            inline_table.insert(key, new_value);
                        } else {
                            inline_table.insert(key, new_value.clone());
                        }
                    }
                }
            }
        } else if let Some(target_table) = base_table.as_table_mut() {
            for (key, value) in table_like_source.iter() {
                if let Some(old_value) = target_table.get(key) {
                    let decor = match old_value {
                        Item::Value(value) => value.decor().clone(),
                        Item::Table(table) => table.decor().clone(),
                        _ => Default::default(),
                    };

                    let mut new_value = value.clone();

                    match &mut new_value {
                        Item::Value(value) => *value.decor_mut() = decor,
                        Item::Table(table) => {
                            if inherit_mode.is_full() {
                                let workspace_val = table
                                    .get_mut("workspace")
                                    .ok_or_else(|| error!("`workspace` key could not be found"))?
                                    .as_value_mut()
                                    .ok_or_else(|| error!("`workspace` key is not a value type"))?;

                                *workspace_val.decor_mut() = decor;
                            } else {
                                *table.decor_mut() = decor
                            }
                        }
                        _ => {}
                    }

                    target_table.insert(key, new_value);
                } else {
                    target_table.insert(key, value.clone());
                }
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
