use std::{num::ParseFloatError, fmt::Display};

use serde::{de::Visitor, Serialize, Deserialize};

struct StringenFloatVisitor;

impl Visitor<'_> for StringenFloatVisitor {
    type Value = StringenFloat;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an float represented by a string")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where E: serde::de::Error, 
    {
        let result = StringenFloat::try_from(v.clone());

        match result {
            Ok(value) => Ok(value),
            Err(_) => Err(E::custom(format!("Invalid float: {}", v))),
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where E: serde::de::Error, 
    {
        let result = StringenFloat::try_from(v.to_string());

        match result {
            Ok(value) => Ok(value),
            Err(_) => Err(E::custom(format!("Invalid float: {}", v))),
        }
    }
}

// Using to convert from BigDecimal in SurrealDB
#[derive(Clone, PartialEq, Debug)]
pub struct StringenFloat(String);

impl Default for StringenFloat {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl Display for StringenFloat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<f64> for StringenFloat {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

impl Into<f64> for StringenFloat {
    fn into(self) -> f64 {
        self.get()
    }
}

impl From<f32> for StringenFloat {
    fn from(value: f32) -> Self {
        Self::new(value as f64)
    }
}

impl Into<f32> for StringenFloat {
    fn into(self) -> f32 {
        self.get() as f32
    }
}

impl TryFrom<String> for StringenFloat {
    type Error = ParseFloatError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let float = value.parse::<f64>()?;
        Ok(Self::new(float))
    }
}

impl StringenFloat {
    pub fn new(value: f64) -> Self {
        Self(value.to_string())
    }

    pub fn get(&self) -> f64 {
        self.0.parse::<f64>().unwrap()
    }
}

impl Serialize for StringenFloat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer 
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for StringenFloat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> 
    {
        deserializer.deserialize_str(StringenFloatVisitor)
    }
}
