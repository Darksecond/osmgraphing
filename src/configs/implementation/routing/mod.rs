use crate::{
    configs,
    defaults::{self, capacity::DimVec},
    helpers::err,
    io::SupportingFileExts,
};
use smallvec::smallvec;
use std::{
    fs::OpenOptions,
    path::{Path, PathBuf},
};
pub mod proto;
pub mod raw;

/// # Specifying routing (TODO update text)
///
/// Further, the metrics, which are used in the routing, can be listed in the routing-section with their previously defined id.
/// Comparisons are made using pareto-optimality, so there is no comparison between metrics.
/// In case you'll use personlized-routing, default-preferences can be set with weights.
/// The example below shows a routing-case, where the metric `distance` is weighted with `169 / (169 + 331) = 33.8 %` while the metric `duration` is weighted with `331 / (169 + 331) = 66.2 %`.
#[derive(Clone, Debug)]
pub struct Config {
    pub route_pairs_file: Option<PathBuf>,
    pub is_ch_dijkstra: bool,
    pub alphas: DimVec<f64>,
    pub tolerated_scales: DimVec<f64>,
}

impl SupportingFileExts for Config {
    fn supported_exts<'a>() -> &'a [&'a str] {
        &["yaml"]
    }
}

impl Config {
    pub fn try_from_str(
        yaml_str: &str,
        parsing_cfg: &configs::parsing::Config,
    ) -> err::Result<Config> {
        let proto_cfg = {
            match serde_yaml::from_str(yaml_str) {
                Ok(proto_cfg) => proto_cfg,
                Err(e) => return Err(format!("{}", e).into()),
            }
        };
        Config::try_from_proto(proto_cfg, parsing_cfg)
    }

    pub fn from_str(yaml_str: &str, parsing_cfg: &configs::parsing::Config) -> Config {
        match Config::try_from_str(yaml_str, parsing_cfg) {
            Ok(cfg) => cfg,
            Err(msg) => panic!("{}", msg),
        }
    }

    fn try_from_proto(
        proto_cfg: proto::Config,
        parsing_cfg: &configs::parsing::Config,
    ) -> err::Result<Config> {
        let dim = parsing_cfg.edges.metrics.units.len();

        // Alpha is 0.0 because non-mentioned id will not be considered.
        let mut alphas = smallvec![0.0; dim];
        // Same argument holds for the toleration.
        let mut tolerated_scales = smallvec![defaults::routing::TOLERATED_SCALE_INF; dim];

        for entry in proto_cfg.metrics.into_iter() {
            let metric_idx = parsing_cfg.edges.metrics.try_idx_of(&entry.id)?;
            alphas[*metric_idx] = entry.alpha;
            tolerated_scales[*metric_idx] = entry.tolerated_scale;
        }

        Ok(Config {
            route_pairs_file: proto_cfg.route_pairs_file,
            is_ch_dijkstra: proto_cfg.is_ch_dijkstra,
            alphas,
            tolerated_scales,
        })
    }

    fn _from_proto(proto_cfg: proto::Config, parsing_cfg: &configs::parsing::Config) -> Config {
        match Config::try_from_proto(proto_cfg, parsing_cfg) {
            Ok(cfg) => cfg,
            Err(msg) => panic!("{}", msg),
        }
    }

    pub fn try_from_yaml<P: AsRef<Path> + ?Sized>(
        path: &P,
        parsing_cfg: &configs::parsing::Config,
    ) -> err::Result<Config> {
        let path = path.as_ref();
        let file = {
            Config::find_supported_ext(path)?;
            OpenOptions::new()
                .read(true)
                .open(path)
                .expect(&format!("Couldn't open {}", path.display()))
        };

        let proto_cfg = match serde_yaml::from_reader(file) {
            Ok(proto_cfg) => proto_cfg,
            Err(e) => return Err(format!("{}", e).into()),
        };
        Config::try_from_proto(proto_cfg, parsing_cfg)
    }

    pub fn from_yaml<P: AsRef<Path> + ?Sized>(
        path: &P,
        parsing_cfg: &configs::parsing::Config,
    ) -> Config {
        match Config::try_from_yaml(path, parsing_cfg) {
            Ok(cfg) => cfg,
            Err(msg) => panic!("{}", msg),
        }
    }
}
