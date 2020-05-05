use crate::configs::SimpleId;
use serde::Deserialize;

/// Don't deny unknown fields to allow multiple configs in one yaml-file.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub routing: Content,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Content {
    pub is_ch_dijkstra: Option<bool>,
    pub metrics: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Entry {
    pub id: SimpleId,
    pub alpha: Option<f64>,
    pub tolerated_scale: Option<f64>,
}
