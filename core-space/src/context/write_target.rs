use super::Modifier;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TargetWriter {
    Space,
    Pkg(String),
    #[default]
    None,
}

impl TargetWriter {
    pub fn space() -> Self {
        Self::Space
    }

    pub fn pkg(name: impl Into<String>) -> Self {
        Self::Pkg(name.into())
    }

    pub fn none() -> Self {
        Self::None
    }

    pub fn is_space(&self) -> bool {
        matches!(self, Self::Space)
    }

    pub fn is_pkg(&self) -> bool {
        matches!(self, Self::Pkg(_))
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn pkg_name(&self) -> Option<&str> {
        match self {
            Self::Pkg(name) => Some(name),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TargetToml {
    target: TargetWriter,
    modifiers: Vec<Modifier>,
}

impl TargetToml {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_target(mut self, target: TargetWriter) -> Self {
        self.target = target;
        self
    }

    pub fn with_modifiers(mut self, modifiers: &[Modifier]) -> Self {
        self.modifiers = modifiers.to_vec();
        self
    }

    pub fn get_target(&self) -> &TargetWriter {
        &self.target
    }

    pub fn get_modifiers(&self) -> &[Modifier] {
        &self.modifiers
    }

    pub fn add_modifier(&mut self, modifier: &Modifier) {
        self.modifiers.push(modifier.clone());
    }
}
