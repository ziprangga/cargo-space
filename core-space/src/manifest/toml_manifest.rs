// This file contains code originally derived from cargo-edit and has been
// modified and adapted for cargo-space.

use crate::errors::{CargoResult, Context, error};
use std::path::Path;

pub type Item = toml_edit::Item;
pub type Array = toml_edit::Array;
pub type InlineTable = toml_edit::InlineTable;
pub type Value = toml_edit::Value;
pub type Table = toml_edit::Table;
pub type DocMut = toml_edit::DocumentMut;
pub type Doc<T> = toml_edit::Document<T>;

/// A TOML manifest backed by [toml_edit::DocumentMut].
///
/// TomlManifest provides access to TOML data and supports traversal through
/// table-like structures, including both standard tables and inline tables.
///
/// Array-of-tables ([[table]]) are not traversed by the table-like traversal
/// methods and require separate handling.
#[derive(Debug, Clone, Default)]
pub struct TomlManifest {
    data: DocMut,
}

impl TomlManifest {
    /// Creates an empty TOML manifest.
    pub fn new() -> Self {
        Self {
            data: DocMut::new(),
        }
    }

    /// Loads and parses a TOML manifest from the specified path.
    ///
    /// Returns an error if the file cannot be read or if its contents
    /// cannot be parsed as valid TOML.
    pub fn from_manifest_path(manifest_path: &Path) -> CargoResult<Self> {
        let content = std::fs::read_to_string(manifest_path)
            .with_context(|| format!("failed to read {}", manifest_path.display()))?;

        let data = content
            .parse::<DocMut>()
            .with_context(|| format!("failed to parse {}", manifest_path.display()))?;

        Ok(Self { data })
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns an immutable reference to the underlying TOML document.
    pub fn data(&self) -> &DocMut {
        &self.data
    }

    /// Returns a mutable reference to the underlying TOML document.
    pub fn data_mut(&mut self) -> &mut DocMut {
        &mut self.data
    }

    /// Returns the table-like item at the specified path.
    ///
    /// Each path segment may refer to either a standard TOML table or an
    /// inline table. This allows the same traversal to work with both forms,
    /// for example:
    ///
    /// ```text
    /// [dependencies.clap]
    /// version = "0.1"
    /// ```
    ///
    /// and:
    ///
    /// ```text
    /// [dependencies]
    /// clap = { version = "0.1" }
    /// ```
    ///
    /// Returns an error if any path segment does not exist or does not
    /// reference a table-like item.
    ///
    /// Array-of-tables are not traversed by this method.
    pub fn get_table_like(&self, table_path: &[String]) -> CargoResult<&Item> {
        Self::descent(self.data.as_item(), table_path)
    }

    /// Returns a mutable table-like item at the specified path.
    ///
    /// Missing path segments are not created. Existing inline tables remain
    /// inline tables when traversed.
    ///
    /// Returns an error if any path segment does not exist or does not
    /// reference a table-like item.
    ///
    /// Array-of-tables are not traversed by this method.
    pub fn get_table_like_mut(&mut self, table_path: &[String]) -> CargoResult<&mut Item> {
        Self::descent_mut(self.data.as_item_mut(), table_path)
    }

    /// Returns a mutable table-like item at the specified path, creating
    /// missing path segments as standard TOML tables.
    ///
    /// Existing inline tables remain inline tables when traversed.
    ///
    /// Returns an error if an existing path segment does not reference a
    /// table-like item.
    ///
    /// Array-of-tables are not traversed by this method.
    pub fn get_or_create_table_like_mut(
        &mut self,
        table_path: &[String],
    ) -> CargoResult<&mut Item> {
        Self::descent_or_create_mut(self.data.as_item_mut(), table_path)
    }

    fn descent<'a>(mut item: &'a Item, sections: &[String]) -> CargoResult<&'a Item> {
        for section in sections {
            item = item
                .get(section)
                .ok_or_else(|| error!("Table `{section}` could not be found"))?;

            if !item.is_table_like() {
                return Err(error!(
                    "`{section}` is not a standard table or inline table"
                ));
            }
        }

        Ok(item)
    }

    fn descent_mut<'a>(mut item: &'a mut Item, sections: &[String]) -> CargoResult<&'a mut Item> {
        for section in sections {
            item = item
                .get_mut(section)
                .ok_or_else(|| error!("Table `{section}` could not be found"))?;

            if !item.is_table_like() {
                return Err(error!(
                    "`{section}` is not a standard table or inline table"
                ));
            }
        }

        Ok(item)
    }

    fn descent_or_create_mut<'a>(
        mut item: &'a mut Item,
        sections: &[String],
    ) -> CargoResult<&'a mut Item> {
        for section in sections {
            item = item[&section].or_insert(toml_edit::table());

            if !item.is_table_like() {
                return Err(error!(
                    "`{section}` is not a standard table or inline table"
                ));
            }
        }

        Ok(item)
    }
}
