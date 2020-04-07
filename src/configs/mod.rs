use crate::{helpers, io::SupportingFileExts};
use serde::Deserialize;
use std::{fmt, fmt::Display, path::Path};

mod raw;

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
/// Default-categories are described in `EdgeCategory`.
///
/// Internally, a default-metric uses provided calculation-rules to be calculated by other default-categories as well (like the duration from distance and maxspeed).
///
/// Keep in mind, that metrics (except for id) are stored as `f64` for better maintainability and efficiency.
///
///
/// ### Specifying routing (in the future)
///
/// Further, the metrics, which are used in the routing, can be listed in the routing-section with their previously defined id.
/// Comparisons are made using pareto-optimality, so there is no comparison between metrics.
/// In case you'll use personlized-routing, default-preferences can be set with weights.
/// The example below shows a routing-case, where the metric `distance` is weighted with `169 / (169 + 331) = 33.8 %` while the metric `duration` is weighted with `331 / (169 + 331) = 66.2 %`.
///
///
/// ### Supported structure
///
/// The supported `yaml`-structure can be seen in `resources/configs/schema.yaml`.
///
// Every metric (!= every category) will be stored in the graph, if mentioned in this `yaml`-file.
/// If a metric is mentioned, but `provided` is false, it will be calculated (e.g. edge-distance from node-coordinates and haversine).
/// Please note, that metrics being calculated (like the duration from distance and maxspeed) need the respective metrics to be calculated.
#[derive(Debug, Deserialize)]
#[serde(from = "raw::Config")]
pub struct Config {
    pub parser: parser::Config,
    pub generator: Option<generator::Config>,
    pub routing: Option<routing::Config>,
}

impl SupportingFileExts for Config {
    fn supported_exts<'a>() -> &'a [&'a str] {
        &["yaml"]
    }
}

impl Config {
    pub fn from_yaml<P: AsRef<Path> + ?Sized>(path: &P) -> Result<Config, String> {
        let file = {
            Config::find_supported_ext(path)?;
            helpers::open_file(path)?
        };
        match serde_yaml::from_reader(file) {
            Ok(cfg) => Ok(cfg),
            Err(msg) => Err(format!("{}", msg)),
        }
    }
}

pub mod parser {
    pub use edges::Category as EdgeCategory;
    pub use nodes::Category as NodeCategory;
    use std::path::PathBuf;

    #[derive(Debug)]
    pub struct Config {
        pub map_file: PathBuf,
        pub vehicles: vehicles::Config,
        pub nodes: nodes::Config,
        pub edges: edges::Config,
    }

    pub mod vehicles {
        use crate::network::VehicleCategory;

        #[derive(Debug)]
        pub struct Config {
            pub category: VehicleCategory,
            pub are_drivers_picky: bool,
        }
    }

    pub mod nodes {
        use serde::Deserialize;

        #[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
        pub enum Category {
            NodeId,
            Latitude,
            Longitude,
            Level,
            Ignore,
        }

        #[derive(Debug)]
        pub struct Config {
            categories: Vec<Category>,
        }

        impl Config {
            pub fn new(categories: Vec<Category>) -> Config {
                Config { categories }
            }

            pub fn categories(&self) -> &Vec<Category> {
                &self.categories
            }
        }
    }

    pub mod edges {
        use crate::{configs::SimpleId, defaults::capacity::DimVec};
        use serde::Deserialize;
        use smallvec::smallvec;
        use std::fmt::{self, Display};

        pub mod metrics {
            use crate::{configs::SimpleId, defaults::capacity::DimVec};
            use serde::Deserialize;

            #[derive(Debug)]
            pub struct Config {
                pub categories: DimVec<super::Category>, // TODO replace by Unit
                pub ids: DimVec<SimpleId>,
                // metric_categories: DimVec<Category>,
                // are_metrics_provided: DimVec<bool>,
                // metric_ids: DimVec<SimpleId>,
                // metric_indices: BTreeMap<SimpleId, MetricIdx>,
                // calc_rules: DimVec<DimVec<(Category, MetricIdx)>>,
            }

            #[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
            pub enum Unit {
                Meters,
                Kilometers,
                Seconds,
                Minutes,
                Hours,
                KilometersPerHour,
                LaneCount,
                F64,
            }

            impl Config {
                pub fn dim(&self) -> usize {
                    self.categories.len()
                }
            }
        }

        #[derive(Debug)]
        pub struct Config {
            // store all for order
            pub categories: Vec<Category>,
            pub ids: Vec<SimpleId>,

            // store only metrics for quick access
            pub metrics: metrics::Config,
        }

        impl Config {
            pub fn new(
                categories: Vec<Category>,
                ids: Vec<SimpleId>,
                metrics: metrics::Config,
            ) -> Config {
                Config {
                    categories,
                    ids,
                    metrics,
                }
            }
        }

