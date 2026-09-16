use std::fmt;

#[derive(Debug, Default, Clone)]
pub struct TablePath(Vec<String>);

impl TablePath {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn push(mut self, header: impl Into<String>) -> Self {
        self.0.push(header.into());
        self
    }

    pub fn as_slice(&self) -> &[String] {
        &self.0
    }
}

impl fmt::Display for TablePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.join("."))
    }
}
