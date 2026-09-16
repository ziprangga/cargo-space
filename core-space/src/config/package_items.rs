use crate::manifest::Array;
use crate::manifest::Item;
use crate::manifest::Table;
use crate::manifest::Value;

use super::Edition;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct PackageItems {
    version: Option<String>,
    edition: Option<Edition>,
    authors: Option<Vec<String>>,
    license: Option<String>,
    repository: Option<String>,
    description: Option<String>,
    publish: Option<Vec<String>>,

    rust_version: Option<String>,
    documentation: Option<String>,
    readme: Option<String>,
    homepage: Option<String>,
    license_file: Option<String>,
    keywords: Option<Vec<String>>,
    categories: Option<Vec<String>>,
    include: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
}

impl PackageItems {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_version(mut self, value: impl Into<String>) -> Self {
        self.version = Some(value.into());
        self
    }

    pub fn with_edition(mut self, value: impl Into<Edition>) -> Self {
        self.edition = Some(value.into());
        self
    }

    pub fn with_authors(mut self, value: Vec<String>) -> Self {
        self.authors = Some(value);
        self
    }

    pub fn with_license(mut self, value: impl Into<String>) -> Self {
        self.license = Some(value.into());
        self
    }

    pub fn with_repository(mut self, value: impl Into<String>) -> Self {
        self.repository = Some(value.into());
        self
    }

    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    pub fn with_publish(mut self, value: Option<Vec<String>>) -> Self {
        self.publish = value;
        self
    }

    pub fn with_documentation(mut self, value: impl Into<String>) -> Self {
        self.documentation = Some(value.into());
        self
    }

    pub fn with_readme(mut self, value: impl Into<String>) -> Self {
        self.readme = Some(value.into());
        self
    }

    pub fn with_homepage(mut self, value: impl Into<String>) -> Self {
        self.homepage = Some(value.into());
        self
    }

    pub fn with_rust_version(mut self, value: impl Into<String>) -> Self {
        self.rust_version = Some(value.into());
        self
    }

    pub fn with_license_file(mut self, value: impl Into<String>) -> Self {
        self.license_file = Some(value.into());
        self
    }

    pub fn with_keywords(mut self, value: Vec<String>) -> Self {
        self.keywords = Some(value);
        self
    }

    pub fn with_categories(mut self, value: Vec<String>) -> Self {
        self.categories = Some(value);
        self
    }

    pub fn with_include(mut self, value: Vec<String>) -> Self {
        self.include = Some(value);
        self
    }

    pub fn with_exclude(mut self, value: Vec<String>) -> Self {
        self.exclude = Some(value);
        self
    }

    pub fn get_version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    pub fn get_edition(&self) -> Option<&Edition> {
        self.edition.as_ref()
    }

    pub fn get_authors(&self) -> Option<&Vec<String>> {
        self.authors.as_ref()
    }

    pub fn get_license(&self) -> Option<&str> {
        self.license.as_deref()
    }

    pub fn get_repository(&self) -> Option<&str> {
        self.repository.as_deref()
    }

    pub fn get_description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn get_publish(&self) -> Option<&Vec<String>> {
        self.publish.as_ref()
    }

    pub fn get_rust_version(&self) -> Option<&str> {
        self.rust_version.as_deref()
    }

    pub fn get_documentation(&self) -> Option<&str> {
        self.documentation.as_deref()
    }

    pub fn get_readme(&self) -> Option<&str> {
        self.readme.as_deref()
    }

    pub fn get_homepage(&self) -> Option<&str> {
        self.homepage.as_deref()
    }

    pub fn get_license_file(&self) -> Option<&str> {
        self.license_file.as_deref()
    }

    pub fn get_keywords(&self) -> Option<&Vec<String>> {
        self.keywords.as_ref()
    }

    pub fn get_categories(&self) -> Option<&Vec<String>> {
        self.categories.as_ref()
    }

    pub fn get_include(&self) -> Option<&Vec<String>> {
        self.include.as_ref()
    }

    pub fn get_exclude(&self) -> Option<&Vec<String>> {
        self.exclude.as_ref()
    }
}

impl PackageItems {
    pub fn from_toml(item: &Item) -> Self {
        Self {
            version: item
                .get("version")
                .and_then(|v| v.as_str())
                .map(String::from),
            edition: item
                .get("edition")
                .and_then(|v| v.as_str())
                .and_then(|v| v.parse::<Edition>().ok()),
            authors: item.get("authors").and_then(Self::string_array),
            license: item
                .get("license")
                .and_then(|v| v.as_str())
                .map(String::from),
            repository: item
                .get("repository")
                .and_then(|v| v.as_str())
                .map(String::from),
            description: item
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from),
            publish: item.get("publish").and_then(Self::string_array),

            rust_version: item
                .get("rust-version")
                .and_then(|v| v.as_str())
                .map(String::from),
            documentation: item
                .get("documentation")
                .and_then(|v| v.as_str())
                .map(String::from),
            readme: item
                .get("readme")
                .and_then(|v| v.as_str())
                .map(String::from),
            homepage: item
                .get("homepage")
                .and_then(|v| v.as_str())
                .map(String::from),
            license_file: item
                .get("license-file")
                .and_then(|v| v.as_str())
                .map(String::from),
            keywords: item.get("keywords").and_then(Self::string_array),
            categories: item.get("categories").and_then(Self::string_array),
            include: item.get("include").and_then(Self::string_array),
            exclude: item.get("exclude").and_then(Self::string_array),
        }
    }

    pub fn to_toml(&self) -> Item {
        let mut table = Table::new();

        if let Some(value) = &self.version {
            table.insert("version", value.as_str().into());
        }

        if let Some(value) = &self.edition {
            table.insert("edition", value.to_string().into());
        }

        if let Some(value) = &self.authors {
            table.insert("authors", Self::string_array_to_toml(value));
        }

        if let Some(value) = &self.license {
            table.insert("license", value.as_str().into());
        }

        if let Some(value) = &self.repository {
            table.insert("repository", value.as_str().into());
        }

        if let Some(value) = &self.description {
            table.insert("description", value.as_str().into());
        }

        if let Some(value) = &self.publish {
            table.insert("publish", Self::string_array_to_toml(value));
        }

        if let Some(value) = &self.rust_version {
            table.insert("rust-version", value.as_str().into());
        }

        if let Some(value) = &self.documentation {
            table.insert("documentation", value.as_str().into());
        }

        if let Some(value) = &self.readme {
            table.insert("readme", value.as_str().into());
        }

        if let Some(value) = &self.homepage {
            table.insert("homepage", value.as_str().into());
        }

        if let Some(value) = &self.license_file {
            table.insert("license-file", value.as_str().into());
        }

        if let Some(value) = &self.keywords {
            table.insert("keywords", Self::string_array_to_toml(value));
        }

        if let Some(value) = &self.categories {
            table.insert("categories", Self::string_array_to_toml(value));
        }

        if let Some(value) = &self.include {
            table.insert("include", Self::string_array_to_toml(value));
        }

        if let Some(value) = &self.exclude {
            table.insert("exclude", Self::string_array_to_toml(value));
        }

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

    fn string_array_to_toml(values: &[String]) -> Item {
        let mut array = Array::new();

        for value in values {
            array.push(value.as_str());
        }

        array.into()
    }
}
