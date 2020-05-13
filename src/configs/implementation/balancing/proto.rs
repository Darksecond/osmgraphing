use serde::Deserialize;
use std::path::PathBuf;

/// Don't deny unknown fields to allow multiple configs in one yaml-file.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub balancing: Content,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Content {
    pub results_file: PathBuf,
    pub num_iterations: usize,
}
