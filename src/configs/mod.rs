use crate::{helpers, helpers::MapFileExt};
use serde::Deserialize;
use std::{
    fmt,
    fmt::Display,
    path::{Path, PathBuf},
};

pub mod graph;

/// Storing (default) settings for parsing the graph.
///
/// # Configuration
///
/// ## Set config-values with yaml-file
///
/// You can change the configuration with an input-file (`*.yaml`).
/// With this `yaml`-config, the parser can be adjusted to parse (edge-)metrics in the order as provided by the config-file.
/// This can help especially with map-files in `fmi`-format, since the metrics are read sequentially.
/// But since `pbf`-files does not provide a column-based metric-list, but intrinsically by parsing `osm`-data, you can distinguish between default-metrics and custom-metrics via the key `category`.
///
/// Default-categories are
/// - `id`, which is not a metric per se and stored differently, but needed for `csv`-like `fmi`-format
/// - `length` in meters
/// - `maxspeed` in km/h
/// - `duration` in milliseconds
/// - `lane-count`
///
/// Internally, a default-metric uses provided calculation-rules to be calculated by other default-categories as well (like the duration from length and maxspeed).
///
/// Keep in mind, that metrics (except for id) are stored as `u32` for better maintainability and efficiency.
/// Note that you can convert `floats` into `u32` by moving the comma.
///
///
/// ### Specifying routing (in the future)
///
/// Further, the metrics, which are used in the routing, can be listed in the routing-section with their previously defined id.
/// Comparisons are made using pareto-optimality, so there is no comparison between metrics.
/// In case you'll use personlized-routing, default-preferences can be set with weights.
/// The example below shows a routing-case, where the metric `length` is weighted with `169 / (169 + 331) = 33.8 %` while the metric `duration` is weighted with `331 / (169 + 331) = 66.2 %`.
///
///
/// ### Supported structure
///
/// The following `yaml`-structure is supported.
/// The used values below are not the defaults.
/// For the defaults, see `resources/configs/`.
///
/// Please note, that metric-categories can be used multiple times.
///
///
/// Every metric (!= every category) will be stored in the graph, if mentioned in this `yaml`-file.
/// If a metric is mentioned, but `provided` is false, it will be calculated (e.g. edge-length from node-coordinates and haversine).
/// Please note, that metrics being calculated (like the duration from length and maxspeed) need the respective metrics to be calculated.
///
/// ```yaml
/// graph:
///   vehicles:
///     # car|bicycle|pedestrian
///     type: car
///     # Possible values: true|false
///     # Value `false` leads to more edges, because edges are added, which are okay but not suitable for this vehicle-type.
///     is-graph-suitable: false
///   edges:
///     # The order here matters if the map-file has a metric-order, like `fmi`-files.
///     # Each metric below will be stored in the graph.
///     # Metrics below, which have `provided=false`, will be calculated by other metrics and the result is being stored.
///     # All other metrics are calculated, if possible, when asked for.
///     #
///     # Default metrics are length and maxspeed.
///     metrics:
///     # not a metric per se but needed here due to `csv`-like `fmi`-format
///     - id: src-id
///       type: id
///     - type: length
///       # Possible values: true|false
///       # Value `false` leads to calculate the value via coordinates and haversine.
///       provided: false
///     - type: maxspeed
///       # Possible values: true|false
///       # Value `false` leads to calculate the value via length and duration.
///       provided: true
///     - type: duration
///       # Possible values: true|false
///       # Value `false` leads to calculate the value via length and maxspeed.
///       provided: false
///     - type: lane-count
///     - id: <String>
///       type: custom
///     - id: <String>
///       type: ignore
///
/// routing: # example with two metrics and default-weights
///   metrics: [length, duration]
///   preferences:
///   - id: length
///     alpha: 169
///   - id: duration
///     alpha: 331
/// ```
#[derive(Debug, Deserialize)]
pub struct Config {
    pub graph: graph::Config,
}

impl Config {
    pub fn from_path<P: AsRef<Path> + ?Sized>(path: &P) -> Result<Config, String> {
        let file = helpers::open_file(path)?;
        // let cfg: Result<Config, serde_yaml::Error> = serde_yaml::from_reader(file);
        match serde_yaml::from_reader(file) {
            Ok(cfg) => Ok(cfg),
            Err(e) => Err(format!("{}", e)),
        }
    }

    pub fn from_map_file<P: AsRef<Path> + ?Sized>(path: &P) -> Result<Config, String> {
        let mut cfg = match MapFileExt::from_path(path)? {
            MapFileExt::PBF => Config::from_path("resources/configs/pbf.yaml"),
            MapFileExt::FMI => Config::from_path("resources/configs/fmi.yaml"),
        }?;
        cfg.graph.map_file = Some(PathBuf::from(path.as_ref()));
        Ok(cfg)
    }

    pub fn from_yaml<P: AsRef<Path> + ?Sized>(_path: &P) -> Result<Config, String> {
        unimplemented!("TODO implement Config::from_yaml")
    }
}

#[derive(Debug, Deserialize)]
pub enum VehicleCategory {
    Car,
    Bicycle,
    Pedestrian,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(from = "String")]
pub struct MetricId(pub String);

impl From<String> for MetricId {
    fn from(id: String) -> MetricId {
        MetricId(id)
    }
}

impl From<&str> for MetricId {
    fn from(id: &str) -> MetricId {
        MetricId(id.to_owned())
    }
}

impl Display for MetricId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Types of metrics to consider when parsing a map.
#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
pub enum MetricCategory {
    Length,
    Maxspeed,
    Duration,
    LaneCount,
    Custom,
    Id,
    Ignore,
}

impl Display for MetricCategory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl MetricCategory {
    pub fn must_be_positive(&self) -> bool {
        match self {
            MetricCategory::Length
            | MetricCategory::Maxspeed
            | MetricCategory::Duration
            | MetricCategory::LaneCount => true,
            MetricCategory::Custom | MetricCategory::Id | MetricCategory::Ignore => false,
        }
    }

    pub fn is_ignored(&self) -> bool {
        match self {
            MetricCategory::Id | MetricCategory::Ignore => true,
            MetricCategory::Length
            | MetricCategory::Maxspeed
            | MetricCategory::Duration
            | MetricCategory::LaneCount
            | MetricCategory::Custom => false,
        }
    }

    fn expected_calc_rules(&self) -> Vec<MetricCategory> {
        match self {
            MetricCategory::Maxspeed => vec![MetricCategory::Length, MetricCategory::Duration],
            MetricCategory::Duration => vec![MetricCategory::Length, MetricCategory::Maxspeed],
            MetricCategory::Length
            | MetricCategory::LaneCount
            | MetricCategory::Custom
            | MetricCategory::Id
            | MetricCategory::Ignore => vec![],
        }
    }
}
