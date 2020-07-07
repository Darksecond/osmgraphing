use crate::configs::SimpleId;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "with_shortcuts")]
    pub is_writing_shortcuts: Option<bool>,
    pub ids: Vec<Category>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Id(SimpleId),
    Ignored,
}
