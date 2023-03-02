use std::fmt::Display;

use serde::{Serialize, Deserialize};
use strum::{EnumIter, IntoEnumIterator, Display};
#[cfg(feature = "surreal")]
use surrealdb::sql::{Value, json};
use types::StringenFloat;

mod types;

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct DBPart {
    pub id: String,
    pub name: String,
    pub image_url: String,
    pub model: String,
    pub manufactuer: String,
    pub release_date: u64,
    pub rating: StringenFloat,
    pub category: PartsCategory,
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
pub struct DBPartProps {
    pub name: String,
    pub image_url: String,
    pub model: String,
    pub manufactuer: String,
    pub release_date: u64,
    pub rating: StringenFloat,
    pub category: PartsCategory,
}

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

#[derive(Clone, Serialize, Deserialize, PartialEq, Default, Debug, EnumIter, Display)]
pub enum PartsCategory {
    #[default]
    Basic,
    CPU(CPUProperties),
}

impl PartsCategory {
    pub fn get_all_variats() -> Vec<String> {
        let mut variants: Vec<String> = Vec::new();
        for variant in PartsCategory::iter() {
            variants.push(variant.to_string());
        }

        return variants;
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct CPUProperties {
    pub cores: u32,
    pub threads: u32,
    pub max_frequency: StringenFloat,
    pub base_frequency: StringenFloat,
    pub max_tdp: u32,
    pub base_tdp: u32,
    pub cache: u32,
    pub max_ram_size: u32,
    pub max_memory_channels: u32,
    pub ecc_memory_supported: bool,
    pub max_pcie_lanes: u32,
    pub max_supported_pcie_version: StringenFloat,
    pub socket: String,
    pub max_temperature: u32,
}

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
            release_date: 12092235,
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
