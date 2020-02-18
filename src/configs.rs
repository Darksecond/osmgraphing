use std::{fmt, fmt::Display};

pub enum VehicleType {
    Car,
    Bicycle,
    Pedestrian,
}

/// Types of metrics to consider when parsing a map.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum MetricType {
    Id,
    Length { provided: bool },
    Maxspeed { provided: bool },
    Duration { provided: bool },
    LaneCount,
    Custom,
    Ignore,
}

impl Display for MetricType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MetricType::Id => String::from("id"),
                MetricType::Length { provided } => format!("length (provided: {})", provided),
                MetricType::Maxspeed { provided } => format!("maxspeed (provided: {})", provided),
                MetricType::Duration { provided } => format!("duration (provided: {})", provided),
                MetricType::LaneCount => String::from("lane-count"),
                MetricType::Custom => String::from("custom"),
                MetricType::Ignore => String::from("ignored"),
            }
        )
    }
}

pub mod graph {
    use super::{edges, paths, VehicleType};

    /// Storing (default) settings for parsing the graph.
    ///
    /// # Configuration
    ///
    /// ## Default
    ///
    /// The default-configuration contains basic metrics of the graph:
    /// - length (in meters)
    /// - maxspeed (in km/h)
    ///
    ///
    /// ## Changing the defaults with yaml-file
    ///
    /// You can change the configuration with an input-file (`*.yaml`).
    /// With this `yaml`-config, the parser can be adjusted to parse (edge-)metrics in the order as provided by the config-file.
    /// This can help especially with map-files in `fmi`-format, since the metrics are read sequentially.
    ///
    /// Further, the metrics, which are used in the routing, can be specified with their previously defined `id`.
    /// Comparisons are made using pareto-optimality, so there is no comparison between metrics.
    /// In case you'll use personlized-routing, default-preferences can be set with weights.
    /// The example below shows a routing-case, where the metric `length` is weighted with `169 / (169 + 331) = 33.8 %` while the metric `duration` is weighted with `331 / (169 + 331) = 66.2 %`.
    ///
    /// The following `yaml`-structure is supported.
    /// The used values below are the defaults.
    ///
    /// Please note, that just a few metric-types can be used multiple times, namely:
    /// - type: `ignore`
    ///
    ///
    /// Every metric will be stored in the graph, if mentioned in this `yaml`-file.
    /// If a metric is mentioned, but `provided` is false, it will be calculated (e.g. edge-length from node-coordinates and haversine).
    /// Please note, that metrics being calculated (like duration from length and maxspeed) need the respective metrics to be calculated.
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
    ///     - id: <String>
    ///       type: length
    ///       # Possible values: true|false
    ///       # Value `false` leads to calculate the value via coordinates and haversine.
    ///       provided: false
    ///     - id: <String>
    ///       type: maxspeed
    ///       # Possible values: true|false
    ///       provided: true
    ///     - id: <String>
    ///       type: duration
    ///       # Possible values: true|false
    ///       # Value `false` leads to calculate the value via length and maxspeed.
    ///       provided: false
    ///     - id: <String>
    ///       type: lane-count
    ///     - id: <String>
    ///       type: u32
    ///     - id: <String>
    ///       type: ignore
    ///
    /// routing: # example with two metrics and weights
    ///   metrics: [<id>, <id>]
    ///   preferences:
    ///   - id: <String>
    ///     alpha: 169
    ///   - id: <String>
    ///     alpha: 331
    /// ```
    pub struct Config {
        pub vehicle_type: VehicleType,
        pub edges: edges::Config,
        pub paths: paths::Config,
        pub is_graph_suitable: bool,
    }

    impl Default for Config {
        fn default() -> Config {
            Config {
                paths: Default::default(),
                vehicle_type: VehicleType::Car,
                edges: Default::default(),
                is_graph_suitable: false,
            }
        }
    }

    impl Config {}
}

pub mod edges {
    use super::MetricType;

    pub struct Config {
        pub metric_ids: Vec<String>,
        pub metric_types: Vec<MetricType>,
    }

    impl Default for Config {
        fn default() -> Config {
            Config {
                metric_ids: vec![
                    String::from("src-id"),
                    String::from("dst-id"),
                    String::from("length"),
                    String::from("?"),
                    String::from("maxspeed"),
                ],
                metric_types: vec![
                    MetricType::Id,
                    MetricType::Id,
                    MetricType::Length { provided: true },
                    MetricType::Ignore,
                    MetricType::Maxspeed { provided: true },
                ],
            }
        }
    }

    impl Config {
        pub fn get(&self, idx: usize) -> Option<(&String, &MetricType)> {
            Some((self.metric_ids.get(idx)?, self.metric_types.get(idx)?))
        }

        pub fn push(&mut self, id: String, metric_type: MetricType) {
            self.metric_ids.push(id);
            self.metric_types.push(metric_type);
        }

        pub fn remove(&mut self, idx: usize) -> (String, MetricType) {
            (self.metric_ids.remove(idx), self.metric_types.remove(idx))
        }
    }
}

pub mod paths {
    use std::path::PathBuf;

    pub struct Config {
        pub map_file: PathBuf,
    }

    impl Default for Config {
        fn default() -> Config {
            Config {
                map_file: PathBuf::from("resources/maps/simple_stuttgart.fmi"),
            }
        }
    }

    impl Config {}
}

pub mod routing {
    pub struct Config {}
}
