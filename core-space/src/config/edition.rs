use std::str::FromStr;

use crate::errors::Error;
use crate::errors::bail_out;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub enum Resolver {
    V1,
    V2,
    #[default]
    V3,
}

impl Resolver {
    pub fn to_string(&self) -> String {
        match self {
            Self::V1 => "1",
            Self::V2 => "2",
            Self::V3 => "3",
        }
        .to_owned()
    }
}

impl FromStr for Resolver {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "1" => Ok(Self::V1),
            "2" => Ok(Self::V2),
            "3" => Ok(Self::V3),
            s => bail_out!(
                "`resolver` setting `{}` is not valid, valid options are \"1\", \"2\" or \"3\"",
                s
            ),
        }
    }
}

impl From<&str> for Resolver {
    fn from(value: &str) -> Self {
        value.parse().expect("invalid resolver")
    }
}

impl From<String> for Resolver {
    fn from(value: String) -> Self {
        value.parse().expect("invalid resolver")
    }
}

#[derive(Default, Clone, Copy, Debug, Hash, PartialOrd, Ord, Eq, PartialEq)]
pub enum Edition {
    Edition2015,

    Edition2018,

    Edition2021,

    #[default]
    Edition2024,
}

impl Edition {
    pub const EDITIONS: &'static [(&'static str, Self)] = &[
        ("2015", Self::Edition2015),
        ("2018", Self::Edition2018),
        ("2021", Self::Edition2021),
        ("2024", Self::Edition2024),
    ];

    pub fn default_resolver(&self) -> Resolver {
        if *self >= Edition::Edition2024 {
            Resolver::V3
        } else if *self >= Edition::Edition2021 {
            Resolver::V2
        } else {
            Resolver::V1
        }
    }
}

impl std::fmt::Display for Edition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Edition::Edition2015 => f.write_str("2015"),
            Edition::Edition2018 => f.write_str("2018"),
            Edition::Edition2021 => f.write_str("2021"),
            Edition::Edition2024 => f.write_str("2024"),
        }
    }
}

impl FromStr for Edition {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "2015" => Ok(Edition::Edition2015),
            "2018" => Ok(Edition::Edition2018),
            "2021" => Ok(Edition::Edition2021),
            "2024" => Ok(Edition::Edition2024),
            s if s.parse().map_or(false, |y: u16| y > 2024 && y < 2050) => bail_out!(
                "this version of Cargo is older than the `{}` edition, \
                 and only supports `2015`, `2018`, `2021`, and `2024` editions.",
                s
            ),
            s => bail_out!(
                "supported edition values are `2015`, `2018`, `2021`, or `2024`, \
                 but `{}` is unknown",
                s
            ),
        }
    }
}

impl From<&str> for Edition {
    fn from(value: &str) -> Self {
        value.parse().expect("invalid edition")
    }
}

impl From<String> for Edition {
    fn from(value: String) -> Self {
        value.parse().expect("invalid edition")
    }
}
