use serde::{Serialize, Deserialize};
use surrealdb::sql::{Value, json};

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct DBPart {
    pub id: String,
    pub name: String,
    pub image_url: String,
    pub model: String,
    pub manufactuer: String,
    pub release_date: u64,
    pub rating: f32,
    pub category: PartsCategory,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DBPartProps {
    pub name: String,
    pub image_url: String,
    pub model: String,
    pub manufactuer: String,
    pub release_date: u64,
    pub rating: f32,
    pub category: PartsCategory,
}

impl Into<Value> for DBPartProps {
    fn into(self) -> Value {
        convert_to_value(&self).unwrap()
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

#[derive(Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub enum PartsCategory {
    #[default]
    Basic,
    CPU(CPUProperties),
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct CPUProperties {
    pub cores: u32,
    pub threads: u32,
    pub max_frequency: f32,
    pub base_frequency: f32,
    pub max_tdp: u32,
    pub base_tdp: u32,
    pub cache: u32,
    pub max_ram_size: u32,
    pub max_memory_channels: u32,
    pub ecc_memory_supported: bool,
    pub max_pcie_lanes: u32,
    pub max_supported_pcie_version: f32,
    pub socket: String,
    pub max_temperature: u32,
}

fn convert_to_value<T>(value: &T) -> anyhow::Result<Value>
where T: Serialize
{
    let json_value = serde_json::to_string(value)?;
    let value = json(&json_value)?;
    Ok(value)
}
