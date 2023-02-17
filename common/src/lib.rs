use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct DBPart {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DBPartProps {
    pub name: String,
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
