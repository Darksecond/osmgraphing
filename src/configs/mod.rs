use serde::Deserialize;
use std::{fmt, fmt::Display};

#[cfg(feature = "gpl")]
pub mod balancing;
#[cfg(feature = "gpl")]
pub mod evaluating_balance;
pub mod parsing;
pub mod routing;
pub mod writing;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(from = "String")]
pub struct SimpleId(pub String);

impl From<String> for SimpleId {
    fn from(id: String) -> SimpleId {
        SimpleId(id)
    }
}

impl From<&str> for SimpleId {
    fn from(id: &str) -> SimpleId {
        SimpleId(id.to_owned())
    }
}

impl Display for SimpleId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for SimpleId {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl AsRef<String> for SimpleId {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl AsRef<SimpleId> for SimpleId {
    fn as_ref(&self) -> &SimpleId {
        &self
    }
}
