use crate::network::vehicles::Category as VehicleCategory;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Config {
    pub category: VehicleCategory,
    pub are_drivers_picky: bool,
}