        /// Types of metrics to consider when parsing a map.
        ///
        /// - `SrcId`/`DstId`, which is not a metric per se and stored differently, but needed for `csv`-like `fmi`-format.
        /// - `Ignore - SrcIdx`/`Ignore - DstIdx`, which are needed to be defined here for using their id in a generator afterwards.
        /// - `Meters` provided in meters, but internally stored as kilometers
        /// - `KilometersPerHour` in km/h
        /// - `Seconds`
        /// - `LaneCount`
        /// - `F64`, which is just the plain f64-value
        /// - `Ignore`, which is used in `csv`-like `fmi`-maps to jump over columns
        #[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
        pub enum Category {
            Meters,
            KilometersPerHour,
            Seconds,
            LaneCount,
            F64,
            SrcId,
            DstId,
            ShortcutEdgeIdx,
            Ignore,
        }

        impl Display for Category {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                fmt::Debug::fmt(self, f)
            }
        }

        impl Category {
            pub fn must_be_positive(&self) -> bool {
                match self {
                    Category::Meters
                    | Category::KilometersPerHour
                    | Category::Seconds
                    | Category::LaneCount => true,
                    Category::F64
                    | Category::ShortcutEdgeIdx
                    | Category::SrcId
                    | Category::DstId
                    | Category::Ignore => false,
                }
            }

            pub fn is_metric(&self) -> bool {
                match self {
                    Category::SrcId
                    | Category::DstId
                    | Category::ShortcutEdgeIdx
                    | Category::Ignore => false,
                    Category::Meters
                    | Category::KilometersPerHour
                    | Category::Seconds
                    | Category::LaneCount
                    | Category::F64 => true,
                }
            }

            pub fn expected_calc_rules(&self) -> DimVec<Category> {
                match self {
                    Category::KilometersPerHour => smallvec![Category::Meters, Category::Seconds],
                    Category::Seconds => smallvec![Category::Meters, Category::KilometersPerHour],
                    Category::Meters
                    | Category::LaneCount
                    | Category::F64
                    | Category::ShortcutEdgeIdx
                    | Category::SrcId
                    | Category::DstId
                    | Category::Ignore => smallvec![],
                }
            }
        }
    }
}

pub mod generator {
    use super::SimpleId;
    pub use edges::Category as EdgeCategory;
    pub use nodes::Category as NodeCategory;
    use std::path::PathBuf;

    pub mod nodes {
        use serde::Deserialize;

        #[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
        pub enum Category {
            NodeId,
            NodeIdx,
            Latitude,
            Longitude,
            Level,
            Ignore,
        }
    }

    pub mod edges {
        use serde::Deserialize;

        #[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
        pub enum Category {
            Meters,
            KilometersPerHour,
            Seconds,
            LaneCount,
            F64,
            SrcId,
            SrcIdx,
            DstId,
            DstIdx,
            Ignore,
        }
    }

    #[derive(Debug)]
    pub struct Config {
        pub map_file: PathBuf,
        pub nodes: Vec<NodeCategory>,
        pub edges: Vec<SimpleId>,
    }
}

pub mod routing {
    use crate::defaults::{self, capacity::DimVec};
    use smallvec::smallvec;

    #[derive(Clone, Debug)]
    pub struct Config {
        pub is_ch_dijkstra: bool,
        pub alphas: DimVec<f64>,
    }

    impl Config {
        pub fn from_str(
            yaml_str: &str,
            cfg_graph: &super::parser::Config,
        ) -> Result<Config, String> {
            let raw_cfg = super::raw::routing::Config::from_str(yaml_str)?;
            Ok(Config::from_raw(raw_cfg, cfg_graph))
        }

        pub fn from_raw(
            raw_cfg: super::raw::routing::Config,
            parser_cfg: &super::parser::Config,
        ) -> Config {
            let mut alphas = smallvec![0.0; parser_cfg.edges.metrics.dim()];

            for entry in raw_cfg.metrics.into_iter() {
                let alpha = entry.alpha.unwrap_or(defaults::routing::ALPHA);

                if let Some(metric_idx) = parser_cfg
                    .edges
                    .metrics
                    .ids
                    .iter()
                    .position(|id| id == &entry.id)
                {
                    alphas[metric_idx] = alpha;
                } else {
                    panic!(
                        "The given id {} should get alpha {}, but doesn't exist.",
                        entry.id, alpha
                    );
                }
            }

            Config {
                is_ch_dijkstra: raw_cfg.is_ch_dijkstra.unwrap_or(false),
                alphas,
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(from = "String")]
pub struct SimpleId(pub String);

impl From<String> for SimpleId {
    fn from(id: String) -> SimpleId {
        SimpleId(id)
    }
}

impl From<&str> for SimpleId {
    fn from(id: &str) -> SimpleId {
        SimpleId(id.to_owned())
    }
}

impl Display for SimpleId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
