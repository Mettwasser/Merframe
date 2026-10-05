use serde::{Deserialize, Serialize};

pub type DucanatorItems = Vec<DucanatorItem>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DucanatorItem {
    pub unique_name: String,
    pub count: u32,
}
