mod dep_kind;
mod dep_source;
mod dependency;
mod edition;
mod package_items;

pub use dep_kind::DepTableKind;
pub use dep_kind::DepTableSection;
pub use dep_source::GitSource;
pub use dep_source::PathSource;
pub use dep_source::RegistrySource;
pub use dep_source::Source;
pub use dependency::DataInherit;
pub use dependency::Dependency;
pub use dependency::RulesInherit;
pub use edition::Edition;
pub use edition::Resolver;
pub use package_items::PackageItems;

use crate::manifest::Array;
use crate::manifest::Item;
use crate::manifest::Table;
use crate::manifest::Value;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Workspace {
    members: Option<Vec<String>>,
    resolver: Option<Resolver>,
    default_members: Option<Vec<String>>,
    exclude: Option<Vec<String>>,

    package_items: PackageItems,
}

impl Workspace {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_members(mut self, members: &[String]) -> Self {
        self.members = Some(members.to_vec());
        self
    }

    pub fn with_resolver(mut self, resolver: impl Into<Resolver>) -> Self {
        self.resolver = Some(resolver.into());
        self
    }

    pub fn with_default_members(mut self, members: Vec<String>) -> Self {
        self.default_members = Some(members);
        self
    }

    pub fn with_exclude(mut self, exclude: Vec<String>) -> Self {
        self.exclude = Some(exclude);
        self
    }

    pub fn with_package_items(mut self, pkg_items: &PackageItems) -> Self {
        self.package_items = pkg_items.clone();
        self
    }

    pub fn get_resolver(&self) -> Option<&Resolver> {
        self.resolver.as_ref()
    }

    pub fn get_default_members(&self) -> Option<&[String]> {
        self.default_members.as_deref()
    }

    pub fn get_exclude(&self) -> Option<&[String]> {
        self.exclude.as_deref()
    }

    pub fn get_package_items(&self) -> &PackageItems {
        &self.package_items
    }
}

impl Workspace {
    pub fn from_toml(item: &Item) -> Self {
        Self {
            members: item.get("members").and_then(Self::string_array),
            resolver: item
                .get("resolver")
                .and_then(|v| v.as_str())
                .and_then(|v| v.parse::<Resolver>().ok()),
            default_members: item.get("default-members").and_then(Self::string_array),
            exclude: item.get("exclude").and_then(Self::string_array),
            package_items: item
                .get("package")
                .map(PackageItems::from_toml)
                .unwrap_or_default(),
        }
    }

    pub fn to_toml(&self) -> Item {
        let mut table = Table::new();

        if let Some(value) = &self.members {
            let mut array = Array::new();

            for value in value {
                array.push(value.as_str());
            }

            table.insert("members", array.into());
        }

        if let Some(value) = &self.resolver {
            table.insert("resolver", value.to_manifest().into());
        }

        if let Some(value) = &self.default_members {
            let mut array = Array::new();

            for value in value {
                array.push(value.as_str());
            }

            table.insert("default-members", array.into());
        }

        if let Some(value) = &self.exclude {
            let mut array = Array::new();

            for value in value {
                array.push(value.as_str());
            }

            table.insert("exclude", array.into());
        }

        table.insert("package", self.package_items.to_toml());

        Item::Table(table)
    }

    fn string_array(item: &Item) -> Option<Vec<String>> {
        item.as_array().map(|array| {
            array
                .iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect()
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Package {
    name: String,
    package_items: PackageItems,
}

impl Package {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, value: impl Into<String>) -> Self {
        self.name = value.into();
        self
    }

    pub fn with_package_items(mut self, pkg_items: &PackageItems) -> Self {
        self.package_items = pkg_items.clone();
        self
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_package_items(&self) -> &PackageItems {
        &self.package_items
    }
}

impl Package {
    pub fn from_toml(item: &Item) -> Self {
        Self {
            name: item
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            package_items: PackageItems::from_toml(item),
        }
    }

    pub fn to_toml(&self) -> Item {
        let mut table = Table::new();

        table.insert("name", self.name.as_str().into());

        if let Item::Table(package_items) = self.package_items.to_toml() {
            table.extend(package_items);
        }

        Item::Table(table)
    }
}
