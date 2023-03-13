use std::{collections::HashMap, str::FromStr};

use serde::{Serialize, Deserialize};
use strum::{EnumIter, IntoEnumIterator, Display, EnumString};
#[cfg(feature = "surreal")]
use surrealdb::sql::{Value, json};
use traits::PartProperties;
use types::StringenFloat;

pub mod types;
pub mod traits;

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct DBPart {
    pub id: String,
    pub name: String,
    pub image_url: String,
    pub model: String,
    pub manufactuer: String,
    pub release_date: String,
    pub rating: StringenFloat,
    pub category: PartsCategory,
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct DBPartProps {
    pub name: String,
    pub image_url: String,
    pub model: String,
    pub manufactuer: String,
    pub release_date: String,
    pub rating: StringenFloat,
    pub category: PartsCategory,
}

impl PartProperties for DBPartProps {}

#[cfg(feature = "surreal")]
impl Into<Value> for DBPartProps {
    fn into(self) -> Value {
        let value = convert_to_value(&self).unwrap();
        value
    }
}

#[derive(Serialize, Deserialize)]
pub struct StatusResponse {
    pub functional: bool
}

#[derive(Serialize, Deserialize)]
pub struct GetPartProps {
    /// None: Get all parts
    /// Some: Get part with specified id
    pub id: Option<String>,
    pub limit: u32,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Default, Debug, EnumIter, Display, EnumString)]
pub enum PartsCategory {
    #[default]
    Basic,
    CPU(CPUProperties),
}

impl PartProperties for PartsCategory {
    fn to_string_vec(&self) -> anyhow::Result<std::collections::HashMap<String, String>> {
        match self {
            PartsCategory::Basic => Ok(HashMap::new()),
            PartsCategory::CPU(props) => props.to_string_vec(),
        }
    }
}

impl PartsCategory {
    pub fn get_all_variats() -> Vec<String> {
        let mut variants: Vec<String> = Vec::new();
        for variant in PartsCategory::iter() {
            variants.push(variant.to_string());
        }

        return variants;
    }

    pub fn from_string(string: &str) -> PartsCategory {
        PartsCategory::from_str(string).unwrap_or(PartsCategory::Basic)
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct CPUProperties {
    pub cores: u32,
    pub threads: u32,
    pub max_frequency: String,
    pub base_frequency: String,
    pub max_tdp: String,
    pub base_tdp: String,
    pub cache: String,
    pub max_ram_size: String,
    pub max_memory_channels: u32,
    pub ecc_memory_supported: bool,
    pub max_pcie_lanes: u32,
    pub max_supported_pcie_version: String,
    pub socket: String,
    pub max_temperature: String,
}

impl PartProperties for CPUProperties {}

#[cfg(feature = "surreal")]
fn convert_to_value<T>(value: &T) -> anyhow::Result<Value>
where T: Serialize
{
    let json_value = serde_json::to_string(value)?;
    let value = json(&json_value)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{StringenFloat, DBPartProps};

    #[test]
    fn stringen_float() -> anyhow::Result<()> {
        let stringen_float = StringenFloat::new(2.5);
        let json = serde_json::to_value(stringen_float.clone())?;
        let decompiled_float: StringenFloat = serde_json::from_value(json)?;

        assert!(stringen_float == decompiled_float);

        Ok(())
    }

    #[test]
    fn stringen_float_2() -> anyhow::Result<()> {
        let stringen_float = StringenFloat::new(0 as f64);
        let json = serde_json::to_value(stringen_float.clone())?;
        let decompiled_float: StringenFloat = serde_json::from_value(json)?;

        assert!(stringen_float == decompiled_float);

        Ok(())
    }

    #[test]
    fn stringen_float_from_json() -> anyhow::Result<()> {
        let value = 3.5;
        let json = json!(format!("{}", value));
        let stringen_float = StringenFloat::new(value);
        let stringen_float_from_json: StringenFloat = serde_json::from_value(json)?;

        assert!(stringen_float == stringen_float_from_json);

        Ok(())
    }

    #[test]
    fn stringen_float_from_json_2() -> anyhow::Result<()> {
        let value = 0;
        let json = json!(format!("{}", value));
        let stringen_float = StringenFloat::new(value as f64);
        let stringen_float_from_json: StringenFloat = serde_json::from_value(json)?;

        assert!(stringen_float == stringen_float_from_json);

        Ok(())
    }

    #[test]
    fn db_parts_from_json() -> anyhow::Result<()> {
        let db_part = DBPartProps {
            name: "Yotu".into(),
            image_url: "".into(),
            model: "LKFHDS".into(),
            manufactuer: "Chinese".into(),
            release_date: "22Q2".to_string(),
            rating: 4.5.into(),
            category: crate::PartsCategory::Basic,
        };
        
        let json = serde_json::to_value(db_part.clone())?;
        let db_parts_from_json: DBPartProps = serde_json::from_value(json)?;

        println!("DBPart: {:?}", db_part.clone());
        println!("DBPartJson: {:?}", db_parts_from_json.clone());

        assert!(db_part == db_parts_from_json);

        Ok(())
    }
}
