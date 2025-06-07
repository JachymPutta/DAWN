use serde::{Deserialize, Deserializer, Serialize, Serializer};
use smol_str::SmolStr;
use std::ops::Deref;

/// A serializable wrapper around `SmolStr`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SerSmolStr(pub SmolStr);

impl Serialize for SerSmolStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for SerSmolStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(SerSmolStr(SmolStr::new(s)))
    }
}

impl From<SmolStr> for SerSmolStr {
    fn from(s: SmolStr) -> Self {
        SerSmolStr(s)
    }
}

impl From<&str> for SerSmolStr {
    fn from(s: &str) -> Self {
        SerSmolStr(SmolStr::new(s))
    }
}

impl From<SerSmolStr> for SmolStr {
    fn from(s: SerSmolStr) -> Self {
        s.0
    }
}

impl Deref for SerSmolStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<SmolStr> for SerSmolStr {
    fn eq(&self, other: &SmolStr) -> bool {
        &self.0 == other
    }
}

impl PartialEq<SerSmolStr> for SmolStr {
    fn eq(&self, other: &SerSmolStr) -> bool {
        self == &other.0
    }
}
