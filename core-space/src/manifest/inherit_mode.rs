use crate::manifest::Item;
use crate::manifest::Table;

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

                if item.is_table_like() {
                    if let Some(table) = item.as_table_like_mut() {
                        table.insert("workspace", true.into());
                    }
                } else {
                    return Err(error!(
                        "can not make `{key}` inherit partially, item not table like"
                    ));
                }

                Ok(item)
            }

            Self::None => Ok(item),
        }
    }

    pub fn resolve_items(&self, item: Item) -> CargoResult<Item> {
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

                table_like.insert("workspace", true.into());

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
