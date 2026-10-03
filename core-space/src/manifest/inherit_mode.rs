use crate::manifest::Item;
use crate::manifest::Table;
use crate::manifest::Value;
use crate::manifest::into_inline_table_item;

use crate::errors::{CargoResult, error};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum InheritMode {
    Full,
    Partial,

    #[default]
    None,
}

impl InheritMode {
    pub fn resolve_key(&self, key: String, item: Item) -> CargoResult<Item> {
        match self {
            Self::Full => {
                let mut dotted = Table::new();
                dotted.set_dotted(true);
                dotted.insert("workspace", true.into());

                let mut table = Table::new();
                table.set_dotted(true);
                table.insert(&key, Item::Table(dotted));

                Ok(Item::Table(table))
            }

            Self::Partial => {
                let mut item = item;

                if let Some(table) = item.as_table_like_mut() {
                    table.insert("workspace", true.into());
                } else {
                    let mut inner_table = Table::new();
                    inner_table.insert("workspace", true.into());
                    let new_item = into_inline_table_item(Item::Table(inner_table))?;

                    item = new_item
                };

                let mut table = Table::new();
                table.insert(&key, item);
                let new_item_table = Item::Table(table);

                Ok(new_item_table)
            }

            Self::None => Ok(item),
        }
    }

    pub fn resolve_items_table(&self, item: Item) -> CargoResult<Item> {
        match self {
            Self::Full => {
                let Some(table_like) = item.as_table_like() else {
                    return Err(error!("can not make it inherit, item not table like"));
                };

                let mut table = Table::new();
                table.set_dotted(true);

                for (key, _) in table_like.iter() {
                    let mut workspace = Table::new();
                    workspace.set_dotted(true);
                    workspace.insert("workspace", true.into());

                    table.insert(key, Item::Table(workspace));
                }

                Ok(Item::Table(table))
            }

            Self::Partial => {
                let mut item = item;

                let Some(table_like) = item.as_table_like_mut() else {
                    return Err(error!("can not make it inherit, item not table like"));
                };

                let entries: Vec<(String, Item)> = table_like
                    .iter()
                    .map(|(key, value)| (key.to_string(), value.clone()))
                    .collect();

                for (key, value) in entries {
                    let item = if let Some(table_in) = value.as_table() {
                        let mut new_table = table_in.clone();
                        new_table.insert("workspace", true.into());

                        Item::Table(new_table)
                    } else if let Some(table_in) = value.as_inline_table() {
                        let mut inline_table = table_in.clone();
                        inline_table.insert("workspace", true.into());

                        Item::Value(Value::InlineTable(inline_table))
                    } else {
                        let mut inner_table = Table::new();
                        inner_table.insert("workspace", true.into());
                        into_inline_table_item(Item::Table(inner_table))?
                    };

                    table_like.insert(&key, item);
                }

                Ok(item)
            }

            Self::None => Ok(item),
        }
    }
}

impl InheritMode {
    pub fn is_full(&self) -> bool {
        matches!(self, Self::Full)
    }

    pub fn is_partial(&self) -> bool {
        matches!(self, Self::Partial)
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

impl From<&str> for InheritMode {
    fn from(value: &str) -> Self {
        match value {
            "full" => Self::Full,
            "partial" => Self::Partial,
            "none" => Self::None,
            _ => Self::None,
        }
    }
}

impl From<String> for InheritMode {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}
