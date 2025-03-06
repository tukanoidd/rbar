use derive_more::derive::{Deref, DerefMut, From, Into};
use iced::Color;
use serde::{de::Visitor, Deserialize, Deserializer};

#[derive(Clone, Copy, Deref, DerefMut, From, Into)]
pub struct ColorWrap(Color);

impl<'de> Deserialize<'de> for ColorWrap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ColorVisitor).map(Into::into)
    }
}

struct ColorVisitor;

impl Visitor<'_> for ColorVisitor {
    type Value = Color;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "color in hex format")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Color::parse(v).ok_or_else(|| E::custom("Invalid color format!"))
    }
}
